/// Complete AST definitions for Axiom language — Final Maturation
use crate::errors::Span;

// ---------------------------------------------------------------------------
// Top-level items
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum Item {
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        span: Span,
    },
    ClassDecl {
        name: String,
        parent: Option<String>,
        body: Vec<ClassMember>,
        span: Span,
    },
    EnumDecl {
        name: String,
        variants: Vec<EnumVariant>,
        span: Span,
    },
    StdImport {
        name: String,
        span: Span,
    },
    LocImport {
        name: String,
        span: Span,
    },
    LibDecl {
        name: String,
        span: Span,
    },
    Statement(Stmt),
}

// ---------------------------------------------------------------------------
// Class members
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum ClassMember {
    Method {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
        span: Span,
    },
    Field {
        name: String,
        default: Option<Expr>,
        span: Span,
    },
}

// ---------------------------------------------------------------------------
// Enum variants
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub has_data: bool, // true if Variant(inner)
    pub span: Span,
}

// ---------------------------------------------------------------------------
// Match arm
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum MatchPattern {
    Identifier(String),
    EnumVariant { enum_name: Option<String>, variant: String, binding: Option<String> },
    Literal(Expr),
    Wildcard,
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Let {
        name: String,
        value: Expr,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    If {
        condition: Expr,
        then_body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    For {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    Block(Vec<Stmt>),
    GoSpawn {
        body: Vec<Stmt>,
        span: Span,
    },
    Match {
        expr: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Out {
        arguments: Vec<Expr>,
        span: Span,
    },
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------
#[derive(Debug, Clone)]
pub enum Expr {
    Number { value: f64, span: Span },
    String { value: String, span: Span },
    Boolean { value: bool, span: Span },
    Identifier { name: String, span: Span },
    SelfRef { span: Span },
    List { items: Vec<Expr>, span: Span },
    BinaryOp {
        left: Box<Expr>,
        op: String,
        right: Box<Expr>,
        span: Span,
    },
    UnaryOp {
        op: String,
        operand: Box<Expr>,
        span: Span,
    },
    Call {
        function: Box<Expr>,
        arguments: Vec<Expr>,
        span: Span,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        arguments: Vec<Expr>,
        span: Span,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
        span: Span,
    },
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    New {
        class_name: String,
        arguments: Vec<Expr>,
        span: Span,
    },
    InterpolatedString {
        parts: Vec<StringPart>,
        span: Span,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
        span: Span,
    },
}

/// Parts of an interpolated string: literal text or embedded expression.
#[derive(Debug, Clone)]
pub enum StringPart {
    Literal(String),
    Expr(Expr),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Number { span, .. }
            | Expr::String { span, .. }
            | Expr::Boolean { span, .. }
            | Expr::Identifier { span, .. }
            | Expr::SelfRef { span }
            | Expr::List { span, .. }
            | Expr::BinaryOp { span, .. }
            | Expr::UnaryOp { span, .. }
            | Expr::Call { span, .. }
            | Expr::MethodCall { span, .. }
            | Expr::Index { span, .. }
            | Expr::MemberAccess { span, .. }
            | Expr::Assign { span, .. }
            | Expr::New { span, .. }
            | Expr::InterpolatedString { span, .. }
            | Expr::Lambda { span, .. } => *span,
        }
    }
}
