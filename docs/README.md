# Axiom Language Documentation

Welcome to the Axiom programming language! Axiom is a modern, compiled language designed for performance, concurrency, and developer productivity.

## Table of Contents

1. [Getting Started](GETTING_STARTED.md) - Installation and first program
2. [Language Reference](LANGUAGE.md) - Complete syntax and grammar
3. [Type System](TYPES.md) - Types, type inference, and type safety
4. [Standard Library](STDLIB.md) - Built-in modules and functions
5. [CLI Tool](CLI.md) - Using the axiom compiler and REPL
6. [Examples](EXAMPLES.md) - Sample programs and patterns
7. [Error Reference](ERRORS.md) - Error codes and troubleshooting

## Quick Start

Install Axiom:
```bash
cargo build --release --bin axiom
./target/release/axiom run hello.ax
```

Create `hello.ax`:
```axiom
let msg = "Hello, Axiom!";
```

Run your program:
```bash
axiom run hello.ax
```

## Key Features

✨ **Modern Syntax** - Clean, expressive syntax inspired by Rust, Python, and JavaScript

🚀 **High Performance** - Compiled to native code with LTO and fat optimizations

🔄 **Concurrent** - Built-in async/await and task spawning with `go` keyword

📦 **Modular** - Package system with dependency management

🎯 **Type Safe** - Static type checking with type inference

🔍 **Diagnostics** - Detailed error messages with source locations

## Language Highlights

### Variables and Binding
```axiom
let x = 42;           // immutable binding
let name = "Axiom";   // string literal
```

### Control Flow
```axiom
if x > 10 {
  // true branch
} else {
  // false branch
}

while x < 100 {
  let x = x + 1;
}

for item in [1, 2, 3] {
  // iterate over list
}
```

### Functions
```axiom
fun add(a, b) {
  ret a + b;
}
```

### Collections
```axiom
let nums = [1, 2, 3, 4, 5];
let map = {};
```

### Concurrency
```axiom
go some_async_task();
```

## Architecture

Axiom is built on a modern compiler architecture:

- **Lexer** - Tokenization with span tracking
- **Parser** - Recursive descent with full operator precedence
- **Semantic Analysis** - Type checking and symbol resolution
- **IR** - SSA-form intermediate representation
- **Runtime** - Tokio-based async executor with scope management

## Performance

Axiom compiles to native machine code with:
- Link-time optimization (LTO)
- Fat LTO codec generation
- Panic abort for minimal runtime overhead
- Thread-safe concurrent data structures (DashMap, Arc)

## Project Status

Current Version: **0.1.0**

**Completed:**
- ✅ Lexer with all token types
- ✅ Parser with full operator precedence
- ✅ Type system and semantic analysis
- ✅ Runtime with control flow support
- ✅ CLI tool (run, check, format, repl)
- ✅ Error diagnostics with codes

**In Progress:**
- 🔄 Standard library implementation
- 🔄 Async/await support
- 🔄 Package engine

**Planned:**
- 📋 LSP server
- 📋 WASM backend
- 📋 Borrow checker
- 📋 Built-in primitives (net, json, sys, concurrency, git via pkg)

## Community & Contributing

Axiom is an open-source project. We welcome:
- Bug reports
- Feature requests
- Documentation improvements
- Code contributions

## License

Axiom Language © 2026

## More Information

- [Language Specification](LANGUAGE.md) - Complete grammar and syntax
- [Contributing Guide](../CONTRIBUTING.md) - How to contribute
- [Issue Tracker](../../../issues) - Report bugs or request features
