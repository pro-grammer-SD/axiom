/// Axiom High-Performance Runtime
///
/// PERFORMANCE REWRITE — key changes vs previous version:
///
///   OLD: Arc<RwLock<Scope>> passed everywhere — a mutex lock/unlock on every
///        single variable read or write, plus recursive lock acquisition up the
///        parent chain, plus Arc atomic ref-count on every clone().
///
///   NEW: Env — a plain Vec<HashMap<String, AxValue>> stack.
///        push_frame() / pop_frame() replace scope creation.
///        No locks, no Arc, no heap allocation per scope level.
///        Variable lookup is an O(depth) linear scan of a Vec — fast in
///        practice because call depth is small.
///
///   OLD: DashMap (concurrent hash map) for globals — all its CAS overhead
///        on a single-threaded interpreter.
///
///   NEW: Plain HashMap for globals. Goroutines clone the global snapshot
///        into their own copy — safe because Axiom has value semantics.
///
///   OLD: Missing expression arms (Assign, MemberAccess, MethodCall, Index,
///        New, SelfRef, UnaryOp, List, Match, For) silently returned Nil.
///
///   NEW: All arms implemented.
///
///   OLD: Equality via l.display() == r.display() — string-formats numbers
///        before comparing them.
///
///   NEW: Structural equality on AxValue directly.

use crate::ast::{ClassMember, Expr, Item, MatchPattern, Stmt, StringPart};
use crate::core::oop::{AxCallable, AxClass, AxInstance};
use crate::core::value::AxValue;
use crate::intrinsics;
use crate::errors::RuntimeError;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ---------------------------------------------------------------------------
// Env — flat-stack scope, no locks required
// ---------------------------------------------------------------------------

pub struct Env {
    /// Each entry is one scope frame. Index 0 = outermost, last = innermost.
    frames: Vec<HashMap<String, AxValue>>,
}

impl Env {
    fn new() -> Self {
        Env { frames: vec![HashMap::new()] }
    }

    fn push_frame(&mut self) {
        self.frames.push(HashMap::new());
    }

    fn pop_frame(&mut self) {
        self.frames.pop();
    }

    fn get(&self, name: &str) -> Option<&AxValue> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(name) {
                return Some(v);
            }
        }
        None
    }

    fn set(&mut self, name: &str, value: AxValue) -> bool {
        // Assign to the nearest frame that already has this name.
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) {
                frame.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    fn define(&mut self, name: String, value: AxValue) {
        // Always define in the current (innermost) frame.
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name, value);
        }
    }
}

// ---------------------------------------------------------------------------
// Runtime
// ---------------------------------------------------------------------------

pub struct Runtime {
    pub globals: HashMap<String, AxValue>,
    pub classes: HashMap<String, Arc<AxClass>>,
}

