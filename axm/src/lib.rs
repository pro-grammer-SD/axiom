/// Axiom Language Library

pub mod ast;
pub mod build_system;
pub mod chk;
pub mod core;
pub mod errors;
pub mod fmt;
pub mod jit;
pub mod lexer;
pub mod loader;
pub mod parser;
pub mod pkg;
pub mod runtime;

// Re-exports for convenience
pub use ast::Item;
pub use chk::SemanticAnalyzer;
pub use core::value::AxValue;
pub use errors::{CompileError, Span};
pub use fmt::format_source;
pub use lexer::Lexer;
pub use loader::{resolve_module_path, load_local_module};
pub use parser::Parser;
pub use runtime::Runtime;
