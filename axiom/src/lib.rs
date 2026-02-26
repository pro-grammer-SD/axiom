/// Axiom Language Library
///
/// Module layout:
///   - conf         — runtime configuration (toggles, ~/.axiom/conf.txt)
///   - nanbox       — NaN-boxed value representation
///   - bytecode     — instruction set, Proto, encoding/decoding
///   - compiler     — AST → bytecode compiler
///   - optimizer    — static bytecode optimisation pipeline
///   - inline_cache — property / call inline caches + shape system
///   - gc           — generational garbage collector
///   - profiler     — opcode counters, hot-loop detection, flame graph
///   - vm           — register-VM interpreter
///   - runtime      — high-level Runtime (conf + VM wiring)
///   - ast / lexer / parser — front-end
///   - chk          — semantic analysis
///   - fmt          — source formatter
///   - intrinsics   — built-in functions
///   - jit          — experimental trace-JIT stub
///   - loader       — module resolution + file loading
///   - module_loader— import / require logic
///   - build_system — axiom build orchestration
///   - pkg          — Axiomide package manager
///   - core         — value types (AxValue) and OOP helpers

// ── Core VM modules ──────────────────────────────────────────────────────────
pub mod conf;
pub mod nanbox;
pub mod bytecode;
pub mod compiler;
pub mod optimizer;
pub mod inline_cache;
pub mod gc;
pub mod profiler;

// ── Front-end / language ──────────────────────────────────────────────────────
pub mod ast;
pub mod lexer;
pub mod parser;
pub mod chk;
pub mod fmt;
pub mod errors;

// ── Runtime, system & utilities ──────────────────────────────────────────────
pub mod vm;
pub mod runtime;
pub mod intrinsics;
pub mod jit;
pub mod loader;
pub mod module_loader;
pub mod core;
pub mod build_system;
pub mod pkg;

// ── Re-exports for convenience ────────────────────────────────────────────────
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
