/// Axiom Static Analyzer (chk) — Final Maturation
/// Performs semantic analysis, symbol resolution, and type inference.
use crate::ast::{Item, Stmt, Expr, MatchPattern, ClassMember};
use crate::errors::{Diagnostic, DiagnosticLevel, Span};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Shared Semantic Structures
// ---------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum AxType {
    Num,
    Str,
    Bool,
    List(Box<AxType>),
    Map(Box<AxType>),
    Class(String),
    Enum(String),
    Func { params: Vec<AxType>, ret: Box<AxType> },
    Any,
    Nil,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: AxType,
    pub span: Span,
    pub is_const: bool,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<usize>,
}

// ---------------------------------------------------------------------------
// Semantic Analyzer
// ---------------------------------------------------------------------------
pub struct SemanticAnalyzer {
    pub scopes: Vec<Scope>,
    pub current_scope: usize,
    pub diagnostics: Vec<Diagnostic>,
    pub classes: HashSet<String>,
    pub enums: HashSet<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut global_scope = Scope {
            symbols: HashMap::new(),
            parent: None,
        };

        // Register built-in functions
        let builtins = vec![
            ("out", AxType::Func { params: vec![], ret: Box::new(AxType::Nil) }),
            ("in", AxType::Func { params: vec![], ret: Box::new(AxType::Str) }),
            ("type", AxType::Func { params: vec![AxType::Any], ret: Box::new(AxType::Str) }),
            ("int", AxType::Func { params: vec![AxType::Any], ret: Box::new(AxType::Num) }),
            ("str", AxType::Func { params: vec![AxType::Any], ret: Box::new(AxType::Str) }),
            ("bol", AxType::Func { params: vec![AxType::Any], ret: Box::new(AxType::Bool) }),
            ("avg", AxType::Func { params: vec![AxType::List(Box::new(AxType::Any))], ret: Box::new(AxType::Num) }),
            ("sqrt", AxType::Func { params: vec![AxType::Num], ret: Box::new(AxType::Num) }),
        ];

        for (name, ty) in builtins {
            global_scope.symbols.insert(
                name.to_string(),
                Symbol {
                    name: name.to_string(),
                    ty,
                    span: Span::default(),
                    is_const: true,
                },
            );
        }

