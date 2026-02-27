/// AXIOM DIAGNOSTIC ENGINE V4.0 — rustc-grade terminal output
///
/// FEATURES
/// ────────
/// • Source context: shows 2 lines above + error line + caret (^^^^) underneath
/// • Row / column / line coordinates pinpointed from byte spans
/// • Levenshtein spell-check for AXM_200 (undefined identifier → "did you mean X?")
/// • Every error routed through AxiomDiagnostic → miette graphical renderer
/// • AXM_100–699 full taxonomy (see ErrorCode enum)
///
/// OUTPUT EXAMPLE
/// ──────────────
///   × [AXM_402] Attempt to call nil value
///    ╭─[stdlib_demo.ax:12:16]
///  11 │     fn inner(x) {
///  12 │         return adder(x)
///    ·                ^^^^^
///    ╰─
///   help: Ensure the identifier is defined before use. Closures capture by value.

use std::fmt;
use miette::{Diagnostic, SourceSpan, NamedSource};
use thiserror::Error;
use crate::errors::{Span, RuntimeError};

// ═══════════════════════════════════════════════════════════════════════════
// Error Code Taxonomy (AXM_100-699)
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // AXM_100-199: Lexical
    UnexpectedToken         = 101,
    UnterminatedString      = 102,
    InvalidNumber           = 103,
    InvalidEscape           = 104,
    UnexpectedEof           = 105,
    // AXM_200-299: Semantic
    UndefinedIdentifier     = 200,
    UndefinedVariable       = 201,
    ArityMismatch           = 202,
    TypeMismatch            = 203,
    DuplicateDeclaration    = 204,
    ReservedKeyword         = 205,
    MissingReturn           = 206,
    UnreachableCode         = 207,
    CircularDependency      = 208,
    // AXM_300-399: Compiler/Quickening
    SpecializationMismatch  = 301,
    UnsupportedOperation    = 302,
    RegisterAllocFailed     = 303,
    JumpOutOfRange          = 304,
    TooManyLocals           = 305,
    // AXM_400-499: Runtime
    NotCallable             = 401,
    NilCall                 = 402,
    DivisionByZero          = 403,
    IndexOutOfBounds        = 404,
    NilAccess               = 405,
    BinaryOpTypeError       = 406,
    UnaryOpTypeError        = 407,
    StackOverflow           = 408,
    HeapExhausted           = 409,
    InvalidConversion       = 410,
    // AXM_500-599: System
    IoError                 = 501,
    UsbError                = 502,
    NetworkError            = 503,
    GcPressure              = 504,
    ResourceLeak            = 505,
    // AXM_600-699: Module
    ModuleNotFound          = 601,
    VersionConflict         = 602,
    CircularImport          = 603,
    LoadFailed              = 604,
    ExportNotFound          = 605,
}

impl ErrorCode {
    pub fn as_u32(self) -> u32 { self as u32 }

    pub fn prefix(self) -> String {
        format!("[AXM_{:03}]", self as u32)
    }

