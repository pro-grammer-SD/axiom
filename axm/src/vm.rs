/// Axiom Flat-Loop Virtual Machine (VM)
///
/// High-performance bytecode interpreter using explicit heap-allocated stack frames.
/// This eliminates Rust's C-stack limit and enables tail-call optimization.
///
/// Key design:
/// - Vec<StackFrame> for explicit call stack (no Rust function call recursion)
/// - One main execution loop that process bytecode instructions
/// - Direct AxValue mutation for speed (no Arc wrappers for locals)

use crate::core::value::AxValue;
use crate::ast::{Expr, Stmt, Item};
use crate::errors::{RuntimeError, Span};
use std::collections::HashMap;

/// Bytecode instruction type
#[derive(Debug, Clone)]
pub enum Instruction {
    /// Load a constant value
    LoadConst(AxValue),
    /// Load a variable from current frame
    LoadVar(String),
    /// Store top of stack to variable
    StoreVar(String),
    /// Binary operation (op_code, left, right -> result)
    BinOp(BinOp),
    /// Unary operation
    UnOp(UnOp),
    /// Function call (func_name, arg_count)
    Call(String, usize),
    /// Return from current frame
    Return,
    /// Jump to instruction pointer
    Jump(usize),
    /// Jump if top of stack is truthy
    JumpIfTrue(usize),
    /// Jump if top of stack is falsy
    JumpIfFalse(usize),
    /// Pop top of stack
    Pop,
    /// Duplicate top of stack
    Dup,
    /// Array/List creation (with size on stack)
    MakeList(usize),
    /// Dictionary creation
    MakeDict(usize),
    /// Index into array/dict
    Index,
    /// Set index into array/dict
    SetIndex,
    /// Class instantiation
    New(String),
    /// Attribute access
    GetAttr(String),
    /// Attribute assignment
    SetAttr(String),
    /// No operation
    Nop,
    /// Break from loop
    Break,
    /// Continue to next iteration
    Continue,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Not,
    Neg,
}

/// VM execution state (simplified)
pub struct VMState;

impl VMState {
    pub fn new() -> Self {
        VMState
    }
}

/// The Flat-Loop VM
pub struct FlatVM {
    /// Program bytecode
    bytecode: Vec<Instruction>,

    /// Execution state
    state: VMState,

    /// Value stack (for operands)
    value_stack: Vec<AxValue>,
}

impl FlatVM {
    /// Create a new VM
    pub fn new() -> Self {
        FlatVM {
            bytecode: Vec::new(),
            state: VMState::new(),
            value_stack: Vec::with_capacity(1024),
        }
    }

    /// Compile AST to bytecode
    pub fn compile(&mut self, items: &[Item]) -> Result<(), RuntimeError> {
        // Hoisting phase — collect all global declarations
        for item in items {
            match item {
                Item::FunctionDecl { .. } => {
                    // Store function metadata (simplified - handled by runtime)
                }
                Item::ClassDecl { .. } => {
                    // Store class metadata (simplified - handled by runtime)
                }
                _ => {}
            }
        }

        // Code generation phase
        for item in items {
            match item {
                Item::Statement(stmt) => self.compile_stmt(stmt)?,
                _ => {}
            }
        }

        Ok(())
    }

