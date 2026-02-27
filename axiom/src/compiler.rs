/// Axiom Bytecode Compiler
///
/// Compiles the AST (Item/Stmt/Expr tree) into a register-based Proto.
///
/// REGISTER ALLOCATION:
///   Simple linear-scan: each local variable occupies a fixed register.
///   Temporaries are allocated on top of locals.
///   Max 255 registers per frame (fits in 1 byte).
///
/// PASSES:
///   1. Compile expression tree → emit instructions, return result register
///   2. For declarations: hoist to globals table before body
///   3. Apply optimizer inline (peephole + constant folding)

use crate::ast::{Expr, Item, MatchPattern, Stmt, StringPart};
use crate::bytecode::{Instr, Op, Proto};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Register allocator
// ---------------------------------------------------------------------------

struct RegAlloc {
    /// Next free register
    next: u8,
    /// Register → local variable name
    locals: HashMap<String, u8>,
    /// Stack of "free" temporaries (for expression sub-trees)
    temp_top: u8,
}

impl RegAlloc {
    fn new() -> Self {
        RegAlloc { next: 0, locals: HashMap::new(), temp_top: 0 }
    }

    fn alloc_local(&mut self, name: impl Into<String>) -> u8 {
        let reg = self.next;
        self.locals.insert(name.into(), reg);
        self.next += 1;
        self.temp_top = self.next;
        reg
    }

    fn alloc_temp(&mut self) -> u8 {
        let reg = self.temp_top;
        self.temp_top += 1;
        if self.temp_top > self.next { self.next = self.temp_top; }
        reg
    }

    fn free_temp(&mut self, reg: u8) {
        if reg + 1 == self.temp_top {
            self.temp_top = reg;
        }
    }

    fn get_local(&self, name: &str) -> Option<u8> {
        self.locals.get(name).copied()
    }

    fn push_scope(&self) -> usize {
        self.locals.len()
    }

    fn pop_scope(&mut self, saved: usize) {
        let to_remove: Vec<String> = self.locals.iter()
            .filter(|(_, &r)| r as usize >= saved)
            .map(|(k, _)| k.clone())
            .collect();
        for k in to_remove {
            self.locals.remove(&k);
        }
        self.temp_top = self.next;
    }

    fn reg_count(&self) -> u8 { self.next }
}

// ---------------------------------------------------------------------------
// Global table
// ---------------------------------------------------------------------------

/// The compiler's global symbol table (shared across compilation units)
pub struct GlobalTable {
    pub names: Vec<String>,
    idx: HashMap<String, u16>,
}

impl GlobalTable {
    pub fn new() -> Self {
        GlobalTable { names: Vec::new(), idx: HashMap::new() }
    }

    pub fn intern(&mut self, name: &str) -> u16 {
        if let Some(&i) = self.idx.get(name) {
            return i;
        }
        let i = self.names.len() as u16;
        self.names.push(name.to_string());
        self.idx.insert(name.to_string(), i);
        i
    }

    pub fn get(&self, name: &str) -> Option<u16> {
        self.idx.get(name).copied()
    }
}

// ---------------------------------------------------------------------------
// Compiler context
// ---------------------------------------------------------------------------

pub struct Compiler<'g> {
    proto: Proto,
    regs: RegAlloc,
    globals: &'g mut GlobalTable,
    current_line: u32,
    /// Pending break-jump patches (for while/for loops)
    break_patches: Vec<Vec<usize>>,
    /// Loop start IP for continue
    loop_starts: Vec<usize>,
}

impl<'g> Compiler<'g> {
    pub fn new(source: impl Into<String>, globals: &'g mut GlobalTable) -> Self {
        Compiler {
            proto: Proto::new(source),
            regs: RegAlloc::new(),
            globals,
            current_line: 1,
            break_patches: Vec::new(),
            loop_starts: Vec::new(),
        }
    }

    fn emit(&mut self, instr: Instr) -> usize {
        self.proto.emit(instr, self.current_line)
    }

    fn emit_load_global(&mut self, dst: u8, name: &str) -> u8 {
        let idx = self.globals.intern(name);
        self.emit(Instr::abx(Op::LoadGlobal, dst, idx));
        dst
    }

    fn emit_store_global(&mut self, src: u8, name: &str) {
        let idx = self.globals.intern(name);
        self.emit(Instr::abx(Op::StoreGlobal, src, idx));
    }

