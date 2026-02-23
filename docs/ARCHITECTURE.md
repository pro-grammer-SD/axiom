# Axiom Complete Ecosystem Architecture

## Overview

The Axiom language provides a complete, production-ready ecosystem combining the **Genesis** syntax with a comprehensive standard library. This document covers the architecture, design decisions, and usage patterns for the entire system.

---

## 1. Core Language Features (Genesis Syntax)

### 1.1 Language Runtime

- **Lexer** (`lexer.rs`): Tokenizes Axiom source code with support for string interpolation
- **Parser** (`parser.rs`): Production recursive descent parser with optional keywords
- **Grammar** (`parser.lalrpop`): LALRPOP-based grammar spec (reference)
- **AST** (`ast.rs`): Abstract syntax tree definitions for all Axiom constructs
- **Runtime** (`runtime.rs`): GIL-free, threadsafe execution engine with flat-stack Env
- **Type System** (`core/value.rs`): Universal `AxValue` enum supporting all Axiom types
- **OOP** (`core/oop.rs`): Class inheritance, method dispatch, instance management

### 1.2 Genesis Syntax Summary

| Feature | Example | Status |
|---------|---------|--------|
| Optional Keywords | `main() { }` instead of `fun main()` | ✅ Implemented |
| Point Reference | `.name = value` instead of `self.name` | ✅ Implemented |
| String Methods | `"text".upper()`, `.align(width)` | ✅ Implemented |
| List Methods | `list.push(item)`, `list.pop()` | ✅ Implemented |
| Global Built-ins | `type()`, `int()`, `str()`, `avg()`, `sqrt()` | ✅ Implemented |
| Clean Match Syntax | `els` wildcard, `.Variant` enum scoping | ✅ Implemented |
| String Interpolation | `"Hello @name"`, `"Result: @(expr)"` | ✅ Implemented |
| Type Coercion | `bol(value)`, `int(value)` | ✅ Implemented |

---

## 2. Standard Library Architecture

### 2.1 Built-in Functions

The runtime ships all built-in functions directly — no external standard library is needed.

Key built-ins: `out`, `in`, `type`, `int`, `str`, `bol`, `sqrt`, `abs`, `floor`, `ceil`, `pow`, `min`, `max`, `avg`.
---

## 3. Standard Library Module Details

### 3.1 std mth - Mathematics & Cryptography

**Functions:**
- `sin(n)`, `cos(n)`, `tan(n)` - Trigonometry
- `log(n, base)` - Logarithm
- `pow(n, exp)` - Exponentiation
- `random()` - Cryptographically secure random (ChaCha20Rng)
- `hash_blake3(data)` - BLAKE3 hashing
- `sqrt(n)` - Square root

**Implementation:** Uses `rand`, `rand_chacha`, `blake3` crates

### 3.2 std ioo - Asynchronous I/O

**Functions:**
- `read_file(path)` - Read file contents
- `write_file(path, content)` - Write to file
- `append_file(path, content)` - Append to file
- `list_dir(path)` - List directory contents
- `exists(path)` - Check file exists

**Implementation:** Tokio-based async runtime with blocking wrapper

### 3.3 std sys - System Information & Time

**Functions:**
- `now()` - Current time as formatted string
- `timestamp()` - Unix timestamp
- `env_get(name)` - Get environment variable
- `cpu_usage()` - Current CPU usage percentage
- `memory_usage()` - Memory usage information
- `hostname()` - Get system hostname

**Implementation:** `chrono` for time, `sysinfo` for system metrics

### 3.4 std jsn - JSON Processing

**Functions:**
- `parse(json_str)` - Parse JSON to AxCValue
- `stringify(value)` - Convert to JSON string
- `pretty_print(value)` - Pretty-printed JSON

**Implementation:** `serde_json` with unsafe FFI boundaries

### 3.5 std net - Networking

**Functions:**
- `http_get(url)` - HTTP GET request
- `http_post(url, body)` - HTTP POST request
- `json_fetch(url)` - Fetch and parse JSON
- `listen(port)` - Create TCP listener
- `connect(host, port)` - Connect to TCP server