    pub fn summary(self) -> &'static str {
        match self {
            Self::UnexpectedToken          => "Unrecognized character in source",
            Self::UnterminatedString       => "Unterminated string literal",
            Self::InvalidNumber            => "Invalid numeric literal",
            Self::InvalidEscape            => "Invalid escape sequence in string",
            Self::UnexpectedEof            => "Unexpected end of file",
            Self::UndefinedIdentifier      => "Undefined identifier",
            Self::UndefinedVariable        => "Undefined variable",
            Self::ArityMismatch            => "Argument count mismatch",
            Self::TypeMismatch             => "Type mismatch in operation",
            Self::DuplicateDeclaration     => "Duplicate declaration",
            Self::ReservedKeyword          => "Reserved keyword used as identifier",
            Self::MissingReturn            => "Missing return statement",
            Self::UnreachableCode          => "Unreachable code after return",
            Self::CircularDependency       => "Circular dependency detected",
            Self::SpecializationMismatch   => "Type specialization mismatch",
            Self::UnsupportedOperation     => "Operation not supported for this type",
            Self::RegisterAllocFailed      => "Register allocation failure",
            Self::JumpOutOfRange           => "Jump offset out of encodable range",
            Self::TooManyLocals            => "Too many local variables (limit: 255)",
            Self::NotCallable              => "Attempt to call non-function value",
            Self::NilCall                  => "Attempt to call nil value — identifier not in scope",
            Self::DivisionByZero           => "Division by zero",
            Self::IndexOutOfBounds         => "List/string index out of bounds",
            Self::NilAccess                => "Property access on nil value",
            Self::BinaryOpTypeError        => "Type error in binary operation",
            Self::UnaryOpTypeError         => "Type error in unary operation",
            Self::StackOverflow            => "Call stack overflow — frame limit exceeded",
            Self::HeapExhausted            => "Heap exhausted (out of memory)",
            Self::InvalidConversion        => "Invalid type conversion",
            Self::IoError                  => "I/O error",
            Self::UsbError                 => "USB device error",
            Self::NetworkError             => "Network unreachable or connection refused",
            Self::GcPressure               => "Garbage collector pressure",
            Self::ResourceLeak             => "Resource leak detected",
            Self::ModuleNotFound           => "Module not found",
            Self::VersionConflict          => "Package version conflict",
            Self::CircularImport           => "Circular import detected",
            Self::LoadFailed               => "Module load failure",
            Self::ExportNotFound           => "Export not found in module",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::NilCall | Self::UndefinedIdentifier =>
                "Ensure the identifier is defined before use. Closures capture upvalues at definition time — verify the variable exists in the enclosing scope.",
            Self::UndefinedVariable =>
                "Declare the variable with `let name = value` before referencing it.",
            Self::ArityMismatch =>
                "Check the function signature. The number of call-site arguments must match declared parameters exactly.",
            Self::TypeMismatch | Self::BinaryOpTypeError =>
                "Use explicit conversion: `str(x)` for strings, `num(x)` for numbers. Check operand types with `ann.type(x)`.",
            Self::DivisionByZero =>
                "Guard the divisor: `if denom != 0 { x / denom } else { fallback }`",
            Self::IndexOutOfBounds =>
                "Check bounds before indexing: `if i < alg.len(list) { list[i] } else { nil }`",
            Self::StackOverflow =>
                "Use iteration (while/for) instead of deep recursion, or ensure the base case is always reachable. TCO only applies to direct tail calls.",
            Self::ModuleNotFound =>
                "Install the module: `axiom pkg install <name>`. Check spelling and ensure ~/.axiomlibs/ is writable.",
            Self::CircularImport =>
                "Break the import cycle by extracting shared code into a third module that both files import without further cross-loading.",
            Self::UsbError =>
                "Verify the device is connected and not claimed by another driver. Check vendor/product IDs with `usb.list()`.",
            Self::UnterminatedString =>
                "Add the closing quote character (\"\") to terminate the string literal.",
            Self::InvalidNumber =>
                "A valid number contains at most one decimal point: `3.14` not `3.1.4`.",
            Self::UnexpectedToken =>
                "Remove or replace the unrecognized character. See the Axiom character set in docs/syntax-ref.md.",
            _ => "See https://docs.axiom-lang.dev/errors for full documentation.",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.prefix(), self.summary())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Levenshtein distance — for AXM_200 "did you mean X?" suggestions
// ═══════════════════════════════════════════════════════════════════════════

/// Compute the Levenshtein edit distance between two strings.
/// Uses the classic DP approach with O(min(a,b)) space.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    if m == 0 { return n; }
    if n == 0 { return m; }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (curr[j - 1] + 1)
                .min(prev[j] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Find the closest match to `name` in `candidates`.
/// Returns `Some(candidate)` if the best distance ≤ `threshold`.
pub fn closest_match<'a>(name: &str, candidates: &[&'a str], threshold: usize) -> Option<&'a str> {
    candidates
        .iter()
        .map(|c| (*c, levenshtein(name, c)))
        .filter(|(_, d)| *d <= threshold)
        .min_by_key(|(_, d)| *d)
        .map(|(c, _)| c)
}

// ═══════════════════════════════════════════════════════════════════════════
// Source location helpers
// ═══════════════════════════════════════════════════════════════════════════

/// Convert a byte offset into (1-based line, 1-based column).
pub fn byte_to_line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let safe_offset = byte_offset.min(source.len());
    let prefix = &source[..safe_offset];
    let line = prefix.chars().filter(|&c| c == '\n').count() + 1;
    let col = match prefix.rfind('\n') {
        Some(nl) => byte_offset - nl,
        None     => byte_offset + 1,
    };
    (line, col)
}