    // -----------------------------------------------------------------------
    // Expression compilation
    // -----------------------------------------------------------------------

    /// Compile an expression into destination register `dst`.
    /// Returns the actual register holding the result.
    #[allow(unreachable_patterns)]
    pub fn compile_expr(&mut self, expr: &Expr, dst: u8) -> u8 {
        match expr {
            Expr::Number { value, .. } => {
                let n = *value;
                // Small integers as LoadInt (fits in i16)
                if n.fract() == 0.0 && n >= i16::MIN as f64 && n <= i16::MAX as f64 {
                    self.emit(Instr::asbx(Op::LoadInt, dst, n as i16));
                } else {
                    let idx = self.proto.add_float(n);
                    self.emit(Instr::abx(Op::LoadFloat, dst, idx));
                }
                dst
            }

            Expr::Boolean { value, .. } => {
                let op = if *value { Op::LoadTrue } else { Op::LoadFalse };
                self.emit(Instr::abc(op, dst, 0, 0));
                dst
            }

            Expr::String { value, .. } => {
                let idx = self.proto.add_string(value.clone());
                self.emit(Instr::abx(Op::LoadStr, dst, idx));
                dst
            }

            Expr::SelfRef { .. } => {
                if let Some(reg) = self.regs.get_local("self") {
                    if reg != dst { self.emit(Instr::abc(Op::Move, dst, reg, 0)); }
                    dst
                } else {
                    self.emit_load_global(dst, "self")
                }
            }

            Expr::Identifier { name, .. } => {
                // Local first
                if let Some(reg) = self.regs.get_local(name) {
                    if reg != dst {
                        self.emit(Instr::abc(Op::Move, dst, reg, 0));
                    }
                    return dst;
                }
                // Global
                self.emit_load_global(dst, name)
            }

            Expr::UnaryOp { op, operand, .. } => {
                let src = self.compile_expr(operand, dst);
                match op.as_str() {
                    "!" => { self.emit(Instr::abc(Op::Not, dst, src, 0)); }
                    "-" => { self.emit(Instr::abc(Op::Neg, dst, src, 0)); }
                    _   => {}
                }
                dst
            }

            Expr::BinaryOp { left, op, right, .. } => {
                // Short-circuit logical operators
                match op.as_str() {
                    "&&" => return self.compile_and(left, right, dst),
                    "||" => return self.compile_or(left, right, dst),
                    _ => {}
                }

                let t1 = self.regs.alloc_temp();
                let t2 = self.regs.alloc_temp();
                let lreg = self.compile_expr(left, t1);
                let rreg = self.compile_expr(right, t2);

                let bc_op = match op.as_str() {
                    "+"  => Op::Add, "-"  => Op::Sub,
                    "*"  => Op::Mul, "/"  => Op::Div,
                    "%"  => Op::Mod, "**" => Op::Pow,
                    "==" => Op::Eq,  "!=" => Op::Ne,
                    "<"  => Op::Lt,  "<=" => Op::Le,
                    ">"  => Op::Gt,  ">=" => Op::Ge,
                    ".." => Op::Concat,
                    _    => Op::Nop,
                };
                self.emit(Instr::abc(bc_op, dst, lreg, rreg));
                self.regs.free_temp(t2);
                self.regs.free_temp(t1);
                dst
            }

            Expr::Assign { target, value, .. } => {
                match target.as_ref() {
                    Expr::Identifier { name, .. } => {
                        if let Some(reg) = self.regs.get_local(name) {
                            self.compile_expr(value, reg);
                            if reg != dst { self.emit(Instr::abc(Op::Move, dst, reg, 0)); }
                            return dst;
                        }
                        // Global assign
                        let t = self.regs.alloc_temp();
                        let r = self.compile_expr(value, t);
                        self.emit_store_global(r, name);
                        if r != dst { self.emit(Instr::abc(Op::Move, dst, r, 0)); }
                        self.regs.free_temp(t);
                    }
                    Expr::MemberAccess { object, member, .. } => {
                        let t_obj = self.regs.alloc_temp();
                        let t_val = self.regs.alloc_temp();
                        let obj_r = self.compile_expr(object, t_obj);
                        let val_r = self.compile_expr(value, t_val);
                        let str_idx = self.proto.add_string(member.clone());
                        self.emit(Instr::abc(Op::SetProp, obj_r, val_r, 0));
                        // Patch Bx
                        let last = self.proto.code.len() - 1;
                        self.proto.code[last] = Instr::abx(Op::SetProp, obj_r, str_idx);
                        self.proto.code[last].0 |= (val_r as u32) << 24;
                        self.regs.free_temp(t_val);
                        self.regs.free_temp(t_obj);
                    }
                    Expr::Index { object, index, .. } => {
                        let t_obj = self.regs.alloc_temp();
                        let t_idx = self.regs.alloc_temp();
                        let t_val = self.regs.alloc_temp();
                        let obj_r = self.compile_expr(object, t_obj);
                        let idx_r = self.compile_expr(index, t_idx);
                        let val_r = self.compile_expr(value, t_val);
                        self.emit(Instr::abc(Op::SetIndex, obj_r, idx_r, val_r));
                        self.regs.free_temp(t_val);
                        self.regs.free_temp(t_idx);
                        self.regs.free_temp(t_obj);
                    }
                    _ => {
                        self.compile_expr(value, dst);
                    }
                }
                dst
            }

            Expr::Call { function, arguments, .. } => {
                // Func goes in t, args in t+1, t+2, ...
                let func_reg = self.regs.alloc_temp();
                let f_r = self.compile_expr(function, func_reg);

                let argc = arguments.len() as u8;
                let mut arg_regs = Vec::new();
                for arg in arguments.iter() {
                    let t = self.regs.alloc_temp();
                    let r = self.compile_expr(arg, t);
                    arg_regs.push(r);
                }

                // Move func into position if needed, args follow
                // iABC: A=dst, B=func_reg, C=argc
                self.emit(Instr::abc(Op::Call, dst, f_r, argc));

                for r in arg_regs.into_iter().rev() {
                    self.regs.free_temp(r);
                }
                self.regs.free_temp(func_reg);
                dst
            }

            Expr::MethodCall { object, method, arguments, .. } => {
                let t_obj = self.regs.alloc_temp();
                let obj_r = self.compile_expr(object, t_obj);
                let str_idx = self.proto.add_string(method.clone());
                let argc = arguments.len() as u8;

                // GetMethod into a temp, then Call
                let t_meth = self.regs.alloc_temp();
                self.emit(Instr::abx(Op::GetMethod, t_meth, str_idx));
                // Patch in obj register
                let last = self.proto.code.len() - 1;
                self.proto.code[last] = Instr(
                    (Op::GetMethod as u32) | ((t_meth as u32) << 8) | ((obj_r as u32) << 16) | ((str_idx as u32) << 8 << 8)
                );

                let mut arg_regs = Vec::new();
                for arg in arguments.iter() {
                    let t = self.regs.alloc_temp();
                    let r = self.compile_expr(arg, t);
                    arg_regs.push(r);
                }

                self.emit(Instr::abc(Op::Call, dst, t_meth, argc));

                for r in arg_regs.into_iter().rev() { self.regs.free_temp(r); }
                self.regs.free_temp(t_meth);
                self.regs.free_temp(t_obj);
                dst
            }

            Expr::MemberAccess { object, member, .. } => {
                let t = self.regs.alloc_temp();
                let obj_r = self.compile_expr(object, t);
                let str_idx = self.proto.add_string(member.clone());
                // GetProp dst, obj, str_idx  — IC attached here
                self.emit(Instr::abx(Op::GetProp, dst, str_idx));
                let last = self.proto.code.len() - 1;
                self.proto.code[last].0 |= (obj_r as u32) << 24;
                self.regs.free_temp(t);
                dst
            }

            Expr::Index { object, index, .. } => {
                let t_obj = self.regs.alloc_temp();
                let t_idx = self.regs.alloc_temp();
                let obj_r = self.compile_expr(object, t_obj);
                let idx_r = self.compile_expr(index, t_idx);
                self.emit(Instr::abc(Op::GetIndex, dst, obj_r, idx_r));
                self.regs.free_temp(t_idx);
                self.regs.free_temp(t_obj);
                dst
            }

            Expr::New { class_name, arguments, .. } => {
                let class_idx = self.globals.intern(class_name);
                self.emit(Instr::abx(Op::NewObj, dst, class_idx));
                // Compile constructor args into temps and call "init"
                let t_meth = self.regs.alloc_temp();
                let str_idx = self.proto.add_string("init");
                self.emit(Instr::abx(Op::GetMethod, t_meth, str_idx));
                let last = self.proto.code.len() - 1;
                self.proto.code[last].0 |= (dst as u32) << 24;

                let argc = arguments.len() as u8;
                let mut arg_regs = Vec::new();
                for arg in arguments {
                    let t = self.regs.alloc_temp();
                    let r = self.compile_expr(arg, t);
                    arg_regs.push(r);
                }

                let t_ret = self.regs.alloc_temp();
                self.emit(Instr::abc(Op::Call, t_ret, t_meth, argc));

                for r in arg_regs.into_iter().rev() { self.regs.free_temp(r); }
                self.regs.free_temp(t_ret);
                self.regs.free_temp(t_meth);
                dst
            }

            Expr::List { items, .. } => {
                let count = items.len();
                let base = self.regs.alloc_temp();
                let start = base;

                // Alloc consecutive regs for all items
                let mut item_regs = vec![base];
                for _ in 1..count {
                    item_regs.push(self.regs.alloc_temp());
                }

                for (i, item) in items.iter().enumerate() {
                    self.compile_expr(item, item_regs[i]);
                }

                // Use abc(dst, base, count): b=base reg, c=count — no post-emit patching needed.
                // Both base and count fit in u8 (max 255 regs / 255-element literal list).
                self.emit(Instr::abc(Op::NewList, dst, start, count as u8));

                for r in item_regs.into_iter().skip(1).rev() {
                    self.regs.free_temp(r);
                }
                self.regs.free_temp(base);
                dst
            }

            Expr::InterpolatedString { parts, .. } => {
                if parts.is_empty() {
                    let idx = self.proto.add_string("");
                    self.emit(Instr::abx(Op::LoadStr, dst, idx));
                    return dst;
                }

                // Compile each part into temps, concat them
                let mut prev = dst;
                let mut first = true;

                for part in parts {
                    let t = self.regs.alloc_temp();
                    match part {
                        StringPart::Literal(s) => {
                            let idx = self.proto.add_string(s.clone());
                            self.emit(Instr::abx(Op::LoadStr, t, idx));
                        }
                        StringPart::Expr(e) => {
                            self.compile_expr(e, t);
                        }
                    }
                    if first {
                        if t != dst { self.emit(Instr::abc(Op::Move, dst, t, 0)); }
                        first = false;
                    } else {
                        self.emit(Instr::abc(Op::Concat, dst, prev, t));
                    }
                    prev = dst;
                    self.regs.free_temp(t);
                }
                dst
            }

            Expr::Lambda { params, body, .. } => {
                // Compile lambda as nested proto
                let mut lambda_compiler = Compiler::new(
                    format!("{}.lambda", self.proto.source),
                    self.globals,
                );
                for p in params {
                    lambda_compiler.regs.alloc_local(p);
                }
                for stmt in body {
                    lambda_compiler.compile_stmt(stmt);
                }
                // Ensure return
                let last = lambda_compiler.proto.code.last().map(|i| i.op());
                if !matches!(last, Some(Op::Return) | Some(Op::ReturnNil) | Some(Op::NilReturn)) {
                    lambda_compiler.emit(Instr::abc(Op::ReturnNil, 0, 0, 0));
                }
                lambda_compiler.proto.reg_count = lambda_compiler.regs.reg_count();
                lambda_compiler.proto.param_count = params.len() as u8;

                let proto_idx = self.proto.protos.len() as u16;
                self.proto.protos.push(lambda_compiler.proto);
                self.emit(Instr::abx(Op::Closure, dst, proto_idx));
                dst
            }

            // Fallback
            _ => {
                self.emit(Instr::abc(Op::LoadNil, dst, 0, 0));
                dst
            }
        }
    }