        SemanticAnalyzer {
            scopes: vec![global_scope],
            current_scope: 0,
            diagnostics: Vec::new(),
            classes: HashSet::new(),
            enums: HashSet::new(),
        }
    }

    pub fn check(&mut self, items: &[Item]) -> Vec<Diagnostic> {
        // Pass 1: Collect top-level declarations (hoisting)
        self.collect_declarations(items);

        // Pass 2: Deep analysis
        for item in items {
            self.analyze_item(item);
        }

        self.diagnostics.clone()
    }

    fn collect_declarations(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::FunctionDecl { name, .. } => {
                    self.define_symbol(name, AxType::Func { params: vec![], ret: Box::new(AxType::Any) }, Span::default());
                }
                Item::ClassDecl { name, .. } => {
                    self.classes.insert(name.clone());
                    self.define_symbol(name, AxType::Class(name.clone()), Span::default());
                }
                Item::EnumDecl { name, .. } => {
                    self.enums.insert(name.clone());
                    self.define_symbol(name, AxType::Enum(name.clone()), Span::default());
                }
                Item::StdImport { name, span } | Item::LocImport { name, span } => {
                    self.validate_local_path(name, *span);
                }
                _ => {}
            }
        }
    }

    fn validate_local_path(&mut self, name: &str, span: Span) {
        let mut path = PathBuf::from(format!("{}.ax", name));
        if !path.exists() {
            path.set_extension("rax");
        }
        if !path.exists() {
            self.diagnostics.push(Diagnostic {
                level: DiagnosticLevel::Error,
                message: format!("Module '{}' not found", name),
                span,
                hint: Some(format!("Ensure '{}' exists in the current directory", name)),
            });
        }
    }

    fn analyze_item(&mut self, item: &Item) {
        match item {
            Item::FunctionDecl { params, body, .. } => {
                self.enter_scope();
                for p in params {
                    self.define_symbol(p, AxType::Any, Span::default());
                }
                self.analyze_block(body);
                self.exit_scope();
            }
            Item::ClassDecl { body, .. } => {
                self.enter_scope();
                self.define_symbol("self", AxType::Any, Span::default());
                for member in body {
                    match member {
                        ClassMember::Method { params, body, .. } => {
                            self.enter_scope();
                            for p in params {
                                self.define_symbol(p, AxType::Any, Span::default());
                            }
                            self.analyze_block(body);
                            self.exit_scope();
                        }
                        ClassMember::Field { default, .. } => {
                            if let Some(expr) = default {
                                self.analyze_expr(expr);
                            }
                        }
                    }
                }
                self.exit_scope();
            }
            Item::Statement(stmt) => self.analyze_stmt(stmt),
            _ => {}
        }
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, value, span } => {
                let ty = self.analyze_expr(value);
                self.define_symbol(name, ty, *span);
            }
            Stmt::Expr(expr) => { self.analyze_expr(expr); }
            Stmt::If { condition, then_body, else_body, .. } => {
                self.analyze_expr(condition);
                self.analyze_block(then_body);
                if let Some(eb) = else_body {
                    self.analyze_block(eb);
                }
            }
            Stmt::While { condition, body, .. } => {
                self.analyze_expr(condition);
                self.analyze_block(body);
            }
            Stmt::For { var, iterable, body, .. } => {
                self.analyze_expr(iterable);
                self.enter_scope();
                self.define_symbol(var, AxType::Any, Span::default());
                self.analyze_block(body);
                self.exit_scope();
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.analyze_expr(v);
                }
            }
            Stmt::Block(stmts) => self.analyze_block(stmts),
            Stmt::GoSpawn { body, .. } => self.analyze_block(body),
            Stmt::Match { expr, arms, .. } => {
                self.analyze_expr(expr);
                for arm in arms {
                    self.enter_scope();
                    self.analyze_pattern(&arm.pattern);
                    self.analyze_block(&arm.body);
                    self.exit_scope();
                }
            }
            Stmt::Out { arguments, .. } => {
                for arg in arguments {
                    self.analyze_expr(arg);
                }
            }
        }
    }

    fn analyze_pattern(&mut self, pattern: &MatchPattern) {
        match pattern {
            MatchPattern::Identifier(i) => {
                self.define_symbol(i, AxType::Any, Span::default());
            }
            MatchPattern::EnumVariant { variant: _, binding, .. } => {
                if let Some(b) = binding {
                    self.define_symbol(b, AxType::Any, Span::default());
                }
            }
            _ => {}
        }
    }

    fn analyze_block(&mut self, stmts: &[Stmt]) {
        self.enter_scope();
        for stmt in stmts {
            self.analyze_stmt(stmt);
        }
        self.exit_scope();
    }

    fn analyze_expr(&mut self, expr: &Expr) -> AxType {
        match expr {
            Expr::Number { .. } => AxType::Num,
            Expr::String { .. } => AxType::Str,
            Expr::Boolean { .. } => AxType::Bool,
            Expr::Identifier { name, span } => {
                if let Some(sym) = self.resolve_symbol(name) {
                    sym.ty.clone()
                } else {
                    self.diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Error,
                        message: format!("Undefined variable '{}'", name),
                        span: *span,
                        hint: None,
                    });
                    AxType::Any
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                self.analyze_expr(left);
                self.analyze_expr(right);
                AxType::Any // Type inference would go here
            }
            Expr::Call { function, arguments, .. } => {
                self.analyze_expr(function);
                for arg in arguments {
                    self.analyze_expr(arg);
                }
                AxType::Any
            }
            Expr::New { class_name, span, .. } => {
                if !self.classes.contains(class_name) {
                    self.diagnostics.push(Diagnostic {
                        level: DiagnosticLevel::Error,
                        message: format!("Undefined class '{}'", class_name),
                        span: *span,
                        hint: None,
                    });
                }
                AxType::Class(class_name.clone())
            }
            Expr::InterpolatedString { parts, .. } => {
                for part in parts {
                    if let crate::ast::StringPart::Expr(e) = part {
                        self.analyze_expr(e);
                    }
                }
                AxType::Str
            }
            _ => AxType::Any,
        }
    }

    // -----------------------------------------------------------------------
    // Helper Methods
    // -----------------------------------------------------------------------

    fn enter_scope(&mut self) {
        let new_scope = Scope {
            symbols: HashMap::new(),
            parent: Some(self.current_scope),
        };
        self.scopes.push(new_scope);
        self.current_scope = self.scopes.len() - 1;
    }

    fn exit_scope(&mut self) {
        if let Some(parent) = self.scopes[self.current_scope].parent {
            self.current_scope = parent;
        }
    }

    fn define_symbol(&mut self, name: &str, ty: AxType, span: Span) {
        self.scopes[self.current_scope].symbols.insert(
            name.to_string(),
            Symbol {
                name: name.to_string(),
                ty,
                span,
                is_const: false,
            },
        );
    }

    fn resolve_symbol(&self, name: &str) -> Option<&Symbol> {
        let mut current = Some(self.current_scope);
        while let Some(idx) = current {
            if let Some(sym) = self.scopes[idx].symbols.get(name) {
                return Some(sym);
            }
            current = self.scopes[idx].parent;
        }
        None
    }
}