/// Extract the text of line `line_number` (1-based) from source.
pub fn get_line(source: &str, line_number: usize) -> Option<&str> {
    source.lines().nth(line_number.saturating_sub(1))
}

// ═══════════════════════════════════════════════════════════════════════════
// AxiomDiagnostic — miette-backed error type
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Error, Diagnostic)]
#[error("{} {message}", self.code.prefix())]
#[diagnostic(help("{hint}"))]
pub struct AxiomDiagnostic {
    pub message:  String,
    pub code:     ErrorCode,
    pub hint:     String,
    /// Suggestion from Levenshtein (AXM_200 only)
    pub suggestion: Option<String>,
    #[source_code]
    pub src:      NamedSource,
    #[label("here")]
    pub span:     SourceSpan,
}

impl AxiomDiagnostic {
    pub fn new(
        code:        ErrorCode,
        message:     impl Into<String>,
        source_name: impl Into<String>,
        source_text: impl Into<String>,
        byte_start:  usize,
        byte_len:    usize,
    ) -> Self {
        AxiomDiagnostic {
            hint:       code.hint().into(),
            message:    message.into(),
            suggestion: None,
            code,
            src:        NamedSource::new(source_name.into(), source_text.into()),
            span:       (byte_start, byte_len.max(1)).into(),
        }
    }