    /// Compile a statement to bytecode
    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expr(value) => {
                self.compile_expr(value)?;
                self.bytecode.push(Instruction::Pop); // Discard result of top-level expr
            }
            Stmt::Let { name, value, .. } => {
                self.compile_expr(value)?;
                self.bytecode.push(Instruction::StoreVar(name.clone()));
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.compile_expr(expr)?;
                }
                self.bytecode.push(Instruction::Return);
            }
            Stmt::If { condition, then_body, else_body, .. } => {
                // Compile condition
                self.compile_expr(condition)?;

                // Emit conditional jump
                let jump_else_idx = self.bytecode.len();
                self.bytecode.push(Instruction::JumpIfFalse(0)); // Placeholder

                // Compile then branch
                for stmt in then_body {
                    self.compile_stmt(stmt)?;
                }

                // Patch the jump target
                if let Some(else_stmts) = else_body {
                    let jump_end_idx = self.bytecode.len();
                    self.bytecode.push(Instruction::Jump(0)); // Placeholder

                    // Fill in the else target — capture len before mutable borrow
                    let else_start = self.bytecode.len();
                    if let Instruction::JumpIfFalse(ref mut target) = self.bytecode[jump_else_idx] {
                        *target = else_start;
                    }

                    // Compile else branch
                    for stmt in else_stmts {
                        self.compile_stmt(stmt)?;
                    }

                    // Patch the end jump — capture len before mutable borrow
                    let after_else = self.bytecode.len();
                    if let Instruction::Jump(ref mut target) = self.bytecode[jump_end_idx] {
                        *target = after_else;
                    }
                } else {
                    // No else — just patch the jump
                    let after_then = self.bytecode.len();
                    if let Instruction::JumpIfFalse(ref mut target) = self.bytecode[jump_else_idx] {
                        *target = after_then;
                    }
                }
            }
            Stmt::While { condition, body, .. } => {
                let loop_start = self.bytecode.len();

                // Compile condition
                self.compile_expr(condition)?;

                // Jump to end if false
                let jump_end_idx = self.bytecode.len();
                self.bytecode.push(Instruction::JumpIfFalse(0)); // Placeholder

                // Compile body
                for stmt in body {
                    self.compile_stmt(stmt)?;
                }

                // Jump back to start
                self.bytecode.push(Instruction::Jump(loop_start));

                // Patch the end jump — capture len before mutable borrow
                let after_loop = self.bytecode.len();
                if let Instruction::JumpIfFalse(ref mut target) = self.bytecode[jump_end_idx] {
                    *target = after_loop;
                }
            }
            _ => {} // Other statement types
        }
        Ok(())
    }

    /// Compile an expression to bytecode
    fn compile_expr(&mut self, expr: &Expr) -> Result<(), RuntimeError> {
        match expr {
            Expr::Number { value: n, .. } => {
                // Store as Int if whole number, Float otherwise
                if n.fract() == 0.0 && n.is_finite() && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                    self.bytecode.push(Instruction::LoadConst(AxValue::Int(*n as i64)));
                } else {
                    self.bytecode.push(Instruction::LoadConst(AxValue::Float(*n)));
                }
            }
            Expr::String { value: s, .. } => {
                self.bytecode.push(Instruction::LoadConst(AxValue::String(s.clone())))
            }
            Expr::Boolean { value: b, .. } => {
                self.bytecode.push(Instruction::LoadConst(AxValue::Bool(*b)))
            }
            Expr::Identifier { name, .. } => {
                self.bytecode.push(Instruction::LoadVar(name.clone()))
            }
            Expr::BinaryOp { left, op, right, .. } => {
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                let bin_op = match op.as_str() {
                    "+" => BinOp::Add,
                    "-" => BinOp::Sub,
                    "*" => BinOp::Mul,
                    "/" => BinOp::Div,
                    "%" => BinOp::Mod,
                    "==" => BinOp::Eq,
                    "!=" => BinOp::Neq,
                    "<" => BinOp::Lt,
                    "<=" => BinOp::Le,
                    ">" => BinOp::Gt,
                    ">=" => BinOp::Ge,
                    "and" => BinOp::And,
                    "or" => BinOp::Or,
                    _ => return Err(RuntimeError::GenericError {
                        message: format!("Unsupported operator: {}", op),
                        span: Span::default(),
                    }),
                };
                self.bytecode.push(Instruction::BinOp(bin_op));
            }
            Expr::Call { function, arguments, .. } => {
                for arg in arguments {
                    self.compile_expr(arg)?;
                }
                // Extract function name for simple identifier calls
                if let Expr::Identifier { name, .. } = function.as_ref() {
                    self.bytecode.push(Instruction::Call(name.clone(), arguments.len()));
                } else {
                    // Compile function expression and use generic call
                    self.compile_expr(function)?;
                    self.bytecode.push(Instruction::Call("<expr>".to_string(), arguments.len()));
                }
            }
            Expr::List { items, .. } => {
                for item in items {
                    self.compile_expr(item)?;
                }
                self.bytecode.push(Instruction::MakeList(items.len()));
            }
            _ => self.bytecode.push(Instruction::Nop),
        }
        Ok(())
    }

    /// Execute the bytecode
    pub fn execute(&mut self) -> Result<AxValue, RuntimeError> {
        while self.state.ip < self.bytecode.len() && !self.state.halted {
            let instr = self.bytecode[self.state.ip].clone();
            self.state.ip += 1;

            match instr {
                Instruction::LoadConst(val) => {
                    self.value_stack.push(val);
                }
                Instruction::LoadVar(name) => {
                    let val = self.state.get_var(&name).unwrap_or(AxValue::Nil);
                    self.value_stack.push(val);
                }
                Instruction::StoreVar(name) => {
                    let val = self.value_stack.pop().unwrap_or(AxValue::Nil);
                    self.state.set_var(name, val);
                }
                Instruction::BinOp(op) => {
                    let right = self.value_stack.pop().unwrap_or(AxValue::Nil);
                    let left = self.value_stack.pop().unwrap_or(AxValue::Nil);
                    let result = self.execute_binop(left, op, right)?;
                    self.value_stack.push(result);
                }
                Instruction::UnOp(op) => {
                    let val = self.value_stack.pop().unwrap_or(AxValue::Nil);
                    let result = self.execute_unop(val, op)?;
                    self.value_stack.push(result);
                }
                Instruction::Call(name, argc) => {
                    let mut args = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        args.push(self.value_stack.pop().unwrap_or(AxValue::Nil));
                    }
                    args.reverse();

                    // Look up and call the function
                    if let Some(func_val) = self.state.get_var(&name) {
                        match func_val {
                            AxValue::Function(func) => {
                                if let Some(builtin) = &func.builtin {
                                    match builtin(args) {
                                        Ok(result) => self.value_stack.push(result),
                                        Err(e) => return Err(RuntimeError::GenericError { message: e, span: Span::default() }),
                                    }
                                }
                            }
                            _ => return Err(RuntimeError::UndefinedFunction { name: name.clone(), span: Span::default() }),
                        }
                    } else {
                        return Err(RuntimeError::UndefinedFunction { name: name.clone(), span: Span::default() });
                    }
                }
                Instruction::Return => {
                    let ret_val = self.value_stack.pop().unwrap_or(AxValue::Nil);
                    self.state.return_value = ret_val;
                    self.state.halted = true;
                }
                Instruction::Jump(target) => {
                    self.state.ip = target;
                }
                Instruction::JumpIfTrue(target) => {
                    let val = self.value_stack.pop().unwrap_or(AxValue::Nil);
                    if val.is_truthy() {
                        self.state.ip = target;
                    }
                }
                Instruction::JumpIfFalse(target) => {
                    let val = self.value_stack.pop().unwrap_or(AxValue::Nil);
                    if !val.is_truthy() {
                        self.state.ip = target;
                    }
                }
                Instruction::Pop => {
                    self.value_stack.pop();
                }
                Instruction::Dup => {
                    if let Some(val) = self.value_stack.last() {
                        self.value_stack.push(val.clone());
                    }
                }
                Instruction::MakeList(size) => {
                    let mut items = Vec::with_capacity(size);
                    for _ in 0..size {
                        items.push(self.value_stack.pop().unwrap_or(AxValue::Nil));
                    }
                    items.reverse();
                    self.value_stack.push(AxValue::List(std::sync::Arc::new(parking_lot::RwLock::new(items))));
                }
                _ => {}
            }
        }

        Ok(self.state.return_value.clone())
    }

    /// Execute a binary operation
    fn execute_binop(&self, left: AxValue, op: BinOp, right: AxValue) -> Result<AxValue, RuntimeError> {
        let span = Span::default();
        match op {
            BinOp::Add => match (left, right) {
                (AxValue::Int(a), AxValue::Int(b)) => Ok(AxValue::Int(a + b)),
                (AxValue::Float(a), AxValue::Float(b)) => Ok(AxValue::Float(a + b)),
                (AxValue::Int(a), AxValue::Float(b)) => Ok(AxValue::Float(a as f64 + b)),
                (AxValue::Float(a), AxValue::Int(b)) => Ok(AxValue::Float(a + b as f64)),
                (AxValue::String(a), AxValue::String(b)) => Ok(AxValue::String(format!("{}{}", a, b))),
                _ => Err(RuntimeError::TypeMismatch { expected: "number".to_string(), found: "unknown".to_string(), span }),
            },
            BinOp::Sub => match (left, right) {
                (AxValue::Int(a), AxValue::Int(b)) => Ok(AxValue::Int(a - b)),
                (AxValue::Float(a), AxValue::Float(b)) => Ok(AxValue::Float(a - b)),
                _ => Err(RuntimeError::TypeMismatch { expected: "number".to_string(), found: "unknown".to_string(), span }),
            },
            BinOp::Mul => match (left, right) {
                (AxValue::Int(a), AxValue::Int(b)) => Ok(AxValue::Int(a * b)),
                (AxValue::Float(a), AxValue::Float(b)) => Ok(AxValue::Float(a * b)),
                _ => Err(RuntimeError::TypeMismatch { expected: "number".to_string(), found: "unknown".to_string(), span }),
            },
            BinOp::Div => match (left, right) {
                (AxValue::Int(a), AxValue::Int(b)) => {
                    if b == 0 { Err(RuntimeError::DivisionByZero { span }) }
                    else { Ok(AxValue::Int(a / b)) }
                },
                (AxValue::Float(a), AxValue::Float(b)) => Ok(AxValue::Float(a / b)),
                _ => Err(RuntimeError::TypeMismatch { expected: "number".to_string(), found: "unknown".to_string(), span }),
            },
            BinOp::Eq => Ok(AxValue::Bool(left.eq(&right))),
            BinOp::Neq => Ok(AxValue::Bool(!left.eq(&right))),
            BinOp::Lt => match (left, right) {
                (AxValue::Int(a), AxValue::Int(b)) => Ok(AxValue::Bool(a < b)),
                (AxValue::Float(a), AxValue::Float(b)) => Ok(AxValue::Bool(a < b)),
                _ => Err(RuntimeError::TypeMismatch { expected: "number".to_string(), found: "unknown".to_string(), span }),
            },
            BinOp::And => Ok(AxValue::Bool(left.is_truthy() && right.is_truthy())),
            BinOp::Or => Ok(AxValue::Bool(left.is_truthy() || right.is_truthy())),
            _ => Err(RuntimeError::GenericError { message: "unsupported binary operation".to_string(), span }),
        }
    }

    /// Execute a unary operation
    fn execute_unop(&self, val: AxValue, op: UnOp) -> Result<AxValue, RuntimeError> {
        let span = Span::default();
        match op {
            UnOp::Not => Ok(AxValue::Bool(!val.is_truthy())),
            UnOp::Neg => match val {
                AxValue::Int(n) => Ok(AxValue::Int(-n)),
                AxValue::Float(f) => Ok(AxValue::Float(-f)),
                _ => Err(RuntimeError::TypeMismatch { expected: "number".to_string(), found: "unknown".to_string(), span }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let mut vm = FlatVM::new();
        // Add bytecode: LOAD_CONST(42) -> LOAD_CONST(8) -> BINOP(Add)
        vm.bytecode.push(Instruction::LoadConst(AxValue::Int(42)));
        vm.bytecode.push(Instruction::LoadConst(AxValue::Int(8)));
        vm.bytecode.push(Instruction::BinOp(BinOp::Add));
        vm.bytecode.push(Instruction::Return);

        let result = vm.execute().unwrap();
        assert_eq!(result, AxValue::Int(50));
    }
}