    fn compile_and(&mut self, left: &Expr, right: &Expr, dst: u8) -> u8 {
        let l = self.compile_expr(left, dst);
        let patch = self.proto.emit_jump(Op::JumpFalse, l, self.current_line);
        self.compile_expr(right, dst);
        self.proto.patch_jump(patch);
        dst
    }

    fn compile_or(&mut self, left: &Expr, right: &Expr, dst: u8) -> u8 {
        let l = self.compile_expr(left, dst);
        let patch = self.proto.emit_jump(Op::JumpTrue, l, self.current_line);
        self.compile_expr(right, dst);
        self.proto.patch_jump(patch);
        dst
    }

    // -----------------------------------------------------------------------
    // Statement compilation
    // -----------------------------------------------------------------------

    pub fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let reg = self.regs.alloc_local(name);
                self.compile_expr(value, reg);
            }

            Stmt::Expr(e) => {
                let t = self.regs.alloc_temp();
                self.compile_expr(e, t);
                self.regs.free_temp(t);
            }

            Stmt::Out { arguments, .. } => {
                // Compile each arg, then call the built-in "out" global
                let t_fn = self.regs.alloc_temp();
                self.emit_load_global(t_fn, "out");

                let argc = arguments.len() as u8;
                let mut arg_regs = Vec::new();
                for arg in arguments {
                    let t = self.regs.alloc_temp();
                    let r = self.compile_expr(arg, t);
                    arg_regs.push(r);
                }
                let t_ret = self.regs.alloc_temp();
                self.emit(Instr::abc(Op::Call, t_ret, t_fn, argc));
                self.regs.free_temp(t_ret);
                for r in arg_regs.into_iter().rev() { self.regs.free_temp(r); }
                self.regs.free_temp(t_fn);
            }

            Stmt::Return { value, .. } => {
                match value {
                    Some(e) => {
                        let t = self.regs.alloc_temp();
                        let r = self.compile_expr(e, t);
                        self.emit(Instr::abc(Op::Return, r, 0, 0));
                        self.regs.free_temp(t);
                    }
                    None => {
                        self.emit(Instr::abc(Op::ReturnNil, 0, 0, 0));
                    }
                }
            }

            Stmt::If { condition, then_body, else_body, .. } => {
                let t = self.regs.alloc_temp();
                self.compile_expr(condition, t);
                let false_jump = self.proto.emit_jump(Op::JumpFalse, t, self.current_line);
                self.regs.free_temp(t);

                let scope = self.regs.push_scope();
                for s in then_body { self.compile_stmt(s); }
                self.regs.pop_scope(scope);

                if let Some(else_stmts) = else_body {
                    let end_jump = self.proto.emit_jump(Op::Jump, 0, self.current_line);
                    self.proto.patch_jump(false_jump);
                    let scope = self.regs.push_scope();
                    for s in else_stmts { self.compile_stmt(s); }
                    self.regs.pop_scope(scope);
                    self.proto.patch_jump(end_jump);
                } else {
                    self.proto.patch_jump(false_jump);
                }
            }

            Stmt::While { condition, body, .. } => {
                let loop_start = self.proto.code.len();
                self.loop_starts.push(loop_start);
                self.break_patches.push(Vec::new());

                let t = self.regs.alloc_temp();
                self.compile_expr(condition, t);
                let exit_jump = self.proto.emit_jump(Op::JumpFalse, t, self.current_line);
                self.regs.free_temp(t);

                let scope = self.regs.push_scope();
                for s in body { self.compile_stmt(s); }
                self.regs.pop_scope(scope);

                // LoopBack (profiling back-edge)
                let offset = loop_start as i32 - self.proto.code.len() as i32 - 1;
                self.emit(Instr::asbx(Op::LoopBack, 0, offset as i16));
                self.proto.patch_jump(exit_jump);

                // Patch break jumps
                let breaks = self.break_patches.pop().unwrap_or_default();
                for b in breaks { self.proto.patch_jump(b); }
                self.loop_starts.pop();
            }

            Stmt::For { var, iterable, body, .. } => {
                // Compile: for v in list { body }
                // Desugars to: let __iter = iterable; let __i = 0; while __i < len(__iter) { let v = __iter[__i]; body; __i++ }
                let t_iter = self.regs.alloc_temp();
                self.compile_expr(iterable, t_iter);

                let t_len = self.regs.alloc_temp();
                self.emit(Instr::abc(Op::ListLen, t_len, t_iter, 0));

                let t_i = self.regs.alloc_temp();
                self.emit(Instr::asbx(Op::LoadInt, t_i, 0));

                let loop_start = self.proto.code.len();
                self.loop_starts.push(loop_start);
                self.break_patches.push(Vec::new());

                // Condition: i < len
                let t_cond = self.regs.alloc_temp();
                self.emit(Instr::abc(Op::Lt, t_cond, t_i, t_len));
                let exit_jump = self.proto.emit_jump(Op::JumpFalse, t_cond, self.current_line);
                self.regs.free_temp(t_cond);

                // let v = iter[i]
                let v_reg = self.regs.alloc_local(var);
                self.emit(Instr::abc(Op::GetIndex, v_reg, t_iter, t_i));

                let scope = self.regs.push_scope();
                for s in body { self.compile_stmt(s); }
                self.regs.pop_scope(scope);

                // i++
                self.emit(Instr::abc(Op::IncrLocal, t_i, 0, 0));

                let offset = loop_start as i32 - self.proto.code.len() as i32 - 1;
                self.emit(Instr::asbx(Op::LoopBack, 0, offset as i16));
                self.proto.patch_jump(exit_jump);

                let breaks = self.break_patches.pop().unwrap_or_default();
                for b in breaks { self.proto.patch_jump(b); }
                self.loop_starts.pop();

                self.regs.free_temp(t_i);
                self.regs.free_temp(t_len);
                self.regs.free_temp(t_iter);
            }

            Stmt::Match { expr, arms, .. } => {
                let t_val = self.regs.alloc_temp();
                self.compile_expr(expr, t_val);

                let mut end_patches = Vec::new();

                for arm in arms {
                    let t_cond = self.regs.alloc_temp();
                    match &arm.pattern {
                        MatchPattern::Wildcard | MatchPattern::Identifier(_) => {
                            // Always matches
                            self.emit(Instr::abc(Op::LoadTrue, t_cond, 0, 0));
                        }
                        MatchPattern::Literal(e) => {
                            let t_lit = self.regs.alloc_temp();
                            self.compile_expr(e, t_lit);
                            self.emit(Instr::abc(Op::Eq, t_cond, t_val, t_lit));
                            self.regs.free_temp(t_lit);
                        }
                        MatchPattern::EnumVariant { enum_name, variant, .. } => {
                            let t_expect = self.regs.alloc_temp();
                            let key = match enum_name {
                                Some(e) => format!("{}.{}", e, variant),
                                None => variant.clone(),
                            };
                            let idx = self.proto.add_string(key);
                            self.emit(Instr::abx(Op::LoadStr, t_expect, idx));
                            self.emit(Instr::abc(Op::Eq, t_cond, t_val, t_expect));
                            self.regs.free_temp(t_expect);
                        }
                    }

                    let skip_jump = self.proto.emit_jump(Op::JumpFalse, t_cond, self.current_line);
                    self.regs.free_temp(t_cond);

                    let scope = self.regs.push_scope();
                    for s in &arm.body { self.compile_stmt(s); }
                    self.regs.pop_scope(scope);

                    let end_j = self.proto.emit_jump(Op::Jump, 0, self.current_line);
                    end_patches.push(end_j);
                    self.proto.patch_jump(skip_jump);
                }

                for ep in end_patches { self.proto.patch_jump(ep); }
                self.regs.free_temp(t_val);
            }

            Stmt::Block(stmts) => {
                let scope = self.regs.push_scope();
                for s in stmts { self.compile_stmt(s); }
                self.regs.pop_scope(scope);
            }

            Stmt::GoSpawn { body, .. } => {
                // Compile body as lambda, then call via goroutine mechanism
                // For now: just compile as block (goroutine scheduling handled by VM)
                let scope = self.regs.push_scope();
                for s in body { self.compile_stmt(s); }
                self.regs.pop_scope(scope);
            }
        }
    }

    pub fn finalize(mut self) -> Proto {
        // Ensure we end with a return
        let last_op = self.proto.code.last().map(|i| i.op());
        if !matches!(last_op, Some(Op::Return) | Some(Op::ReturnNil) | Some(Op::NilReturn) | Some(Op::Halt)) {
            self.emit(Instr::abc(Op::ReturnNil, 0, 0, 0));
        }
        self.proto.reg_count = self.regs.reg_count().max(8); // at least 8 regs
        self.proto
    }
}

