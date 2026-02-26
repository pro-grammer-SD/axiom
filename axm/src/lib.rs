/// Axiom Language — Library Root
///
/// Module map
/// ──────────
///   Front-end
///     ast           — Abstract Syntax Tree node types
///     lexer         — Tokeniser
///     parser        — Recursive-descent parser → AST
///     chk           — Semantic analyser (symbol resolution, type inference)
///     fmt           — Source formatter
///     errors        — Diagnostic / error types with source spans
///
///   Compilation
///     bytecode      — Instruction set (Op), Proto, Instr encoding/decoding
///     compiler      — AST → register bytecode (compile_program)
///     optimizer     — Peephole + constant-fold passes on Proto
///
///   Execution
///     vm_core       — Register-based bytecode VM (Val, VmCore)
///     runtime       — High-level Runtime: compile → VM → tree-walk fallback
///
///   Runtime support
///     nanbox        — NaN-boxed 64-bit value representation
///     inline_cache  — Polymorphic inline caches + shape system
///     gc            — Generational garbage collector
///     profiler      — Opcode counters, hot-loop detection, flame graph
///     conf          — Runtime configuration (toggles, ~/.axiom/conf.txt)
///     intrinsics    — Statically-linked standard library (23 modules)
///     jit           — Experimental trace-JIT stub
///     loader        — Module file resolution + loading
///
///   Packaging
///     pkg           — Axiomite package manager (Axiomite.toml, deps)
///     core          — AxValue, AxCallable, AxClass, AxInstance

// ── Compilation pipeline ──────────────────────────────────────────────────────
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod chk;
pub mod fmt;
pub mod errors;

// ── Bytecode layer ────────────────────────────────────────────────────────────
pub mod bytecode;
pub mod compiler;
pub mod optimizer;

// ── Execution ─────────────────────────────────────────────────────────────────
pub mod vm_core;
pub mod runtime;

// ── Runtime support ───────────────────────────────────────────────────────────
pub mod nanbox;
pub mod inline_cache;
pub mod gc;
pub mod profiler;
pub mod conf;
pub mod intrinsics;
pub mod jit;
pub mod loader;

// ── Core value types ─────────────────────────────────────────────────────────
pub mod core;

// ── Package management ────────────────────────────────────────────────────────
pub mod pkg;

// ── Public re-exports ─────────────────────────────────────────────────────────
pub use ast::Item;
pub use chk::SemanticAnalyzer;
pub use conf::AxConf;
pub use core::value::AxValue;
pub use errors::{CompileError, Span};
pub use fmt::format_source;
pub use lexer::Lexer;
pub use loader::{resolve_module_path, load_local_module};
pub use nanbox::NanVal;
pub use parser::Parser;
pub use runtime::Runtime;