**Implementation:** `reqwest` for HTTP, tokio for sockets, `axum` for servers

### 3.6 std git - Git Operations

**Functions:**
- `init(path)` - Initialize repository
- `clone(url, path)` - Clone remote repo
- `current_branch(path)` - Get current branch
- `status(path)` - Get file status
- `last_commit(path)` - Get last commit hash

**Implementation:** `git2` library bindings

### 3.7 std htm - HTML Processing

**Functions:**
- `parse(html)` - Parse HTML document
- `text(html)` - Extract text content
- `select(html, css_selector)` - Query with CSS selector
- `get_attribute(element, attr)` - Get element attribute

**Implementation:** `html5ever` + `scraper` for CSS selectors

### 3.8 std col - Collections & Regex

**Functions:**
- `map_new()` - Create new map
- `set_new()` - Create new set
- `re_match(pattern, text)` - Regex match
- `re_find(pattern, text)` - Find matches
- `re_replace(pattern, text, replacement)` - Replace with regex

**Implementation:** `dashmap` for concurrent collections, `fancy-regex` for expressions

### 3.9 std conc - Concurrency Primitives

**Functions:**
- `channel()` - Create MPMC channel
- `send(channel_id, value)` - Send to channel
- `recv(channel_id)` - Receive from channel
- `try_recv(channel_id)` - Non-blocking receive
- `mutex_new()` - Create mutex
- `rwlock_new()` - Create read-write lock

**Implementation:** `flume` for channels, dashmap for concurrent structures

### 3.10 std mem - Serialization

**Functions:**
- `json_encode(value)` - Encode to JSON bytes
- `json_decode(bytes)` - Decode from JSON
- `xml_parse(xml_str)` - Parse XML
- `msgpack_encode(value)` - MessagePack compression
- `msgpack_decode(bytes)` - MessagePack decompression

**Implementation:** `serde_json`, `quick-xml`, `rmp`

### 3.11 std tui - Terminal UI

**Functions:**
- `create_window()` - Create TUI window
- `render_text(window, x, y, text)` - Render text
- `render_progress(window, percent)` - Render progress bar
- `poll_input()` - Wait for user input
- `clear_screen()` - Clear terminal

**Implementation:** `ratatui` (modern TUI framework)

### 3.12 std diag - Diagnostics

**Functions:**
- `profile_start(name)` - Start profiling block
- `profile_end(name)` - End profiling, return duration
- `get_profile(name)` - Get profiling results
- `memory_stats()` - Get detailed memory stats

### 3.13 std concurrency - Advanced Concurrency

**Functions:**
- `spawn_worker(count)` - Thread pool
- `submit_task(pool, closure)` - Submit work
- `barrier_new(count)` - Synchronization barrier

---

## 4. Build & Deployment System

### 4.1 Build Process

1. **Compile main compiler** (`axm/`):
   ```bash
   cargo build -p axm --release
   ```

### 4.2 Compilation Targets

- **axm binary**: Main compiler and runtime
- **Documentation**: Generated from markdown sources

### 4.3 Distribution

Axiom programs distributed as:
- `.ax` scripts (source)
- `.axc` compiled bytecode (future)
- `.axp` packaged applications (future)

---

## 5. Runtime Execution Model

### 5.1 Execution Pipeline

```
Source Code (.ax)
    ↓
Lexer (tokenization)
    ↓
Parser (production recursive descent)
    ↓
AST (abstract syntax tree)
    ↓
Semantic Analysis (type checking)
    ↓
Runtime Evaluation (interpreter)
    ↓
Output/effects
```

### 5.2 Value Representation

All Axiom values are represented as `AxValue` enum:

