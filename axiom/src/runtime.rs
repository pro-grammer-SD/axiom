/// Axiom High-Performance Runtime — Bytecode Edition
use crate::ast::{ClassMember, Expr, Item, MatchPattern, Stmt, StringPart};
use crate::compiler::compile_program;
use crate::core::oop::{AxCallable, AxClass, AxInstance};
use crate::core::value::AxValue;
use crate::errors::RuntimeError;
use crate::intrinsics;
use crate::vm_core::{Val, VmCore, VmFun};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct Env {
    frames: Vec<HashMap<String, AxValue>>,
}

impl Env {
    fn new() -> Self { Env { frames: vec![HashMap::new()] } }
    fn push_frame(&mut self) { self.frames.push(HashMap::new()); }
    fn pop_frame(&mut self)  { self.frames.pop(); }
    fn get(&self, name: &str) -> Option<&AxValue> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(name) { return Some(v); }
        }
        None
    }
    fn set(&mut self, name: &str, value: AxValue) -> bool {
        for frame in self.frames.iter_mut().rev() {
            if frame.contains_key(name) { frame.insert(name.to_string(), value); return true; }
        }
        false
    }
    fn define(&mut self, name: String, value: AxValue) {
        if let Some(f) = self.frames.last_mut() { f.insert(name, value); }
    }
}

pub struct Runtime {
    pub globals: HashMap<String, AxValue>,
    pub classes: HashMap<String, Arc<AxClass>>,
    call_depth: std::cell::Cell<usize>,
}

const MAX_CALL_DEPTH: usize = 1000;

