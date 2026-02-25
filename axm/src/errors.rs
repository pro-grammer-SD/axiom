/// Axiom error types — Final Maturation
/// Supports lexer, parser, runtime, type, and diagnostic errors with Miette integration

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub source_id: u32,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(source_id: u32, start: usize, end: usize) -> Self {
        Span { source_id, start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            source_id: self.source_id,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Span { source_id: 0, start: 0, end: 0 }
    }
}

// ---------------------------------------------------------------------------
// Lexer errors
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum LexerError {
    UnexpectedCharacter { ch: char, span: Span },
    UnterminatedString { span: Span },
    InvalidNumber { text: String, span: Span },
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::UnexpectedCharacter { ch, .. } => {
                write!(f, "Unexpected character: '{}'", ch)
            }
            LexerError::UnterminatedString { .. } => {
                write!(f, "Unterminated string literal")
            }
            LexerError::InvalidNumber { text, .. } => {
                write!(f, "Invalid number: '{}'", text)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Parser errors
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum ParserError {
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },
    InvalidSyntax {
        context: String,
        span: Span,
    },
    UnexpectedEof {
        context: String,
        span: Span,
    },
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::UnexpectedToken {
                expected, found, ..
            } => {
                write!(f, "Expected {}, found {}", expected, found)
            }
            ParserError::InvalidSyntax { context, .. } => {
                write!(f, "Invalid syntax in {}", context)
            }
            ParserError::UnexpectedEof { context, .. } => {
                write!(f, "Unexpected end of file in {}", context)
            }
        }
    }
}

impl std::error::Error for ParserError {}

// ---------------------------------------------------------------------------
// Type errors
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum TypeError {
    TypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },
    UndefinedVariable {
        name: String,
        span: Span,
    },
    UndefinedFunction {
        name: String,
        span: Span,
    },
    UndefinedClass {
        name: String,
        span: Span,
    },
    UndefinedMethod {
        class_name: String,
        method_name: String,
        span: Span,
    },
    ArityMismatch {
        expected: usize,
        found: usize,
        span: Span,
    },
    DuplicateDefinition {
        name: String,
        span: Span,
    },
    InvalidOperation {
        message: String,
        span: Span,
    },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::TypeMismatch { expected, found, .. } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            TypeError::UndefinedVariable { name, .. } => {
                write!(f, "Undefined variable: '{}'", name)
            }
            TypeError::UndefinedFunction { name, .. } => {
                write!(f, "Undefined function: '{}'", name)
            }
            TypeError::UndefinedClass { name, .. } => {
                write!(f, "Undefined class: '{}'", name)
            }
            TypeError::UndefinedMethod {
                class_name,
                method_name,
                ..
            } => {
                write!(f, "Undefined method: '{}' on class '{}'", method_name, class_name)
            }
            TypeError::ArityMismatch { expected, found, .. } => {
                write!(f, "Arity mismatch: expected {} args, found {}", expected, found)
            }
            TypeError::DuplicateDefinition { name, .. } => {
                write!(f, "Duplicate definition: '{}'", name)
            }
            TypeError::InvalidOperation { message, .. } => {
                write!(f, "Invalid operation: {}", message)
            }
        }
    }
}

impl std::error::Error for TypeError {}

// ---------------------------------------------------------------------------
// Runtime errors
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum RuntimeError {
    UndefinedVariable { name: String, span: Span },
    UndefinedFunction { name: String, span: Span },
    UndefinedClass { name: String },
    UndefinedMethod { class_name: String, method_name: String },
    TypeMismatch { expected: String, found: String, span: Span },
    ArityMismatch { expected: usize, found: usize },
    IndexOutOfBounds { index: i64, length: usize },
    DivisionByZero { span: Span },
    ImportError { module: String, message: String },
    GenericError { message: String, span: Span },
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::UndefinedVariable { name, .. } => {
                write!(f, "Undefined variable: '{}'", name)
            }
            RuntimeError::UndefinedFunction { name, .. } => {
                write!(f, "Undefined function: '{}'", name)
            }
            RuntimeError::UndefinedClass { name } => {
                write!(f, "Undefined class: '{}'", name)
            }
            RuntimeError::UndefinedMethod { class_name, method_name } => {
                write!(f, "Undefined method '{}' on '{}'", method_name, class_name)
            }
            RuntimeError::TypeMismatch { expected, found, .. } => {
                write!(f, "Type mismatch: expected {}, found {}", expected, found)
            }
            RuntimeError::ArityMismatch { expected, found } => {
                write!(f, "Expected {} arguments, got {}", expected, found)
            }
            RuntimeError::IndexOutOfBounds { index, length } => {
                write!(f, "Index {} out of bounds for length {}", index, length)
            }
            RuntimeError::DivisionByZero { .. } => {
                write!(f, "Division by zero")
            }
            RuntimeError::ImportError { module, message } => {
                write!(f, "Import error for '{}': {}", module, message)
            }
            RuntimeError::GenericError { message, .. } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl std::error::Error for RuntimeError {}

// ---------------------------------------------------------------------------
// Diagnostic — structured error for chk
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.level {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Info => "info",
        };
        write!(f, "[{}] {}", prefix, self.message)?;
        if let Some(ref hint) = self.hint {
            write!(f, "\n  hint: {}", hint)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CompileError — top-level wrapper
// ---------------------------------------------------------------------------
#[derive(Debug)]
pub enum CompileError {
    Lexer(LexerError),
    Parser(ParserError),
    Type(TypeError),
    Runtime(RuntimeError),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Lexer(e) => write!(f, "Lexer error: {}", e),
            CompileError::Parser(e) => write!(f, "Parser error: {}", e),
            CompileError::Type(e) => write!(f, "Type error: {}", e),
            CompileError::Runtime(e) => write!(f, "Runtime error: {}", e),
        }
    }
}

impl std::error::Error for CompileError {}
