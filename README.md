# Axiom Language

Axiom is a high-performance,GIL-free, thread-safe language designed for Omega maturity. It features a "Genesis" syntax that is hyper-minimal, making keywords like `fun`, `cls`, and `enm` optional.

## Features

- **Genesis Syntax**: Minimalist and clean.
- **High Performance**: Designed from the ground up for speed.
- **Concurrent**: Built-in support for goroutines and thread-safe collections.
- **OOP & Functional**: Supports classes, enums, and lambdas.
- **Built-in Runtime**: Core builtins covering I/O, math, strings, and lists — no external stdlib needed.

## Quick Start

### Build

```bash
cargo build
```

### Run an Example

```bash
./target/debug/axm run examples/bigdemo.ax
```

## Project Structure

- `axm`: The main language runtime and CLI toolchain.

## Examples

Check out the `examples/` directory for demonstrations of language features :D