impl Runtime {
    pub fn new() -> Self {
        let mut globals: HashMap<String, AxValue> = HashMap::new();
        macro_rules! native {
            ($name:expr, $body:expr) => {
                globals.insert($name.into(), AxValue::Fun(Arc::new(AxCallable::Native { name: $name.into(), func: $body })));
            };
        }
        native!("type", |args| args.first().map(|a| AxValue::Str(a.type_name().to_string())).unwrap_or(AxValue::Nil));
        native!("int", |args| match args.first() {
            Some(AxValue::Num(n)) => AxValue::Num(*n),
            Some(AxValue::Str(s)) => s.parse::<f64>().map(AxValue::Num).unwrap_or(AxValue::Nil),
            Some(AxValue::Bol(b)) => AxValue::Num(if *b { 1.0 } else { 0.0 }),
            _ => AxValue::Nil,
        });
        native!("str", |args| args.first().map(|a| AxValue::Str(a.display())).unwrap_or(AxValue::Nil));
        native!("bol", |args| args.first().map(|a| AxValue::Bol(a.is_truthy())).unwrap_or(AxValue::Nil));
        native!("out", |args| { println!("{}", args.iter().map(|a| a.display()).collect::<Vec<_>>().join(" ")); AxValue::Nil });
        native!("print", |args| { println!("{}", args.iter().map(|a| a.display()).collect::<Vec<_>>().join(" ")); AxValue::Nil });
        native!("in", |args| {
            use std::io::Write;
            if let Some(AxValue::Str(p)) = args.first() { print!("{}", p); let _ = std::io::stdout().flush(); }
            let mut s = String::new();
            match std::io::stdin().read_line(&mut s) {
                Ok(_) => AxValue::Str(s.trim_end_matches(['\r', '\n']).to_string()),
                Err(_) => AxValue::Nil,
            }
        });
        native!("sqrt",  |args| match args.first() { Some(AxValue::Num(n)) => AxValue::Num(n.sqrt()), _ => AxValue::Nil });
        native!("abs",   |args| match args.first() { Some(AxValue::Num(n)) => AxValue::Num(n.abs()), _ => AxValue::Nil });
        native!("floor", |args| match args.first() { Some(AxValue::Num(n)) => AxValue::Num(n.floor()), _ => AxValue::Nil });
        native!("ceil",  |args| match args.first() { Some(AxValue::Num(n)) => AxValue::Num(n.ceil()), _ => AxValue::Nil });
        native!("pow",   |args| match (args.first(), args.get(1)) { (Some(AxValue::Num(b)), Some(AxValue::Num(e))) => AxValue::Num(b.powf(*e)), _ => AxValue::Nil });
        native!("min",   |args| match (args.first(), args.get(1)) { (Some(AxValue::Num(a)), Some(AxValue::Num(b))) => AxValue::Num(a.min(*b)), _ => AxValue::Nil });
        native!("max",   |args| match (args.first(), args.get(1)) { (Some(AxValue::Num(a)), Some(AxValue::Num(b))) => AxValue::Num(a.max(*b)), _ => AxValue::Nil });
        native!("avg",   |args| match args.first() {
            Some(AxValue::Lst(items)) => {
                let items = items.read().unwrap();
                if items.is_empty() { return AxValue::Nil; }
                let sum: f64 = items.iter().filter_map(|v| v.as_num().ok()).sum();
                AxValue::Num(sum / items.len() as f64)
            }
            _ => AxValue::Nil,
        });
        globals.insert("nil".into(), AxValue::Nil);
        // chdir / cwd — registered here so both VM and tree-walk paths can find them
        globals.insert("chdir".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "chdir".into(),
            func: |args| match args.first() {
                Some(AxValue::Str(path)) => match std::env::set_current_dir(path) {
                    Ok(_)  => AxValue::Bol(true),
                    Err(e) => AxValue::Str(format!("ERROR: {}", e)),
                },
                _ => AxValue::Str("ERROR: chdir expects a string path".into()),
            },
        })));
        globals.insert("cwd".into(), AxValue::Fun(Arc::new(AxCallable::Native {
            name: "cwd".into(),
            func: |_args| match std::env::current_dir() {
                Ok(path) => AxValue::Str(path.display().to_string()),
                Err(e)   => AxValue::Str(format!("ERROR: {}", e)),
            },
        })));
        intrinsics::register(&mut globals);
        // Register nil as a global constant
        globals.insert("nil".to_string(), AxValue::Nil);
        Runtime { globals, classes: HashMap::new(), call_depth: std::cell::Cell::new(0) }
    }

    pub fn run(&mut self, items: Vec<Item>) -> Result<(), RuntimeError> {
        // Use tree-walk runtime for all programs
        // The VM path has issues with module marshaling; it's an optimization that needs proper globals bridging
        self.run_tree_walk(items)
    }

    fn run_via_vm(&mut self, items: &[Item]) -> Result<bool, RuntimeError> {
        let needs_tree_walk = items.iter().any(|item| {
            matches!(item, Item::ClassDecl { .. } | Item::LoadStmt { .. })
        });
        if needs_tree_walk { return Ok(false); }

        let (proto, global_table) = compile_program(items, "<main>");
        let n_globals = global_table.names.len();
        let mut vm = VmCore::new(n_globals + 64);

        for (idx, name) in global_table.names.iter().enumerate() {
            if let Some(ax_val) = self.globals.get(name) {
                match ax_val {
                    AxValue::Fun(callable) => {
                        if let AxCallable::Native { name: fn_name, func } = callable.as_ref() {
                            let func_ptr = *func;
                            let fn_name_c = fn_name.clone();
                            let vm_fn = VmFun::Native {
                                name: fn_name_c,
                                func: Box::new(move |args: &[Val]| {
                                    let ax_args: Vec<AxValue> = args.iter().map(VmCore::val_to_ax).collect();
                                    Ok(VmCore::ax_to_val(&func_ptr(ax_args)))
                                }),
                            };
                            vm.set_global_at(idx, Val::Fun(Arc::new(vm_fn)));
                        }
                    }
                    other => {
                        let v = VmCore::ax_to_val(other);
                        if !matches!(v, Val::Nil) { vm.set_global_at(idx, v); }
                    }
                }
            }
        }

        let proto = Arc::new(proto);
        vm.run(proto)?;

        for (idx, name) in global_table.names.iter().enumerate() {
            let vm_val = vm.get_global_at(idx);
            if !matches!(vm_val, Val::Nil) {
                self.globals.insert(name.clone(), VmCore::val_to_ax(&vm_val));
            }
        }

        Ok(true)
    }

    fn run_tree_walk(&mut self, items: Vec<Item>) -> Result<(), RuntimeError> {
        for item in &items { self.register_decl(item); }
        let mut env = Env::new();
        for item in &items {
            if let Item::LoadStmt { path, is_lib, alias, .. } = item {
                self.handle_load(path, *is_lib, alias.as_deref(), &mut env)?;
            }
        }
        for item in &items {
            if let Item::Statement(stmt) = item { self.exec_stmt(stmt, &mut env)?; }
        }
        if let Some(main_fn) = self.globals.get("main").cloned() {
            self.call_value(main_fn, vec![], &mut env)?;
        }
        Ok(())
    }

    fn handle_load(&mut self, path: &str, is_lib: bool, alias: Option<&str>, env: &mut Env) -> Result<(), RuntimeError> {
        use crate::pkg::AxiomiteConfig;
        use std::path::PathBuf;
        let (resolved_path, pkg_root) = if is_lib {
            let home = dirs::home_dir().ok_or_else(|| RuntimeError::GenericError { message: "Cannot determine home directory".into(), span: Default::default() })?;
            let parts: Vec<&str> = path.trim_start_matches('@').split('/').collect();
            if parts.len() != 2 { return Err(RuntimeError::GenericError { message: format!("Invalid library path: {}", path), span: Default::default() }); }
            let root = home.join(".axiomlibs").join(parts[0]).join(parts[1]);
            (root.join("lib.ax"), Some(root))
        } else {
            let p = PathBuf::from(path);
            let root = p.parent().map(|r| r.to_path_buf());
            (p, root)
        };
        if let Some(ref root) = pkg_root {
            let toml_path = root.join("Axiomite.toml");
            if toml_path.exists() {
                if let Ok(config) = AxiomiteConfig::from_file(&toml_path) {
                    for (k, v) in &config.env { std::env::set_var(k, v); self.globals.insert(k.clone(), AxValue::Str(v.clone())); }
                    for dep in &config.dependencies.requires {
                        if !self.globals.contains_key(dep.as_str()) { self.handle_load(&format!("@{}", dep), true, None, env)?; }
                    }
                }
            }
            let init_path = root.join("init.ax");
            if init_path.exists() {
                if let Ok(src) = std::fs::read_to_string(&init_path) {
                    let mut p = crate::Parser::new(&src, 0);
                    if let Ok(init_items) = p.parse() {
                        for item in &init_items { self.register_decl(item); }
                        for item in &init_items { if let Item::Statement(s) = item { self.exec_stmt(s, env)?; } }
                    }
                }
            }
        }
        let source = std::fs::read_to_string(&resolved_path).map_err(|e| RuntimeError::GenericError { message: format!("Cannot load '{}': {}", resolved_path.display(), e), span: Default::default() })?;
        let mut parser = crate::Parser::new(&source, 0);
        let loaded_items = parser.parse().map_err(|e| RuntimeError::GenericError { message: format!("Parse error in '{}': {}", resolved_path.display(), e), span: Default::default() })?;
        let module_map = Arc::new(DashMap::new());
        for item in &loaded_items {
            self.register_decl(item);
            if let Item::FunctionDecl { name, params, body, .. } = item {
                module_map.insert(name.clone(), AxValue::Fun(Arc::new(AxCallable::UserDefined { params: params.clone(), body: body.clone(), captured: std::collections::HashMap::new() })));
            }
        }
        for item in &loaded_items {
            if let Item::Statement(stmt) = item {
                self.exec_stmt(stmt, env)?;
                if let Stmt::Let { name, .. } = stmt {
                    if let Some(v) = self.globals.get(name) { module_map.insert(name.clone(), v.clone()); }
                    else if let Some(v) = env.get(name) { module_map.insert(name.clone(), v.clone()); }
                }
            }
        }
        let module_val = AxValue::Map(module_map);
        let full_key = path.trim_start_matches('@').replace('/', ".").replace('-', "_");
        self.globals.insert(full_key, module_val.clone());
        self.globals.insert(path.to_string(), module_val.clone());
        if let Some(a) = alias { self.globals.insert(a.to_string(), module_val); }
        Ok(())
    }

    fn register_decl(&mut self, item: &Item) {
        match item {
            Item::FunctionDecl { name, params, body, .. } => {
                self.globals.insert(name.clone(), AxValue::Fun(Arc::new(AxCallable::UserDefined { params: params.clone(), body: body.clone(), captured: std::collections::HashMap::new() })));
            }
            Item::ClassDecl { name, body, .. } => {
                let mut ax_class = AxClass::new(name.clone());
                for member in body {
                    match member {
                        ClassMember::Method { name: mn, params, body, .. } => {
                            ax_class.methods.insert(mn.clone(), AxCallable::UserDefined { params: params.clone(), body: body.clone(), captured: std::collections::HashMap::new() });
                        }
                        ClassMember::Field { name: fn_, default, .. } => { ax_class.fields.push((fn_.clone(), default.clone())); }
                    }
                }
                self.classes.insert(name.clone(), Arc::new(ax_class));
            }
            Item::EnumDecl { name, variants, .. } => {
                for v in variants { self.globals.insert(format!("{}.{}", name, v.name), AxValue::Str(format!("{}.{}", name, v.name))); }
                self.globals.insert(name.clone(), AxValue::Str(name.clone()));
            }
            _ => {}
        }
    }

    fn exec_stmt(&self, stmt: &Stmt, env: &mut Env) -> Result<Option<AxValue>, RuntimeError> {
        match stmt {
            Stmt::Let { name, value, .. } => { let val = self.eval(value, env)?; env.define(name.clone(), val); }
            Stmt::Expr(e) => { self.eval(e, env)?; }
            Stmt::Out { arguments, .. } => {
                let mut parts = Vec::with_capacity(arguments.len());
                for arg in arguments { parts.push(self.eval(arg, env)?.display()); }
                println!("{}", parts.join(""));
            }
            Stmt::Return { value, .. } => {
                let v = match value { Some(e) => self.eval(e, env)?, None => AxValue::Nil };
                return Ok(Some(v));
            }
            Stmt::If { condition, then_body, else_body, .. } => {
                if self.eval(condition, env)?.is_truthy() { return self.exec_block(then_body, env); }
                else if let Some(eb) = else_body { return self.exec_block(eb, env); }
            }
            Stmt::While { condition, body, .. } => {
                while self.eval(condition, env)?.is_truthy() {
                    if let Some(ret) = self.exec_block(body, env)? { return Ok(Some(ret)); }
                }
            }
            Stmt::For { var, iterable, body, .. } => {
                let iter_val = self.eval(iterable, env)?;
                let items = match &iter_val {
                    AxValue::Lst(list) => list.read().unwrap().clone(),
                    AxValue::Str(s) => s.chars().map(|c| AxValue::Str(c.to_string())).collect(),
                    _ => return Err(RuntimeError::GenericError { message: format!("'{}' is not iterable", iter_val.type_name()), span: Default::default() }),
                };
                for item in items {
                    env.push_frame(); env.define(var.clone(), item);
                    let ret = self.exec_block_in_env(body, env)?;
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
                        if let MatchPattern::EnumVariant { binding: Some(b), .. } = &arm.pattern { env.define(b.clone(), val.clone()); }
                        let ret = self.exec_block_in_env(&arm.body, env)?;
                        env.pop_frame();
                        return Ok(ret);
                    }
                }
            }
            Stmt::GoSpawn { body, .. } => {
                let g = self.globals.clone(); let c = self.classes.clone(); let body = body.clone();
                tokio::spawn(async move { let rt = Runtime { globals: g, classes: c, call_depth: std::cell::Cell::new(0) }; let mut env = Env::new(); let _ = rt.exec_block_in_env(&body, &mut env); });
            }
        }
        Ok(None)
    }

    fn exec_block(&self, stmts: &[Stmt], env: &mut Env) -> Result<Option<AxValue>, RuntimeError> {
        env.push_frame(); let ret = self.exec_block_in_env(stmts, env); env.pop_frame(); ret
    }

    pub fn exec_block_in_env(&self, stmts: &[Stmt], env: &mut Env) -> Result<Option<AxValue>, RuntimeError> {
        for stmt in stmts { if let Some(ret) = self.exec_stmt(stmt, env)? { return Ok(Some(ret)); } }
        Ok(None)
    }

    fn pattern_matches(&self, pattern: &MatchPattern, value: &AxValue) -> bool {
        match pattern {
            MatchPattern::Wildcard | MatchPattern::Identifier(_) => true,
            MatchPattern::Literal(expr) => {
                let mut env = Env::new();
                if let Ok(lit) = self.eval(expr, &mut env) { self.values_equal(&lit, value) } else { false }
            }
            MatchPattern::EnumVariant { enum_name, variant, .. } => {
                let expected = match enum_name { Some(e) => format!("{}.{}", e, variant), None => variant.clone() };
                match value { AxValue::Str(s) => s == &expected || s.ends_with(&format!(".{}", variant)), _ => false }
            }
        }
    }

    fn values_equal(&self, a: &AxValue, b: &AxValue) -> bool {
        match (a, b) {
            (AxValue::Num(x), AxValue::Num(y)) => x == y,
            (AxValue::Str(x), AxValue::Str(y)) => x == y,
            (AxValue::Bol(x), AxValue::Bol(y)) => x == y,
            (AxValue::Nil, AxValue::Nil) => true,
            _ => false,
        }
    }

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
                    Expr::Identifier { name, .. } => { if !env.set(name, val.clone()) { env.define(name.clone(), val.clone()); } }
                    Expr::MemberAccess { object, member, .. } => {
                        if let AxValue::Instance(inst) = self.eval(object, env)? { inst.write().unwrap().fields.insert(member.clone(), val.clone()); }
                    }
                    _ => {}
                }
                Ok(val)
            }
            Expr::UnaryOp { op, operand, .. } => {
                let v = self.eval(operand, env)?;
                match op.as_str() { "!" => Ok(AxValue::Bol(!v.is_truthy())), "-" => Ok(AxValue::Num(-v.as_num().unwrap_or(0.0))), _ => Ok(AxValue::Nil) }
            }
            Expr::BinaryOp { left, op, right, .. } => {
                match op.as_str() {
                    "&&" => { let l = self.eval(left, env)?; return if l.is_truthy() { self.eval(right, env) } else { Ok(l) }; }
                    "||" => { let l = self.eval(left, env)?; return if l.is_truthy() { Ok(l) } else { self.eval(right, env) }; }
                    _ => {}
                }
                let l = self.eval(left, env)?; let r = self.eval(right, env)?;
                match op.as_str() {
                    "+"  => match (&l, &r) { (AxValue::Num(a), AxValue::Num(b)) => Ok(AxValue::Num(a + b)), _ => Ok(AxValue::Str(format!("{}{}", l.display(), r.display()))) },
                    "-"  => Ok(AxValue::Num(l.as_num().unwrap_or(0.0) - r.as_num().unwrap_or(0.0))),
                    "*"  => Ok(AxValue::Num(l.as_num().unwrap_or(0.0) * r.as_num().unwrap_or(0.0))),
                    "/"  => { let d = r.as_num().unwrap_or(1.0); if d == 0.0 { return Err(RuntimeError::GenericError { message: "Division by zero".into(), span: Default::default() }); } Ok(AxValue::Num(l.as_num().unwrap_or(0.0) / d)) }
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
                if let Expr::Identifier { name, .. } = &**function {
                    if name == "str" {
                        if let Some(arg) = arguments.first() { return Ok(AxValue::Str(self.eval(arg, env)?.display())); }
                        return Ok(AxValue::Nil);
                    }
                }
                let func = self.eval(function, env)?;
                let mut args = Vec::with_capacity(arguments.len());
                for arg in arguments { args.push(self.eval(arg, env)?); }
                self.call_value(func, args, env)
            }
            Expr::MethodCall { object, method, arguments, .. } => {
                let obj = self.eval(object, env)?;
                let mut args = Vec::with_capacity(arguments.len()); for arg in arguments { args.push(self.eval(arg, env)?); }

                // ── Higher-order stdlib intercept ────────────────────────────
                // Native intrinsics cannot call user-defined functions because they lack
                // runtime context.  Intercept known higher-order patterns here so that
                // lambdas/closures passed to alg.map / alg.filter work correctly.
                if matches!(&obj, AxValue::Map(_)) {
                    match method.as_str() {
                        "map" => {
                            if let (Some(list_val), Some(fn_val)) = (args.first(), args.get(1)) {
                                if let (AxValue::Lst(list), AxValue::Fun(callable)) = (list_val, fn_val) {
                                    if matches!(callable.as_ref(), AxCallable::UserDefined { .. }) {
                                        let items = list.read().unwrap().clone();
                                        let func  = fn_val.clone();
                                        let mut results = Vec::with_capacity(items.len());
                                        for item in items {
                                            results.push(self.call_value(func.clone(), vec![item], env)?);
                                        }
                                        return Ok(AxValue::Lst(Arc::new(RwLock::new(results))));
                                    }
                                }
                            }
                        }
                        "filter" => {
                            if let (Some(list_val), Some(fn_val)) = (args.first(), args.get(1)) {
                                if let (AxValue::Lst(list), AxValue::Fun(callable)) = (list_val, fn_val) {
                                    if matches!(callable.as_ref(), AxCallable::UserDefined { .. }) {
                                        let items = list.read().unwrap().clone();
                                        let func  = fn_val.clone();
                                        let mut results = Vec::new();
                                        for item in items {
                                            let keep = self.call_value(func.clone(), vec![item.clone()], env)?;
                                            if keep.is_truthy() { results.push(item); }
                                        }
                                        return Ok(AxValue::Lst(Arc::new(RwLock::new(results))));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                // ── End higher-order intercept ───────────────────────────────

                self.call_method(obj, method, args, env)
            }
            Expr::MemberAccess { object, member, .. } => {
                let obj = self.eval(object, env)?;
                match &obj {
                    AxValue::Instance(inst) => {
                        let r = inst.read().unwrap();
                        if let Some(v) = r.fields.get(member) { return Ok(v.clone()); }
                        if let Some(m) = r.class.methods.get(member) { return Ok(AxValue::Fun(Arc::new(m.clone()))); }
                        Ok(AxValue::Nil)
                    }
                    AxValue::Str(s) => match member.as_str() { "len" => Ok(AxValue::Num(s.len() as f64)), _ => Ok(AxValue::Nil) }
                    AxValue::Map(map) => Ok(map.get(member).map(|v| (*v).clone()).unwrap_or(AxValue::Nil)),
                    AxValue::Lst(l) => match member.as_str() { "len" => Ok(AxValue::Num(l.read().unwrap().len() as f64)), _ => Ok(AxValue::Nil) }
                    _ => Ok(AxValue::Nil),
                }
            }
            Expr::Index { object, index, .. } => {
                let obj = self.eval(object, env)?; let idx = self.eval(index, env)?;
                match (&obj, &idx) {
                    (AxValue::Lst(list), AxValue::Num(n)) => {
                        let lst = list.read().unwrap(); let i = *n as isize; let len = lst.len() as isize;
                        let i = if i < 0 { len + i } else { i };
                        if i >= 0 && (i as usize) < lst.len() { Ok(lst[i as usize].clone()) }
                        else { Err(RuntimeError::GenericError { message: "Index out of range".into(), span: Default::default() }) }
                    }
                    (AxValue::Str(s), AxValue::Num(n)) => Ok(s.chars().nth(*n as usize).map(|c| AxValue::Str(c.to_string())).unwrap_or(AxValue::Nil)),
                    _ => Ok(AxValue::Nil),
                }
            }
            Expr::New { class_name, arguments, .. } => {
                let class = self.classes.get(class_name).cloned().ok_or_else(|| RuntimeError::GenericError { message: format!("Unknown class '{}'", class_name), span: Default::default() })?;
                let fields: DashMap<String, AxValue> = DashMap::new();
                for (fn_, default) in &class.fields { fields.insert(fn_.clone(), if let Some(e) = default { self.eval(e, env)? } else { AxValue::Nil }); }
                let inst = Arc::new(RwLock::new(AxInstance { class: Arc::clone(&class), fields }));
                let mut args = Vec::with_capacity(arguments.len()); for arg in arguments { args.push(self.eval(arg, env)?); }
                let iv = AxValue::Instance(Arc::clone(&inst));
                if let Some(AxCallable::UserDefined { params, body, captured }) = class.methods.get("init").cloned() {
                    env.push_frame();
                    for (k, v) in &captured { env.define(k.clone(), v.clone()); }
                    env.define("self".into(), iv.clone());
                    for (p, a) in params.iter().zip(args.iter()) { env.define(p.clone(), a.clone()); }
                    self.exec_block_in_env(&body, env)?; env.pop_frame();
                }
                Ok(iv)
            }
            Expr::List { items, .. } => {
                let mut vals = Vec::with_capacity(items.len()); for item in items { vals.push(self.eval(item, env)?); }
                Ok(AxValue::Lst(Arc::new(RwLock::new(vals))))
            }
            Expr::InterpolatedString { parts, .. } => {
                let mut result = String::new();
                for part in parts { match part { StringPart::Literal(s) => result.push_str(s), StringPart::Expr(e) => result.push_str(&self.eval(e, env)?.display()) } }
                Ok(AxValue::Str(result))
            }
            // Lambda expression: fn(params) { body } — creates a callable value.
            // This is used directly for anonymous lambdas AND for named nested
            // functions that the parser rewrites as: let name = fn(params) { body }
            // We capture the current environment as a closure snapshot.
            Expr::Lambda { params, body, .. } => {
                let mut captured = std::collections::HashMap::new();
                for frame in &env.frames {
                    for (k, v) in frame {
                        captured.insert(k.clone(), v.clone());
                    }
                }
                Ok(AxValue::Fun(Arc::new(AxCallable::UserDefined {
                    params: params.clone(),
                    body: body.clone(),
                    captured,
                })))
            }
            _ => Ok(AxValue::Nil),
        }
    }

    fn lookup(&self, name: &str, env: &Env) -> Result<AxValue, RuntimeError> {
        if let Some(v) = env.get(name) { return Ok(v.clone()); }
        if let Some(v) = self.globals.get(name) { return Ok(v.clone()); }
        Err(RuntimeError::UndefinedVariable { name: name.to_string(), span: Default::default() })
    }

    pub fn call_value(&self, func: AxValue, args: Vec<AxValue>, env: &mut Env) -> Result<AxValue, RuntimeError> {
        let depth = self.call_depth.get();
        if depth >= MAX_CALL_DEPTH {
            return Err(RuntimeError::GenericError {
                message: "[AXM_408] Call stack overflow — frame limit reached. Check for infinite recursion.".to_string(),
                span: Default::default(),
            });
        }
        self.call_depth.set(depth + 1);
        let result = self.call_value_inner(func, args, env);
        self.call_depth.set(depth);
        result
    }

    fn call_value_inner(&self, func: AxValue, args: Vec<AxValue>, env: &mut Env) -> Result<AxValue, RuntimeError> {
        match func {
            AxValue::Fun(callable) => match &*callable {
                AxCallable::Native { func, .. } => Ok(func(args)),
                AxCallable::UserDefined { params, body, captured } => {
                    if args.len() != params.len() {
                        return Err(RuntimeError::ArityMismatch {
                            expected: params.len(),
                            found: args.len(),
                        });
                    }
                    env.push_frame();
                    // Inject captured closure variables first (so params can override them)
                    for (k, v) in captured {
                        env.define(k.clone(), v.clone());
                    }
                    for (p, a) in params.iter().zip(args.iter()) { env.define(p.clone(), a.clone()); }
                    let ret = self.exec_block_in_env(body, env)?; env.pop_frame();
                    Ok(ret.unwrap_or(AxValue::Nil))
                }
            }
            AxValue::Nil => Err(RuntimeError::NilCall {
                hint: "Value is nil — check that the variable is assigned before use (AXM_402)".into(),
                span: Default::default(),
            }),
            _ => Err(RuntimeError::GenericError { message: format!("Not callable: {}", func.type_name()), span: Default::default() }),
        }
    }

    fn call_method(&self, obj: AxValue, method: &str, args: Vec<AxValue>, env: &mut Env) -> Result<AxValue, RuntimeError> {
        let depth = self.call_depth.get();
        if depth >= MAX_CALL_DEPTH {
            return Err(RuntimeError::GenericError {
                message: "[AXM_408] Call stack overflow — frame limit reached.".to_string(),
                span: Default::default(),
            });
        }
        self.call_depth.set(depth + 1);
        let result = self.call_method_inner(obj, method, args, env);
        self.call_depth.set(depth);
        result
    }

    fn call_method_inner(&self, obj: AxValue, method: &str, args: Vec<AxValue>, env: &mut Env) -> Result<AxValue, RuntimeError> {
        match &obj {
            AxValue::Instance(inst) => {
                let callable = { inst.read().unwrap().class.methods.get(method).cloned() };
                match callable {
                    Some(AxCallable::UserDefined { params, body, captured }) => {
                        env.push_frame();
                        for (k, v) in &captured { env.define(k.clone(), v.clone()); }
                        env.define("self".into(), obj.clone());
                        for (p, a) in params.iter().zip(args.iter()) { env.define(p.clone(), a.clone()); }
                        let ret = self.exec_block_in_env(&body, env)?; env.pop_frame();
                        Ok(ret.unwrap_or(AxValue::Nil))
                    }
                    Some(AxCallable::Native { func, .. }) => Ok(func(args)),
                    None => Err(RuntimeError::GenericError { message: format!("No method '{}' on instance", method), span: Default::default() }),
                }
            }
            AxValue::Map(map) => {
                if let Some(v) = map.get(method) { return self.call_value((*v).clone(), args, env); }
                Err(RuntimeError::GenericError { message: format!("No method '{}' on Map", method), span: Default::default() })
            }
            AxValue::Str(s) => {
                match method {
                    "len"       => Ok(AxValue::Num(s.len() as f64)),
                    "upper"     => Ok(AxValue::Str(s.to_uppercase())),
                    "lower"     => Ok(AxValue::Str(s.to_lowercase())),
                    "trim"      => Ok(AxValue::Str(s.trim().to_string())),
                    "split"     => { let sep = args.first().and_then(|a| if let AxValue::Str(s) = a { Some(s.as_str()) } else { None }).unwrap_or(" "); Ok(AxValue::Lst(Arc::new(RwLock::new(s.split(sep).map(|p| AxValue::Str(p.to_string())).collect())))) }
                    "contains"  => Ok(AxValue::Bol(s.contains(&args.first().map(|a| a.display()).unwrap_or_default()))),
                    "starts_with" => Ok(AxValue::Bol(s.starts_with(&args.first().map(|a| a.display()).unwrap_or_default()))),
                    "ends_with" => Ok(AxValue::Bol(s.ends_with(&args.first().map(|a| a.display()).unwrap_or_default()))),
                    "replace"   => { let from = args.first().map(|a| a.display()).unwrap_or_default(); let to = args.get(1).map(|a| a.display()).unwrap_or_default(); Ok(AxValue::Str(s.replace(&from, &to))) }
                    "align"     => { let w = args.first().and_then(|a| a.as_num().ok()).unwrap_or(0.0) as usize; let d = args.get(1).map(|a| a.display()).unwrap_or_else(|| "left".into()); Ok(AxValue::Str(match d.as_str() { "right" => format!("{:>width$}", s, width=w), "center" => format!("{:^width$}", s, width=w), _ => format!("{:<width$}", s, width=w) })) }
                    _ => Err(RuntimeError::GenericError { message: format!("No method '{}' on Str", method), span: Default::default() }),
                }
            }
            AxValue::Lst(list) => {
                match method {
                    "len"      => Ok(AxValue::Num(list.read().unwrap().len() as f64)),
                    "push"     => { if let Some(v) = args.into_iter().next() { list.write().unwrap().push(v); } Ok(AxValue::Nil) }
                    "pop"      => Ok(list.write().unwrap().pop().unwrap_or(AxValue::Nil)),
                    "first"    => Ok(list.read().unwrap().first().cloned().unwrap_or(AxValue::Nil)),
                    "last"     => Ok(list.read().unwrap().last().cloned().unwrap_or(AxValue::Nil)),
                    "contains" => { let needle = args.first().cloned().unwrap_or(AxValue::Nil); Ok(AxValue::Bol(list.read().unwrap().iter().any(|v| self.values_equal(v, &needle)))) }
                    "join"     => { let sep = args.first().map(|a| a.display()).unwrap_or_default(); Ok(AxValue::Str(list.read().unwrap().iter().map(|v: &AxValue| v.display()).collect::<Vec<_>>().join(&sep))) }
                    _ => Err(RuntimeError::GenericError { message: format!("No method '{}' on List", method), span: Default::default() }),
                }
            }
            _ => Err(RuntimeError::GenericError { message: format!("No method '{}' on {}", method, obj.type_name()), span: Default::default() }),
        }
    }
}