```rust
pub enum AxValue {
    Num(f64),                    // Floating-point numbers
    Str(String),                 // Strings
    Bol(bool),                   // Booleans
    Lst(Arc<RwLock<Vec<...>>>),  // Lists (mutable)
    Map(Arc<DashMap<...>>),      // Maps (thread-safe)
    Instance(Arc<RwLock<...>>),  // Class instances
    EnumVariant(Arc<str>, Box),  // Enum variants
    Fun(Arc<AxCallable>),        // Functions
    Nil,                         // Null value
}
```

### 5.3 Memory Management

- **Strings**: Owned `String` with COW optimization
- **Collections**: Arc-wrapped with interior mutability (RwLock/DashMap)
- **Functions**: Arc-wrapped callable code
- **Instances**: Arc-wrapped with thread-safe field access
- **Garbage Collection**: Implicit via Rust's Rc/Arc and drop semantics

### 5.4 Concurrency

- **Goroutines**: Spawned with `go { ... }` using tokio::spawn
- **Channels**: MPMC communication via goroutines
- **Mutexes/RwLocks**: Built-in to collection types
- **Thread Safety**: Collections use DashMap for lock-free access

---

## 6. Type System & Coercion

### 6.1 Type Inference

Axiom uses **structural typing** with runtime type checking:

```axiom
let x = 42;        // Inferred: Num
let s = "hello";   // Inferred: Str
let lst = [1, 2];  // Inferred: Lst
```

### 6.2 Type Coercion Rules

| From | To | Result |
|------|----|----|
| Num | Str | String representation |
| Num | Bol | false if 0.0, else true |
| Str | Num | Parsed number or nil |
| Str | Bol | false if empty, else true |
| Any | Str | Display representation |

### 6.3 Runtime Type Checking

Use `type()` function or `match` patterns:

```axiom
match value {
    42 => out "number",
    "text" => out "string",
    true => out "boolean",
    els => out "other",
}
```

---

## 7. Object-Oriented Programming

### 7.1 Classes

Classes support:
- Fields with default values
- Methods with implicit `self` (or `.`)
- Constructors (`init` method)
- Single inheritance (`ext` keyword)

```axiom
User {
    let name;
    let age;
    
    init(name, age) {
        .name = name;
        .age = age;
    }
    
    greet() {
        out "Hello, I'm @(.name)";
    }
}

let user = new User("Alice", 30);
user.greet();
```

### 7.2 Enums

Enums support:
- Named variants
- Optional data payloads
- Pattern matching

```axiom
Result {
    Ok(value),
    Error(message),
}

match result {
    Ok(v) => out "Success: @(v)",
    Error(e) => out "Failed: @(e)",
    els => out "Unknown",
}
```

### 7.3 Inheritance

Classes can extend a single parent:

```axiom
Animal {
    let name;
    speak() { out "Some sound"; }
}

Dog {
    .. ext Animal
    breed;
    
    speak() { out "Woof!"; }
}
```

---

## 8. Advanced Features

### 8.1 String Interpolation

Three forms:

1. **Simple variable**: `"Hello @name"`
2. **Expression**: `"Result: @(expr)"`
3. **Method call**: `"Upper: @(text.upper())"`

### 8.2 Pattern Matching

```axiom
match value {
    literal => action,
    identifier => action,
    EnumVariant(binding) => action,
    .ImplicitVariant => action,
    els => default_action,
}
```

### 8.3 Goroutines

Lightweight concurrency:

```axiom
go {
    for i in [1, 2, 3] {
        out @("Concurrent: @(i)");
    }
};
```

### 8.4 Method Chaining

```axiom
let result = data
    .filter(|x| x > 5)
    .map(|x| x * 2)
    .join(",");
```

---

## 9. Compiler Phases

### 9.1 Lexical Analysis

- Source text → Tokens
- Handles string interpolation at this stage
- Preserves source locations for error reporting

### 9.2 Parsing

- Tokens → Abstract Syntax Tree
- LALRPOP grammar with operator precedence
- Supports both traditional and Genesis syntax

### 9.3 Semantic Analysis

- Type checking (if enabled)
- Name resolution
- Import/export validation

### 9.4 Runtime Evaluation

- Direct AST interpretation
- Lazy evaluation where possible
- Eager for side effects