    pub fn no_source(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::new(code, message, "<unknown>", " ", 0, 0)
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        let s = suggestion.into();
        self.hint = format!("{}  →  Did you mean '{}'?", self.hint, s);
        self.suggestion = Some(s);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RustcRenderer — manual rustc-style source context (fallback / supplement)
// ═══════════════════════════════════════════════════════════════════════════
//
// Produces output like:
//
//   error[AXM_402]: Attempt to call nil value
//    --> stdlib_demo.ax:12:16
//    |
//  11 |     fn inner(x) {
//  12 |         return adder(x)
//    |                ^^^^^ identifier resolves to nil
//    |
//    = help: Ensure the identifier is defined before use.
//

pub fn render_rustc_style(
    code:        ErrorCode,
    message:     &str,
    source_name: &str,
    source_text: &str,
    byte_start:  usize,
    byte_len:    usize,
    hint:        &str,
) -> String {
    use std::fmt::Write as FmtWrite;
    let mut out = String::new();

    let (line, col) = byte_to_line_col(source_text, byte_start);
    let span_len = byte_len.max(1);

    // Header line
    let _ = writeln!(out, "\x1b[1;31merror\x1b[0m\x1b[1m[{}]\x1b[0m: {}", code.prefix(), message);
    // Location
    let _ = writeln!(out, " \x1b[1;34m-->\x1b[0m {}:{}:{}", source_name, line, col);
    let _ = writeln!(out, "  \x1b[1;34m|\x1b[0m");

    // Context: one line before (if exists)
    if line > 1 {
        if let Some(prev_line_text) = get_line(source_text, line - 1) {
            let _ = writeln!(out, "\x1b[1;34m{:>3} |\x1b[0m {}", line - 1, prev_line_text);
        }
    }

    // Error line
    if let Some(err_line_text) = get_line(source_text, line) {
        let _ = writeln!(out, "\x1b[1;34m{:>3} |\x1b[0m {}", line, err_line_text);
        // Caret line
        let prefix_spaces = " ".repeat(col.saturating_sub(1) + 4 + 2); // 4 for "NNN |", 2 for space
        let carets = "^".repeat(span_len);
        let _ = writeln!(out, "  \x1b[1;34m|\x1b[0m {}\x1b[1;31m{}\x1b[0m", prefix_spaces, carets);
    }

    let _ = writeln!(out, "  \x1b[1;34m|\x1b[0m");
    let _ = writeln!(out, "  \x1b[1;34m=\x1b[0m \x1b[1mhelp\x1b[0m: {}", hint);
    let _ = writeln!(out);

    out
}

// ═══════════════════════════════════════════════════════════════════════════
// DiagnosticEngine — single reporting interface
// ═══════════════════════════════════════════════════════════════════════════

pub struct DiagnosticEngine {
    source_name:  String,
    source_text:  String,
    /// Known identifiers for Levenshtein spell-check
    known_names:  Vec<String>,
}

impl DiagnosticEngine {
    pub fn new(source_name: impl Into<String>, source_text: impl Into<String>) -> Self {
        DiagnosticEngine {
            source_name: source_name.into(),
            source_text: source_text.into(),
            known_names: Vec::new(),
        }
    }

    /// Register known identifiers for spell-check suggestions (AXM_200)
    pub fn register_names(&mut self, names: impl IntoIterator<Item = String>) {
        self.known_names.extend(names);
    }

    pub fn source_name(&self) -> &str { &self.source_name }
    pub fn source_text(&self) -> &str { &self.source_text }

    /// Convert a RuntimeError into a fully-spanned AxiomDiagnostic
    pub fn from_runtime(&self, err: &RuntimeError) -> AxiomDiagnostic {
        let (code, msg, span) = match err {
            RuntimeError::NilCall { hint, span } =>
                (ErrorCode::NilCall, format!("{}", hint), *span),
            RuntimeError::NotCallable { type_name, span } =>
                (ErrorCode::NotCallable,
                 format!("Value of type '{}' is not callable", type_name),
                 *span),
            RuntimeError::UndefinedVariable { name, span } =>
                (ErrorCode::UndefinedVariable,
                 format!("'{}' is not defined in this scope", name),
                 *span),
            RuntimeError::UndefinedFunction { name, span } =>
                (ErrorCode::UndefinedVariable,
                 format!("Function '{}' is not defined", name),
                 *span),
            RuntimeError::DivisionByZero { span } =>
                (ErrorCode::DivisionByZero, "Division by zero".into(), *span),
            RuntimeError::IndexOutOfBounds { index, length } =>
                (ErrorCode::IndexOutOfBounds,
                 format!("Index {} out of bounds (len={})", index, length),
                 Span::default()),
            RuntimeError::ArityMismatch { expected, found } =>
                (ErrorCode::ArityMismatch,
                 format!("Expected {} args, got {}", expected, found),
                 Span::default()),
            RuntimeError::TypeMismatch { expected, found, span } =>
                (ErrorCode::TypeMismatch,
                 format!("Expected {}, found {}", expected, found),
                 *span),
            RuntimeError::ImportError { module, message } =>
                (ErrorCode::ModuleNotFound,
                 format!("Cannot import '{}': {}", module, message),
                 Span::default()),
            RuntimeError::GenericError { message, span } =>
                (ErrorCode::NotCallable, message.clone(), *span),
            _ => (ErrorCode::NotCallable, format!("{}", err), Span::default()),
        };

        AxiomDiagnostic::new(
            code, msg,
            &self.source_name, &self.source_text,
            span.start,
            span.end.saturating_sub(span.start).max(1),
        )
    }

    /// Convert a ParserError into a fully-spanned AxiomDiagnostic
    pub fn from_parser(&self, err: &crate::errors::ParserError) -> AxiomDiagnostic {
        use crate::errors::ParserError;
        
        let (code, msg, span) = match err {
            ParserError::UnexpectedToken { expected, found, span } => {
                let msg = format!("Unexpected token '{}'. Expected: {}", found, expected);
                (ErrorCode::UnexpectedToken, msg, *span)
            }
            ParserError::InvalidSyntax { context, span } => {
                let msg = format!("Invalid syntax in {}", context);
                (ErrorCode::UnexpectedToken, msg, *span)
            }
            ParserError::UnexpectedEof { context, span } => {
                let msg = format!("Unexpected end of file while parsing {}", context);
                (ErrorCode::UnexpectedEof, msg, *span)
            }
        };

        AxiomDiagnostic::new(
            code, msg,
            &self.source_name, &self.source_text,
            span.start,
            span.end.saturating_sub(span.start).max(1),
        )
    }
    pub fn undefined_identifier(&self, name: &str, span: Span) -> AxiomDiagnostic {
        let refs: Vec<&str> = self.known_names.iter().map(|s| s.as_str()).collect();
        let suggestion = closest_match(name, &refs, 2);

        let message = match suggestion {
            Some(s) => format!("'{}' is not defined — did you mean '{}'?", name, s),
            None    => format!("'{}' is not defined in any reachable scope", name),
        };

        let diag = AxiomDiagnostic::new(
            ErrorCode::UndefinedIdentifier,
            message,
            &self.source_name, &self.source_text,
            span.start,
            span.end.saturating_sub(span.start).max(name.len()),
        );

        match suggestion {
            Some(s) => diag.with_suggestion(s),
            None    => diag,
        }
    }

    /// Build a nil-call error (AXM_402)
    pub fn nil_call(&self, identifier: &str, span: Span) -> AxiomDiagnostic {
        AxiomDiagnostic::new(
            ErrorCode::NilCall,
            format!("Attempt to call nil value '{}' — check parent-scope binding (AXM_402)", identifier),
            &self.source_name, &self.source_text,
            span.start,
            span.end.saturating_sub(span.start).max(identifier.len()),
        )
    }

    /// Emit to stderr using miette's fancy graphical renderer
    pub fn emit(&self, diag: &AxiomDiagnostic) {
        use miette::GraphicalReportHandler;
        let mut out = String::new();
        let _ = GraphicalReportHandler::new().render_report(&mut out, diag);
        eprintln!("{}", out);
    }

    /// Emit a RuntimeError to stderr (converts + renders)
    pub fn emit_runtime(&self, err: &RuntimeError) {
        self.emit(&self.from_runtime(err));
    }

    /// Emit a rustc-style diagnostic with source context + carets
    pub fn emit_rustc(
        &self,
        code:       ErrorCode,
        message:    &str,
        byte_start: usize,
        byte_len:   usize,
    ) {
        let hint = code.hint();
        let rendered = render_rustc_style(
            code, message,
            &self.source_name, &self.source_text,
            byte_start, byte_len, hint,
        );
        eprint!("{}", rendered);
    }

    /// Format a diagnostic to a String (useful in tests / snapshots)
    pub fn format_diagnostic(&self, diag: &AxiomDiagnostic) -> String {
        use miette::GraphicalReportHandler;
        let mut out = String::new();
        let _ = GraphicalReportHandler::new().render_report(&mut out, diag);
        out
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Unit tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_exact() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_one_edit() {
        assert_eq!(levenshtein("mesage", "message"), 1);
        assert_eq!(levenshtein("pint", "print"), 1);
    }

    #[test]
    fn test_closest_match_suggestion() {
        let candidates = ["message", "value", "counter", "print"];
        let result = closest_match("mesage", &candidates, 2);
        assert_eq!(result, Some("message"));
    }

    #[test]
    fn test_closest_match_no_suggestion() {
        let candidates = ["message", "value"];
        // "xyz" is too far from both
        let result = closest_match("xyz", &candidates, 2);
        assert_eq!(result, None);
    }

    #[test]
    fn test_byte_to_line_col() {
        let src = "let x = 1\nlet y = 2\nlet z = 3";
        // "let z" starts at byte 20
        let (line, col) = byte_to_line_col(src, 20);
        assert_eq!(line, 3);
        assert_eq!(col, 1);
    }

    #[test]
    fn test_error_code_prefix() {
        assert_eq!(ErrorCode::NilCall.prefix(), "[AXM_402]");
        assert_eq!(ErrorCode::UndefinedIdentifier.prefix(), "[AXM_200]");
        assert_eq!(ErrorCode::StackOverflow.prefix(), "[AXM_408]");
    }

    #[test]
    fn test_diagnostic_no_source() {
        let d = AxiomDiagnostic::no_source(ErrorCode::DivisionByZero, "Division by zero");
        assert!(d.message.contains("Division by zero"));
        assert!(d.code == ErrorCode::DivisionByZero);
    }

    #[test]
    fn test_undefined_identifier_suggestion() {
        let mut engine = DiagnosticEngine::new("test.ax", "let message = 1\nprint(mesage)");
        engine.register_names(vec!["message".into(), "print".into()]);
        let span = crate::errors::Span::new(0, 22, 27);
        let diag = engine.undefined_identifier("mesage", span);
        assert!(diag.message.contains("did you mean 'message'"));
        assert_eq!(diag.code, ErrorCode::UndefinedIdentifier);
    }

    #[test]
    fn test_rustc_render_no_panic() {
        let src = "let x = 10\nlet y = §\nprint(x + y)\n";
        let rendered = render_rustc_style(
            ErrorCode::UnexpectedToken,
            "Unrecognized character '§'",
            "test.ax", src,
            18, 2, // byte offset of §
            ErrorCode::UnexpectedToken.hint(),
        );
        assert!(rendered.contains("AXM_101"));
        assert!(rendered.contains("test.ax:2:"));
    }
}
