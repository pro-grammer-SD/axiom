use crate::parser::{Expr, Stmt};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Number,
    String,
    Bool,
    List(Box<Type>),
    Nil,
    Any, // For dynamically typed expressions
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type Error: {}", self.message)
    }
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    functions: HashMap<String, FunctionSignature>,
    in_function: bool,
    function_return_type: Option<Type>,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            in_function: false,
            function_return_type: None,
            errors: Vec::new(),
        }
    }

    pub fn check(&mut self, stmts: &[Stmt]) -> Result<(), Vec<TypeError>> {
        // First pass: collect all function definitions
        for stmt in stmts {
            if let Stmt::Function {
                name,
                params,
                body: _,
            } = stmt
            {
                let param_types = params.iter().map(|p| (p.clone(), Type::Any)).collect();
                self.functions.insert(
                    name.clone(),
                    FunctionSignature {
                        params: param_types,
                        return_type: Type::Any,
                    },
                );
            }
        }

        // Second pass: check each statement
        for stmt in stmts {
            self.check_stmt(stmt);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                let _ = self.check_expr(expr);
            }
            Stmt::Let { name, value } => {
                if let Ok(ty) = self.check_expr(value) {
                    self.define_var(name.clone(), ty);
                }
            }
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                if let Ok(cond_ty) = self.check_expr(cond) {
                    if !self.is_bool_or_any(&cond_ty) {
                        self.error(format!(
                            "If condition must be bool, got {:?}",
                            cond_ty
                        ));
                    }
                }

                self.push_scope();
                for s in then_body {
                    self.check_stmt(s);
                }
                self.pop_scope();

                if let Some(body) = else_body {
                    self.push_scope();
                    for s in body {
                        self.check_stmt(s);
                    }
                    self.pop_scope();
                }
            }
            Stmt::While { cond, body } => {
                if let Ok(cond_ty) = self.check_expr(cond) {
                    if !self.is_bool_or_any(&cond_ty) {
                        self.error(format!("While condition must be bool, got {:?}", cond_ty));
                    }
                }

                self.push_scope();
                for s in body {
                    self.check_stmt(s);
                }
                self.pop_scope();
            }
            Stmt::For { var, iter, body } => {
                if let Ok(iter_ty) = self.check_expr(iter) {
                    match iter_ty {
                        Type::List(elem_ty) => {
                            self.push_scope();
                            self.define_var(var.clone(), *elem_ty);
                            for s in body {
                                self.check_stmt(s);
                            }
                            self.pop_scope();
                        }
                        Type::Any => {
                            self.push_scope();
                            self.define_var(var.clone(), Type::Any);
                            for s in body {
                                self.check_stmt(s);
                            }
                            self.pop_scope();
                        }
                        _ => {
                            self.error(format!("For loop requires list, got {:?}", iter_ty));
                        }
                    }
                }
            }
            Stmt::Return(Some(expr)) => {
                if let Ok(ret_ty) = self.check_expr(expr) {
                    if self.in_function {
                        if let Some(ref expected_ty) = self.function_return_type {
                            if !self.types_compatible(&ret_ty, expected_ty) {
                                self.error(format!(
                                    "Return type mismatch: expected {:?}, got {:?}",
                                    expected_ty, ret_ty
                                ));
                            }
                        }
                    } else {
                        self.error("Return statement outside function".to_string());
                    }
                }
            }
            Stmt::Return(None) => {
                if !self.in_function {
                    self.error("Return statement outside function".to_string());
                }
            }
            Stmt::Function {
                name,
                params,
                body,
            } => {
                // Check for duplicate definitions
                if self.scopes[0].contains_key(name) {
                    self.error(format!("Function {} already defined", name));
                    return;
                }

                let was_in_function = self.in_function;
                self.in_function = true;
                self.function_return_type = Some(Type::Any);

                self.push_scope();
                for param in params {
                    self.define_var(param.clone(), Type::Any);
                }

                for stmt in body {
                    self.check_stmt(stmt);
                }

                self.pop_scope();

                self.in_function = was_in_function;
                self.function_return_type = None;

                self.define_var(name.clone(), Type::Any);
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> {
        match expr {
            Expr::Number(_) => Ok(Type::Number),
            Expr::String(_) => Ok(Type::String),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::Ident(name) => self.lookup_var(name),
            Expr::List(items) => {
                if items.is_empty() {
                    return Ok(Type::List(Box::new(Type::Any)));
                }

                let first_ty = self.check_expr(&items[0])?;

                for item in &items[1..] {
                    let ty = self.check_expr(item)?;
                    if !self.types_compatible(&ty, &first_ty) {
                        self.error(format!(
                            "List element type mismatch: expected {:?}, got {:?}",
                            first_ty, ty
                        ));
                    }
                }

                Ok(Type::List(Box::new(first_ty)))
            }
            Expr::BinOp { left, op, right } => {
                let left_ty = self.check_expr(left)?;
                let right_ty = self.check_expr(right)?;

                match op.as_str() {
                    "+" | "-" | "*" | "/" | "%" => {
                        if !self.is_number_or_any(&left_ty) {
                            return Err(TypeError {
                                message: format!("Left operand of {} must be number, got {:?}", op, left_ty),
                            });
                        }
                        if !self.is_number_or_any(&right_ty) {
                            return Err(TypeError {
                                message: format!("Right operand of {} must be number, got {:?}", op, right_ty),
                            });
                        }
                        Ok(Type::Number)
                    }
                    "<" | "<=" | ">" | ">=" => {
                        if !self.is_number_or_any(&left_ty) {
                            return Err(TypeError {
                                message: format!("Left operand of {} must be number, got {:?}", op, left_ty),
                            });
                        }
                        if !self.is_number_or_any(&right_ty) {
                            return Err(TypeError {
                                message: format!("Right operand of {} must be number, got {:?}", op, right_ty),
                            });
                        }
                        Ok(Type::Bool)
                    }
                    "==" | "!=" => {
                        if !self.types_compatible(&left_ty, &right_ty) {
                            return Err(TypeError {
                                message: format!(
                                    "Type mismatch in comparison: {:?} {} {:?}",
                                    left_ty, op, right_ty
                                ),
                            });
                        }
                        Ok(Type::Bool)
                    }
                    "&&" | "||" => {
                        if !self.is_bool_or_any(&left_ty) {
                            return Err(TypeError {
                                message: format!("Left operand of {} must be bool, got {:?}", op, left_ty),
                            });
                        }
                        if !self.is_bool_or_any(&right_ty) {
                            return Err(TypeError {
                                message: format!("Right operand of {} must be bool, got {:?}", op, right_ty),
                            });
                        }
                        Ok(Type::Bool)
                    }
                    _ => Err(TypeError {
                        message: format!("Unknown binary operator: {}", op),
                    }),
                }
            }
            Expr::UnOp { op, operand } => {
                let operand_ty = self.check_expr(operand)?;
                match op.as_str() {
                    "-" => {
                        if !self.is_number_or_any(&operand_ty) {
                            return Err(TypeError {
                                message: format!("Unary {} requires number, got {:?}", op, operand_ty),
                            });
                        }
                        Ok(Type::Number)
                    }
                    "!" => {
                        if !self.is_bool_or_any(&operand_ty) {
                            return Err(TypeError {
                                message: format!("Unary {} requires bool, got {:?}", op, operand_ty),
                            });
                        }
                        Ok(Type::Bool)
                    }
                    _ => Err(TypeError {
                        message: format!("Unknown unary operator: {}", op),
                    }),
                }
            }
            Expr::Call { func, args } => {
                // Check built-in functions
                match func.as_str() {
                    "out" => {
                        for arg in args {
                            let _ = self.check_expr(arg);
                        }
                        Ok(Type::Nil)
                    }
                    "len" => {
                        if args.len() != 1 {
                            return Err(TypeError {
                                message: format!("len expects 1 argument, got {}", args.len()),
                            });
                        }
                        let arg_ty = self.check_expr(&args[0])?;
                        match arg_ty {
                            Type::String | Type::List(_) => Ok(Type::Number),
                            Type::Any => Ok(Type::Number),
                            _ => Err(TypeError {
                                message: "len requires string or list".to_string(),
                            }),
                        }
                    }
                    "type" => {
                        if args.len() != 1 {
                            return Err(TypeError {
                                message: format!("type expects 1 argument, got {}", args.len()),
                            });
                        }
                        let _ = self.check_expr(&args[0])?;
                        Ok(Type::String)
                    }
                    _ => {
                        // Check user-defined functions
                        if let Some(sig) = self.functions.get(func).cloned() {
                            if sig.params.len() != args.len() {
                                return Err(TypeError {
                                    message: format!(
                                        "Function {} expects {} arguments, got {}",
                                        func,
                                        sig.params.len(),
                                        args.len()
                                    ),
                                });
                            }

                            for (expected_param, arg) in sig.params.iter().zip(args.iter()) {
                                let arg_ty = self.check_expr(arg)?;
                                if !self.types_compatible(&arg_ty, &expected_param.1) {
                                    return Err(TypeError {
                                        message: format!(
                                            "Argument type mismatch for {}: expected {:?}, got {:?}",
                                            func, expected_param.1, arg_ty
                                        ),
                                    });
                                }
                            }

                            Ok(sig.return_type.clone())
                        } else {
                            Err(TypeError {
                                message: format!("Undefined function: {}", func),
                            })
                        }
                    }
                }
            }
            Expr::Index { object, index } => {
                let obj_ty = self.check_expr(object)?;
                let idx_ty = self.check_expr(index)?;

                if !self.is_number_or_any(&idx_ty) {
                    return Err(TypeError {
                        message: "List index must be number".to_string(),
                    });
                }

                match obj_ty {
                    Type::List(elem_ty) => Ok(*elem_ty),
                    Type::Any => Ok(Type::Any),
                    _ => Err(TypeError {
                        message: format!("Cannot index {:?}", obj_ty),
                    }),
                }
            }
        }
    }

    fn lookup_var(&self, name: &str) -> Result<Type, TypeError> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Ok(ty.clone());
            }
        }
        Err(TypeError {
            message: format!("Undefined variable: {}", name),
        })
    }

    fn define_var(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn error(&mut self, message: String) {
        self.errors.push(TypeError { message });
    }

    fn is_number_or_any(&self, ty: &Type) -> bool {
        matches!(ty, Type::Number | Type::Any)
    }

    fn is_bool_or_any(&self, ty: &Type) -> bool {
        matches!(ty, Type::Bool | Type::Any)
    }

    fn types_compatible(&self, actual: &Type, expected: &Type) -> bool {
        matches!(actual, Type::Any) || matches!(expected, Type::Any) || actual == expected
    }
}