---

## 10. Error Handling

### 10.1 Error Types

```rust
pub enum RuntimeError {
    UndefinedVariable { name: String, span: Span },
    UndefinedFunction { name: String, span: Span },
    UndefinedClass { name: String },
    TypeError { expected: String, actual: String, span: Span },
    GenericError { message: String, span: Span },
}
```

### 10.2 Error Reporting

Errors include:
- Source file and line number
- Context (code snippet)
- Helpful suggestions

### 10.3 Exception-Free Design

- No try/catch blocks (currently)
- Use `Option` types (`nil` value)
- Match on result values

---

## 11. Performance Characteristics

### 11.1 Runtime Performance

| Operation | Time Complexity | Notes |
|-----------|-----------------|-------|
| Variable lookup | O(1) | Hash map access |
| Function call | O(n) | n = parameter count |
| List access | O(1) | Direct indexing |
| List append | O(1) | Amortized |
| Arithmetic | O(1) | Floating-point |
| String: concat | O(n+m) | n,m = string lengths |

### 11.2 Memory Usage

- **Per value**: 8-16 bytes base + payload
- **Collections**: Arc overhead (~16 bytes) + contents
- **Functions**: Code pointer + captured environment

### 11.3 Optimizations (Current & Planned)

- ✅ Lock-free collections (DashMap)
- ✅ Async I/O integration (Tokio)
- ✅ Flat-stack scope management (Env)
- ✅ Production-quality recursive descent parser
- [ ] String interning (future)
- [ ] JIT compilation (future - Cranelift)

---

## 12. Security Considerations

### 12.1 Safety Guarantees

- No unsafe pointer arithmetic (unless in native modules)
- Memory safety via Rust's type system
- Thread safety for all collections

### 12.2 Sandboxing

- Currently no isolation
- Future: WebAssembly-based sandboxing
- Future: Process isolation for untrusted code

### 12.3 Cryptography

- `blake3` for hashing (constant-time comparison)
- ChaCha20Rng for secure randomness
- TLS support via `rustls`

---

## 13. Testing & Quality Assurance

### 13.1 Test Suite

```bash
# Run all tests
cargo test --all

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Test specific module
cargo test -p axiom-mth
```

### 13.2 Example Programs

All features demonstrated in:
- `examples/bigdemo.ax` - Comprehensive language showcase
- `examples/test_*.ax` - Feature-specific tests

### 13.3 Benchmarking

```bash
cargo bench --all --release
```

---

## 14. Roadmap & Future Work

### Phase 1 (Current - Genesis)
- ✅ Optional keywords
- ✅ Point reference (`.`)
- ✅ String methods
- ✅ Global built-ins

### Phase 2 (Near Future)
- [ ] Bytecode compilation (.axc)
- [ ] Incremental compilation
- [ ] Better error messages
- [ ] Integrated debugger

### Phase 3 (Medium Term)
- [ ] Cranelift JIT compiler
- [ ] Full async/await syntax
- [ ] Trait system
- [ ] Module system improvements

### Phase 4 (Long Term)
- [ ] WebAssembly backend
- [ ] Distributed programming
- [ ] IDE/LSP support
- [ ] Package manager (axpm)

---

## 15. Community & Contribution

### 15.1 Code of Conduct

- Be respectful and inclusive
- Assume good intentions
- Focus on ideas, not individuals

### 15.2 Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Submit a pull request
5. Participate in code review

### 15.3 Getting Help

- Documentation: `docs/` directory
- Examples: `examples/` directory
- Issues: GitHub issue tracker
- Discussions: GitHub discussions

---

## Conclusion

Axiom Genesis represents the **second generation** of the Axiom language, combining:
- **Elegant syntax** that lets you write less
- **Production-ready runtime** with thread-safe concurrency
- **Extensible architecture** for custom native modules

Whether you're building data processing pipelines, web servers, system tools, or concurrent applications, Axiom Genesis provides the ergonomics and power to get it done efficiently.

**Write less. Achieve more. This is Axiom Genesis.**