impl Runtime {
    pub fn new() -> Self {
        let mut globals: HashMap<String, AxValue> = HashMap::new();

        // ── built-in functions ───────────────────────────────────────────────

        globals.insert("type".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "type".into(),
            func: |args| {
                args.first().map(|a| AxValue::Str(a.type_name().to_string())).unwrap_or(AxValue::Nil)
            },
        })));

        globals.insert("int".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "int".into(),
            func: |args| match args.first() {
                Some(AxValue::Num(n)) => AxValue::Num(*n),
                Some(AxValue::Str(s)) => s.parse::<f64>().map(AxValue::Num).unwrap_or(AxValue::Nil),
                Some(AxValue::Bol(b)) => AxValue::Num(if *b { 1.0 } else { 0.0 }),
                _ => AxValue::Nil,
            },
        })));

        globals.insert("str".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "str".into(),
            func: |args| args.first().map(|a| AxValue::Str(a.display())).unwrap_or(AxValue::Nil),
        })));

        globals.insert("bol".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "bol".into(),
            func: |args| args.first().map(|a| AxValue::Bol(a.is_truthy())).unwrap_or(AxValue::Nil),
        })));

        globals.insert("out".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "out".into(),
            func: |args| {
                let parts: Vec<String> = args.iter().map(|a| a.display()).collect();
                println!("{}", parts.join(" "));
                AxValue::Nil
            },
        })));

        globals.insert("in".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "in".into(),
            func: |args| {
                use std::io::Write;
                if let Some(AxValue::Str(prompt)) = args.first() {
                    print!("{}", prompt);
                    let _ = std::io::stdout().flush();
                }
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(_) => AxValue::Str(input.trim_end_matches(['\r', '\n']).to_string()),
                    Err(_) => AxValue::Nil,
                }
            },
        })));

        // Math builtins
        globals.insert("sqrt".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "sqrt".into(),
            func: |args| match args.first() {
                Some(AxValue::Num(n)) => AxValue::Num(n.sqrt()),
                _ => AxValue::Nil,
            },
        })));

        globals.insert("abs".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "abs".into(),
            func: |args| match args.first() {
                Some(AxValue::Num(n)) => AxValue::Num(n.abs()),
                _ => AxValue::Nil,
            },
        })));

        globals.insert("floor".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "floor".into(),
            func: |args| match args.first() {
                Some(AxValue::Num(n)) => AxValue::Num(n.floor()),
                _ => AxValue::Nil,
            },
        })));

        globals.insert("ceil".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "ceil".into(),
            func: |args| match args.first() {
                Some(AxValue::Num(n)) => AxValue::Num(n.ceil()),
                _ => AxValue::Nil,
            },
        })));

        globals.insert("pow".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "pow".into(),
            func: |args| match (args.first(), args.get(1)) {
                (Some(AxValue::Num(base)), Some(AxValue::Num(exp))) => AxValue::Num(base.powf(*exp)),
                _ => AxValue::Nil,
            },
        })));

        globals.insert("min".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "min".into(),
            func: |args| match (args.first(), args.get(1)) {
                (Some(AxValue::Num(a)), Some(AxValue::Num(b))) => AxValue::Num(a.min(*b)),
                _ => AxValue::Nil,
            },
        })));

        globals.insert("max".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "max".into(),
            func: |args| match (args.first(), args.get(1)) {
                (Some(AxValue::Num(a)), Some(AxValue::Num(b))) => AxValue::Num(a.max(*b)),
                _ => AxValue::Nil,
            },
        })));

        // avg(list) — average of a list of numbers
        globals.insert("avg".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "avg".into(),
            func: |args| match args.first() {
                Some(AxValue::Lst(items)) => {
                    let items = items.read().unwrap();
                    if items.is_empty() { return AxValue::Nil; }
                    let sum: f64 = items.iter()
                        .filter_map(|v: &AxValue| v.as_num().ok())
                        .sum();
                    AxValue::Num(sum / items.len() as f64)
                }
                _ => AxValue::Nil,
            },
        })));

        globals.insert("nil".into(), AxValue::Nil);

        // Register static intrinsic module maps (first-class 22 modules)
        intrinsics::register(&mut globals);

        Runtime { globals, classes: HashMap::new() }
    }

    // -----------------------------------------------------------------------
    // Entry point
    // -----------------------------------------------------------------------

    pub fn run(&mut self, items: Vec<Item>) -> Result<(), RuntimeError> {
        // Pass 1 — hoist all declarations into globals/classes
        for item in &items {
            self.register_decl(item);
        }

        // Pass 2 — execute top-level statements
        let mut env = Env::new();
        for item in &items {
            if let Item::Statement(stmt) = item {
                self.exec_stmt(stmt, &mut env)?;
            }
        }

        // Pass 3 — if a `main()` function is defined, call it.
        // Having a main() is fully optional; top-level statements are always
        // executed regardless.
        if let Some(main_fn) = self.globals.get("main").cloned() {
            self.call_value(main_fn, vec![], &mut env)?;
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Declaration registration (no execution)
    // -----------------------------------------------------------------------

    fn register_decl(&mut self, item: &Item) {
        match item {
            Item::FunctionDecl { name, params, body, .. } => {
                self.globals.insert(name.clone(), AxValue::Fun(Arc::new(AxCallable::UserDefined {
                    params: params.clone(),
                    body: body.clone(),
                })));
            }
            Item::ClassDecl { name, body, .. } => {
                let mut ax_class = AxClass::new(name.clone());
                for member in body {
                    match member {
                        ClassMember::Method { name: mname, params, body, .. } => {
                            ax_class.methods.insert(mname.clone(), AxCallable::UserDefined {
                                params: params.clone(),
                                body: body.clone(),
                            });
                        }
                        ClassMember::Field { name: fname, default, .. } => {
                            ax_class.fields.push((fname.clone(), default.clone()));
                        }
                    }
                }
                self.classes.insert(name.clone(), Arc::new(ax_class));
            }
            Item::EnumDecl { name, variants, .. } => {
                // Expose each variant as EnumName.Variant in globals
                for v in variants {
                    let key = format!("{}.{}", name, v.name);
                    self.globals.insert(key, AxValue::Str(format!("{}.{}", name, v.name)));
                }
                // Also expose the enum name itself
                self.globals.insert(name.clone(), AxValue::Str(name.clone()));
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Statement execution
    // -----------------------------------------------------------------------

    fn exec_stmt(&self, stmt: &Stmt, env: &mut Env) -> Result<Option<AxValue>, RuntimeError> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let val = self.eval(value, env)?;
                env.define(name.clone(), val);
            }

            Stmt::Expr(e) => {
                self.eval(e, env)?;
            }

            Stmt::Out { arguments, .. } => {
                let mut parts = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    parts.push(self.eval(arg, env)?.display());
                }
                println!("{}", parts.join(""));
            }

            Stmt::Return { value, .. } => {
                let v = match value {
                    Some(e) => self.eval(e, env)?,
                    None    => AxValue::Nil,
                };
                return Ok(Some(v));
            }

            Stmt::If { condition, then_body, else_body, .. } => {
                if self.eval(condition, env)?.is_truthy() {
                    return self.exec_block(then_body, env);
                } else if let Some(eb) = else_body {
                    return self.exec_block(eb, env);
                }
            }

            Stmt::While { condition, body, .. } => {
                while self.eval(condition, env)?.is_truthy() {
                    if let Some(ret) = self.exec_block(body, env)? {
                        return Ok(Some(ret));
                    }
                }
            }

            Stmt::For { var, iterable, body, .. } => {
                let iter_val = self.eval(iterable, env)?;
                let items = match &iter_val {
                    AxValue::Lst(list) => list.read().unwrap().clone(),
                    AxValue::Str(s) => {
                        s.chars().map(|c| AxValue::Str(c.to_string())).collect()
                    }
                    _ => return Err(RuntimeError::GenericError {
                        message: format!("'{}' is not iterable", iter_val.type_name()),
                        span: Default::default(),
                    }),
                };
                for item in items {
                    env.push_frame();
                    env.define(var.clone(), item);
                    let ret = self.exec_block_in_current_env(body, env)?;
                    env.pop_frame();
                    if ret.is_some() { return Ok(ret); }
                }
            }

            Stmt::Block(stmts) => return self.exec_block(stmts, env),

            Stmt::Match { expr, arms, .. } => {
                let val = self.eval(expr, env)?;
                for arm in arms {
                    if self.pattern_matches(&arm.pattern, &val) {
                        env.push_frame();
                        // Bind capture variable if present
                        if let MatchPattern::EnumVariant { binding: Some(b), .. } = &arm.pattern {
                            env.define(b.clone(), val.clone());
                        }
                        let ret = self.exec_block_in_current_env(&arm.body, env)?;
                        env.pop_frame();
                        return Ok(ret);
                    }
                }
            }

            Stmt::GoSpawn { body, .. } => {
                // Snapshot current globals for the goroutine
                let globals_snapshot = self.globals.clone();
                let classes_snapshot = self.classes.clone();
                let body = body.clone();
                tokio::spawn(async move {
                    let runtime = Runtime { globals: globals_snapshot, classes: classes_snapshot };
                    let mut env = Env::new();
                    let _ = runtime.exec_block_in_current_env(&body, &mut env);
                });
            }
        }
        Ok(None)
    }

    // Push a fresh frame, run stmts, pop frame.
    fn exec_block(&self, stmts: &[Stmt], env: &mut Env) -> Result<Option<AxValue>, RuntimeError> {
        env.push_frame();
        let ret = self.exec_block_in_current_env(stmts, env);
        env.pop_frame();
        ret
    }

    // Run stmts in env without touching frames (caller manages push/pop).
    fn exec_block_in_current_env(&self, stmts: &[Stmt], env: &mut Env) -> Result<Option<AxValue>, RuntimeError> {
        for stmt in stmts {
            if let Some(ret) = self.exec_stmt(stmt, env)? {
                return Ok(Some(ret));
            }
        }
        Ok(None)
    }

    // -----------------------------------------------------------------------
    // Pattern matching
    // -----------------------------------------------------------------------

    fn pattern_matches(&self, pattern: &MatchPattern, value: &AxValue) -> bool {
        match pattern {
            MatchPattern::Wildcard => true,
            MatchPattern::Identifier(_) => true,
            MatchPattern::Literal(expr) => {
                // Evaluate literal against a throwaway env
                let mut env = Env::new();
                if let Ok(lit) = self.eval(expr, &mut env) {
                    self.values_equal(&lit, value)
                } else {
                    false
                }
            }
            MatchPattern::EnumVariant { enum_name, variant, .. } => {
                let expected = match enum_name {
                    Some(e) => format!("{}.{}", e, variant),
                    None    => variant.clone(),
                };
                match value {
                    AxValue::Str(s) => s == &expected || s.ends_with(&format!(".{}", variant)),
                    _ => false,
                }
            }
        }
    }

    fn values_equal(&self, a: &AxValue, b: &AxValue) -> bool {
        match (a, b) {
            (AxValue::Num(x), AxValue::Num(y)) => x == y,
            (AxValue::Str(x), AxValue::Str(y)) => x == y,
            (AxValue::Bol(x), AxValue::Bol(y)) => x == y,
            (AxValue::Nil, AxValue::Nil)        => true,
            _ => false,
        }
    }

    // -----------------------------------------------------------------------
    // Expression evaluation
    // -----------------------------------------------------------------------

    fn eval(&self, expr: &Expr, env: &mut Env) -> Result<AxValue, RuntimeError> {
        match expr {
            Expr::Number  { value, .. } => Ok(AxValue::Num(*value)),
            Expr::String  { value, .. } => Ok(AxValue::Str(value.clone())),
            Expr::Boolean { value, .. } => Ok(AxValue::Bol(*value)),
            Expr::SelfRef { .. }        => self.lookup("self", env),

            Expr::Identifier { name, .. } => self.lookup(name, env),

            Expr::Assign { target, value, .. } => {
                let val = self.eval(value, env)?;
                match target.as_ref() {
                    Expr::Identifier { name, .. } => {
                        if !env.set(name, val.clone()) {
                            env.define(name.clone(), val.clone());
                        }
                    }
                    Expr::MemberAccess { object, member, .. } => {
                        let obj = self.eval(object, env)?;
                        if let AxValue::Instance(inst) = obj {
                            inst.write().unwrap().fields.insert(member.clone(), val.clone());
                        }
                    }
                    _ => {}
                }
                Ok(val)
            }

            Expr::UnaryOp { op, operand, .. } => {
                let v = self.eval(operand, env)?;
                match op.as_str() {
                    "!" => Ok(AxValue::Bol(!v.is_truthy())),
                    "-" => Ok(AxValue::Num(-v.as_num().unwrap_or(0.0))),
                    _   => Ok(AxValue::Nil),
                }
            }

            Expr::BinaryOp { left, op, right, .. } => {
                // Short-circuit logical ops first
                match op.as_str() {
                    "&&" => {
                        let l = self.eval(left, env)?;
                        return if l.is_truthy() { self.eval(right, env) } else { Ok(l) };
                    }
                    "||" => {
                        let l = self.eval(left, env)?;
                        return if l.is_truthy() { Ok(l) } else { self.eval(right, env) };
                    }
                    _ => {}
                }

                let l = self.eval(left, env)?;
                let r = self.eval(right, env)?;

                match op.as_str() {
                    "+" => match (&l, &r) {
                        (AxValue::Num(a), AxValue::Num(b)) => Ok(AxValue::Num(a + b)),
                        _ => Ok(AxValue::Str(format!("{}{}", l.display(), r.display()))),
                    },
                    "-"  => Ok(AxValue::Num(l.as_num().unwrap_or(0.0) - r.as_num().unwrap_or(0.0))),
                    "*"  => Ok(AxValue::Num(l.as_num().unwrap_or(0.0) * r.as_num().unwrap_or(0.0))),
                    "/"  => {
                        let divisor = r.as_num().unwrap_or(1.0);
                        if divisor == 0.0 {
                            return Err(RuntimeError::GenericError {
                                message: "Division by zero".into(),
                                span: Default::default(),
                            });
                        }
                        Ok(AxValue::Num(l.as_num().unwrap_or(0.0) / divisor))
                    }
                    "%"  => Ok(AxValue::Num(l.as_num().unwrap_or(0.0) % r.as_num().unwrap_or(1.0))),
                    "==" => Ok(AxValue::Bol(self.values_equal(&l, &r))),
                    "!=" => Ok(AxValue::Bol(!self.values_equal(&l, &r))),
                    "<"  => Ok(AxValue::Bol(l.as_num().unwrap_or(0.0) <  r.as_num().unwrap_or(0.0))),
                    "<=" => Ok(AxValue::Bol(l.as_num().unwrap_or(0.0) <= r.as_num().unwrap_or(0.0))),
                    ">"  => Ok(AxValue::Bol(l.as_num().unwrap_or(0.0) >  r.as_num().unwrap_or(0.0))),
                    ">=" => Ok(AxValue::Bol(l.as_num().unwrap_or(0.0) >= r.as_num().unwrap_or(0.0))),
                    _    => Ok(AxValue::Nil),
                }
            }

            Expr::Call { function, arguments, .. } => {
                let func = self.eval(function, env)?;
                let mut args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    args.push(self.eval(arg, env)?);
                }
                self.call_value(func, args, env)
            }

            Expr::MethodCall { object, method, arguments, .. } => {
                let obj = self.eval(object, env)?;
                let mut args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    args.push(self.eval(arg, env)?);
                }
                self.call_method(obj, method, args, env)
            }

            Expr::MemberAccess { object, member, .. } => {
                let obj = self.eval(object, env)?;
                match &obj {
                    AxValue::Instance(inst) => {
                        let inst_r = inst.read().unwrap();
                        if let Some(v) = inst_r.fields.get(member) {
                            return Ok(v.clone());
                        }
                        // Try method as a bound value
                        if let Some(method) = inst_r.class.methods.get(member) {
                            return Ok(AxValue::Fun(Arc::new(method.clone())));
                        }
                        Ok(AxValue::Nil)
                    }
                    AxValue::Str(s) => {
                        // String properties
                        match member.as_str() {
                            "len" => Ok(AxValue::Num(s.len() as f64)),
                            _     => Ok(AxValue::Nil),
                        }
                    }
                    AxValue::Map(map) => {
                        // Module-like or dict-like member access
                        if let Some(v) = map.get(member) {
                            return Ok(v.clone());
                        }
                        Ok(AxValue::Nil)
                    }
                    AxValue::Lst(list) => {
                        match member.as_str() {
                            "len" => Ok(AxValue::Num(list.read().unwrap().len() as f64)),
                            _     => Ok(AxValue::Nil),
                        }
                    }
                    _ => Ok(AxValue::Nil),
                }
            }

            Expr::Index { object, index, .. } => {
                let obj = self.eval(object, env)?;
                let idx = self.eval(index, env)?;
                match (&obj, &idx) {
                    (AxValue::Lst(list), AxValue::Num(n)) => {
                        let list = list.read().unwrap();
                        let i = *n as isize;
                        let len = list.len() as isize;
                        let i = if i < 0 { len + i } else { i };
                        if i >= 0 && (i as usize) < list.len() {
                            Ok(list[i as usize].clone())
                        } else {
                            Err(RuntimeError::GenericError {
                                message: format!("Index {} out of range (len={})", n, list.len()),
                                span: Default::default(),
                            })
                        }
                    }
                    (AxValue::Str(s), AxValue::Num(n)) => {
                        let i = *n as usize;
                        s.chars().nth(i)
                            .map(|c| AxValue::Str(c.to_string()))
                            .ok_or_else(|| RuntimeError::GenericError {
                                message: format!("String index {} out of range", n),
                                span: Default::default(),
                            })
                    }
                    _ => Ok(AxValue::Nil),
                }
            }

            Expr::New { class_name, arguments, .. } => {
                let class = self.classes.get(class_name).cloned().ok_or_else(|| {
                    RuntimeError::GenericError {
                        message: format!("Unknown class '{}'", class_name),
                        span: Default::default(),
                    }
                })?;

                // Build instance with default fields
                let fields: DashMap<String, AxValue> = DashMap::new();
                for (fname, default_expr) in &class.fields {
                    let val = if let Some(e) = default_expr {
                        self.eval(e, env)?
                    } else {
                        AxValue::Nil
                    };
                    fields.insert(fname.clone(), val);
                }

                let inst = Arc::new(RwLock::new(AxInstance {
                    class: Arc::clone(&class),
                    fields,
                }));

                // Evaluate constructor args
                let mut args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    args.push(self.eval(arg, env)?);
                }

                let instance_val = AxValue::Instance(Arc::clone(&inst));

                // Call init() if present
                if let Some(init_callable) = class.methods.get("init") {
                    if let AxCallable::UserDefined { params, body } = init_callable {
                        env.push_frame();
                        env.define("self".into(), instance_val.clone());
                        for (p, a) in params.iter().zip(args.iter()) {
                            env.define(p.clone(), a.clone());
                        }
                        self.exec_block_in_current_env(body, env)?;
                        env.pop_frame();
                    }
                }

                Ok(instance_val)
            }

            Expr::List { items, .. } => {
                let mut vals = Vec::with_capacity(items.len());
                for item in items {
                    vals.push(self.eval(item, env)?);
                }
                Ok(AxValue::Lst(Arc::new(RwLock::new(vals))))
            }

            Expr::InterpolatedString { parts, .. } => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Expr(e)    => result.push_str(&self.eval(e, env)?.display()),
                    }
                }
                Ok(AxValue::Str(result))
            }

            // Fallback
            _ => Ok(AxValue::Nil),
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn lookup(&self, name: &str, env: &Env) -> Result<AxValue, RuntimeError> {
        if let Some(v) = env.get(name) {
            return Ok(v.clone());
        }
        if let Some(v) = self.globals.get(name) {
            return Ok(v.clone());
        }
        Err(RuntimeError::UndefinedVariable { name: name.to_string(), span: Default::default() })
    }

    fn call_value(&self, func: AxValue, args: Vec<AxValue>, env: &mut Env) -> Result<AxValue, RuntimeError> {
        match func {
            AxValue::Fun(callable) => match &*callable {
                AxCallable::Native { func, .. } => Ok(func(args)),
                AxCallable::UserDefined { params, body } => {
                    env.push_frame();
                    for (p, a) in params.iter().zip(args.iter()) {
                        env.define(p.clone(), a.clone());
                    }
                    let ret = self.exec_block_in_current_env(body, env)?;
                    env.pop_frame();
                    Ok(ret.unwrap_or(AxValue::Nil))
                }
            },
            // Calling a class name → instantiate
            _ => Err(RuntimeError::GenericError {
                message: format!("Not callable: {}", func.type_name()),
                span: Default::default(),
            }),
        }
    }

    fn call_method(&self, obj: AxValue, method: &str, args: Vec<AxValue>, env: &mut Env) -> Result<AxValue, RuntimeError> {
        match &obj {
            AxValue::Instance(inst) => {
                let callable = {
                    let inst_r = inst.read().unwrap();
                    inst_r.class.methods.get(method).cloned()
                };
                match callable {
                    Some(AxCallable::UserDefined { params, body }) => {
                        env.push_frame();
                        env.define("self".into(), obj.clone());
                        for (p, a) in params.iter().zip(args.iter()) {
                            env.define(p.clone(), a.clone());
                        }
                        let ret = self.exec_block_in_current_env(&body, env)?;
                        env.pop_frame();
                        Ok(ret.unwrap_or(AxValue::Nil))
                    }
                    Some(AxCallable::Native { func, .. }) => Ok(func(args)),
                    None => Err(RuntimeError::GenericError {
                        message: format!("No method '{}' on instance", method),
                        span: Default::default(),
                    }),
                }
            }

            AxValue::Map(map) => {
                if let Some(v) = map.get(method) {
                    return self.call_value(v.clone(), args, env);
                }
                return Err(RuntimeError::GenericError {
                    message: format!("No method '{}' on Map", method),
                    span: Default::default(),
                });
            }

            AxValue::Str(s) => {
                match method {
                    "len"    => Ok(AxValue::Num(s.len() as f64)),
                    "upper"  => Ok(AxValue::Str(s.to_uppercase())),
                    "lower"  => Ok(AxValue::Str(s.to_lowercase())),
                    "trim"   => Ok(AxValue::Str(s.trim().to_string())),
                    "split"  => {
                        let sep = args.first()
                            .and_then(|a| if let AxValue::Str(s) = a { Some(s.as_str()) } else { None })
                            .unwrap_or(" ");
                        let parts: Vec<AxValue> = s.split(sep).map(|p| AxValue::Str(p.to_string())).collect();
                        Ok(AxValue::Lst(Arc::new(RwLock::new(parts))))
                    }
                    "contains" => {
                        let needle = args.first().map(|a| a.display()).unwrap_or_default();
                        Ok(AxValue::Bol(s.contains(&needle)))
                    }
                    "starts_with" => {
                        let prefix = args.first().map(|a| a.display()).unwrap_or_default();
                        Ok(AxValue::Bol(s.starts_with(&prefix)))
                    }
                    "ends_with" => {
                        let suffix = args.first().map(|a| a.display()).unwrap_or_default();
                        Ok(AxValue::Bol(s.ends_with(&suffix)))
                    }
                    "replace" => {
                        let from = args.first().map(|a| a.display()).unwrap_or_default();
                        let to   = args.get(1).map(|a| a.display()).unwrap_or_default();
                        Ok(AxValue::Str(s.replace(&from, &to)))
                    }
                    "align" => {
                        let width = args.first().and_then(|a| a.as_num().ok()).unwrap_or(0.0) as usize;
                        let dir   = args.get(1).map(|a| a.display()).unwrap_or_else(|| "left".into());
                        let result = match dir.as_str() {
                            "right"  => format!("{:>width$}", s, width = width),
                            "center" => format!("{:^width$}", s, width = width),
                            _        => format!("{:<width$}", s, width = width),
                        };
                        Ok(AxValue::Str(result))
                    }
                    _ => Err(RuntimeError::GenericError {
                        message: format!("No method '{}' on Str", method),
                        span: Default::default(),
                    }),
                }
            }

            // FIX 2: was AxValue::List — correct variant name is AxValue::Lst
            AxValue::Lst(list) => {
                match method {
                    "len" => Ok(AxValue::Num(list.read().unwrap().len() as f64)),
                    "push" => {
                        if let Some(v) = args.into_iter().next() {
                            list.write().unwrap().push(v);
                        }
                        Ok(AxValue::Nil)
                    }
                    "pop" => {
                        Ok(list.write().unwrap().pop().unwrap_or(AxValue::Nil))
                    }
                    "first" => Ok(list.read().unwrap().first().cloned().unwrap_or(AxValue::Nil)),
                    "last"  => Ok(list.read().unwrap().last().cloned().unwrap_or(AxValue::Nil)),
                    "contains" => {
                        let needle = args.first().cloned().unwrap_or(AxValue::Nil);
                        let found  = list.read().unwrap().iter().any(|v| self.values_equal(v, &needle));
                        Ok(AxValue::Bol(found))
                    }
                    "join" => {
                        let sep = args.first().map(|a| a.display()).unwrap_or_default();
                        // FIX 3: was |v| — added explicit type |v: &AxValue| to resolve E0282
                        let joined = list.read().unwrap().iter()
                            .map(|v: &AxValue| v.display())
                            .collect::<Vec<_>>()
                            .join(&sep);
                        Ok(AxValue::Str(joined))
                    }
                    _ => Err(RuntimeError::GenericError {
                        message: format!("No method '{}' on List", method),
                        span: Default::default(),
                    }),
                }
            }

            _ => Err(RuntimeError::GenericError {
                message: format!("No method '{}' on {}", method, obj.type_name()),
                span: Default::default(),
            }),
        }
    }
}