// ---------------------------------------------------------------------------
// Top-level compile function
// ---------------------------------------------------------------------------

pub fn compile_program(items: &[Item], source: &str) -> (Proto, GlobalTable) {
    let mut globals = GlobalTable::new();

    // Pre-intern standard globals so the VM can index them by u16
    for name in &["out", "print", "in", "str", "int", "bol", "type", "nil",
                   "sqrt", "abs", "floor", "ceil", "pow", "min", "max", "avg",
                   "alg", "ann", "aut", "clr", "col", "con", "csv", "dfm",
                   "env", "git", "ioo", "jsn", "log", "mth", "net", "num",
                   "plt", "pth", "sys", "tim", "tui", "cli",
                   "chdir", "cwd", "__load"] {
        globals.intern(name);
    }

    // Pre-intern all user-declared names so we can reference them without a
    // second live borrow on globals later.
    for item in items {
        match item {
            Item::FunctionDecl { name, .. } => { globals.intern(name); }
            Item::ClassDecl    { name, .. } => { globals.intern(name); }
            _ => {}
        }
    }

    // ── Pass 1: compile every function body into its own Proto ───────────────
    // Each fn_compiler borrows globals exclusively, then is dropped before the
    // next one is created — this avoids the double-borrow compile error.
    let mut fn_protos: Vec<(String, Proto)> = Vec::new();

    for item in items {
        if let Item::FunctionDecl { name, params, body, .. } = item {
            let compiled_proto = {
                let mut fn_compiler = Compiler::new(
                    format!("{}:{}", source, name),
                    &mut globals,
                );
                for p in params {
                    fn_compiler.regs.alloc_local(p);
                }
                for stmt in body {
                    fn_compiler.compile_stmt(stmt);
                }
                let last = fn_compiler.proto.code.last().map(|i| i.op());
                if !matches!(last, Some(Op::Return) | Some(Op::ReturnNil) | Some(Op::NilReturn)) {
                    fn_compiler.emit(Instr::abc(Op::ReturnNil, 0, 0, 0));
                }
                fn_compiler.proto.reg_count = fn_compiler.regs.reg_count();
                fn_compiler.proto.param_count = params.len() as u8;
                fn_compiler.proto
            }; // fn_compiler dropped here — globals borrow released
            fn_protos.push((name.clone(), compiled_proto));
        }
    }

    // ── Pass 2: build the top-level Proto ────────────────────────────────────
    // All fn_compilers are gone; we can now hold the single main compiler.
    let mut compiler = Compiler::new(source, &mut globals);

    // Hoist compiled function closures into globals
    for (name, proto) in fn_protos {
        let proto_idx = compiler.proto.protos.len() as u16;
        compiler.proto.protos.push(proto);
        let t = compiler.regs.alloc_temp();
        compiler.emit(Instr::abx(Op::Closure, t, proto_idx));
        compiler.emit_store_global(t, &name);
        compiler.regs.free_temp(t);
    }

    // Hoist class placeholders (class bodies executed by the runtime)
    for item in items {
        if let Item::ClassDecl { name, .. } = item {
            let class_idx = compiler.globals.intern(name);
            let t = compiler.regs.alloc_temp();
            compiler.emit(Instr::abx(Op::LoadGlobal, t, class_idx));
            compiler.regs.free_temp(t);
        }
    }

    // Compile top-level statements
    for item in items {
        match item {
            Item::Statement(stmt) => { compiler.compile_stmt(stmt); }
            Item::LoadStmt { path, alias, .. } => {
                let t_fn = compiler.regs.alloc_temp();
                compiler.emit_load_global(t_fn, "__load");
                let t_path = compiler.regs.alloc_temp();
                let idx = compiler.proto.add_string(path.clone());
                compiler.emit(Instr::abx(Op::LoadStr, t_path, idx));
                let t_ret = compiler.regs.alloc_temp();
                compiler.emit(Instr::abc(Op::Call, t_ret, t_fn, 1));
                if let Some(alias_name) = alias {
                    compiler.emit_store_global(t_ret, alias_name);
                }
                let full_key = path.trim_start_matches('@').replace('/', ".").replace('-', "_");
                compiler.emit_store_global(t_ret, &full_key);
                compiler.regs.free_temp(t_ret);
                compiler.regs.free_temp(t_path);
                compiler.regs.free_temp(t_fn);
            }
            _ => {}
        }
    }

    let proto = compiler.finalize();
    (proto, globals)
}
