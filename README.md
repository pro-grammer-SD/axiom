# Axiom Language — Intrinsic Monolith Edition

Axiom is a **statically-linked, monolithic high-performance language** with zero external module dependencies. All 22 standard library modules are directly embedded in the core binary as intrinsic Rust implementations, providing instant startup and guaranteed availability.

## Key Highlights

### ⚡ **Intrinsic Monolith**
- **All 22 stdlib modules compiled directly into the binary** — no .rax files, no dynamic loading, no external dependencies
- **Zero startup overhead** — everything available instantly
- **Raw Rust performance** — each function maps directly to optimized Rust crate operations
- **Static linking** — single, self-contained executable with no runtime dependencies

### 🎯 **22 Embedded Modules**

| Module | Purpose | Functions |
|--------|---------|-----------|
| **alg** | Algorithms & Logic | `range`, `sum`, `filter`, `fold`, `sort` |
| **ann** | Reflection & Annotations | `type_of`, `is_num`, `is_str`, `is_lst`, `is_map`, `fields` |
| **aut** | Automation & Time | `now`, `sleep`, `timestamp`, `parse_time`, `delay` |
| **clr** | Color Operations | `rgb`, `hex`, `hsv` |
| **col** | Collections (Maps) | `new`, `get`, `set`, `remove`, `len`, `keys`, `values` |
| **con** | Concurrency | `spawn`, `wait`, `mutex_new` |
| **csv** | CSV Processing | `parse`, `write`, `headers` |
| **dfm** | DataFrames (Polars) | `from_csv`, `shape`, `select`, `filter` |
| **env** | Environment Variables | `get`, `set`, `load`, `all` |
| **git** | Git Operations | `branch`, `log`, `status`, `clone` |
| **ioo** | File I/O | `read`, `write`, `append`, `exists`, `delete`, `list` |
| **jsn** | JSON Processing | `parse`, `stringify`, `get` |
| **log** | Logging & Progress | `info`, `warn`, `error`, `progress` |
| **mth** | Mathematics | `sqrt`, `sin`, `cos`, `tan`, `abs`, `floor`, `ceil`, `round`, `pow`, `log10` |
| **net** | Networking (HTTP) | `get`, `post` |
| **num** | Numerics (ndarray) | `zeros`, `ones`, `range_array` |
| **plt** | Plotting (Plotters) | `scatter`, `line` |
| **pth** | Path Operations | `list`, `walk`, `join` |
| **str** | String Operations | `match` (regex), `replace`, `split`, `join`, `len`, `upper`, `lower` |
| **sys** | System Information | `info`, `cpu_usage`, `memory` |
| **tim** | Time & Formatting | `now`, `format` |
| **tui** | Terminal UI | `box`, `line`, `table` |

### ✨ **Core Architecture**

```
Source Code (.ax)
    ↓
Lexer (logos)
    ↓
Parser (LALRPOP)
    ↓
AST
    ↓
Type Checker
    ↓
Runtime/VM
    ↓
Intrinsics Engine (22 pure Rust modules)
    ↓
Output
```

## Quick Start

### Build

```bash
cargo build --release
```

This creates a single, statically-linked `axm` executable with **all 22 modules compiled in**.

### Run Examples

```bash
# Run a simple example
./target/release/axm examples/fib.ax

# Run the comprehensive stdlib test
./target/release/axm examples/stdlib_demo.ax
```

### Comprehensive Stdlib Verification

The `stdlib_demo.ax` script exercises all 22 modules:

```bash
./target/release/axm examples/stdlib_demo.ax
```

If this runs successfully, **the entire intrinsic monolith is verified**.

## Language Features

### Genesis Syntax (Hyper-Minimal)

```axiom
// Functions
fn add(x, y) { x + y }

// Classes
cls Person {
    name: Str
    age: Num
    
    fn greet() {
        print("Hi, I'm " + self.name);
    }
}

// Enums
enm Status { Active, Inactive, Pending }

// Control Flow
if condition { ... }
for item in list { ... }
match value { ... }
```

### Type System

```axiom
let num: Num = 42;
let str: Str = "hello";
let bool: Bol = true;
let list: Lst = [1, 2, 3];
let map: Map = { "key" => "value" };
```

### Concurrency

```axiom
// Spawn async task
let result = con.spawn(fn() { heavy_computation() });

// Wait for result
let value = con.wait(result);

// Mutex for thread-safe state
let counter = con.mutex_new(0);
```

## Project Structure

```
axiom/
├── axm/                    # Main language compiler & runtime
│   └── src/
│       ├── main.rs         # CLI entry point
│       ├── lexer.rs        # Token generation (logos)
│       ├── parser.rs       # Grammar rules (LALRPOP)
│       ├── ast.rs          # Abstract syntax tree
│       ├── runtime.rs      # Execution engine
│       ├── vm.rs           # Bytecode interpreter
│       └── intrinsics.rs   # 22 embedded stdlib modules ⭐
├── examples/
│   ├── fib.ax              # Fibonacci example
│   ├── stdlib_demo.ax      # Comprehensive 22-module test ⭐
│   └── ...                 # More examples
├── docs/
│   ├── ARCHITECTURE.md     # System design
│   ├── MONOLITH_STATUS.md  # Migration details
│   └── ...
└── Cargo.toml              # Project manifest
```

## Performance Characteristics

- **Startup**: **Instant** — all code pre-compiled, zero dynamic loading
- **Memory**: Efficient — single binary, no runtime overhead
- **Execution**: Native performance — raw Rust implementations
- **Concurrency**: True parallelism — rayon-backed with task-local state

## Building from Source

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))

### Compile

```bash
cd axiom
cargo build --release
```

Output: `target/release/axm` (statically-linked executable)

### Run Tests

```bash
cargo check
cargo test
```

## Documentation

- [ARCHITECTURE.md](docs/ARCHITECTURE.md) — System design and bytecode format
- [LANGUAGE.md](docs/LANGUAGE.md) — Full language specification
- [MODULE_REFERENCE.md](docs/MODULE_REFERENCE.md) — Detailed API documentation
- [MONOLITH_STATUS.md](docs/MONOLITH_STATUS.md) — Migration from modular to monolithic
- [EXAMPLES.md](docs/EXAMPLES.md) — Code samples for all features

## Examples

### Fibonacci (Recursive)

```axiom
fn fib(n) {
    if n <= 1 { n }
    else { fib(n - 1) + fib(n - 2) }
}

print(fib(10));  // 55
```

### CSV Processing

```axiom
let csv_data = "name,age
Alice,30
Bob,25";

let parsed = csv.parse(csv_data);
let headers = csv.headers(parsed);
print(headers);  // [name, age]
```

### Concurrent Computation

```axiom
let tasks = [
    con.spawn(fn() { alg.sum(alg.range(1000)) }),
    con.spawn(fn() { alg.sum(alg.range(2000)) })
];

for task in tasks {
    print(con.wait(task));
}
```

### Color Operations

```axiom
let red = clr.rgb(255, 0, 0);
let blue_hex = clr.hex("#0000FF");
let green_hsv = clr.hsv(120, 100, 100);
```

### System Information

```axiom
let cpu = sys.cpu_usage();
let mem = sys.memory();
let info = sys.info();

print("CPU: " + str(cpu) + "%");
print(mem);
print(info);
```

## Why Intrinsic Monolith?

| Aspect | Traditional Modular | Intrinsic Monolith |
|--------|---------------------|-------------------|
| **Startup** | Module loading overhead | Instant (pre-compiled) |
| **Binary Size** | Multiple .rax files | Single executable |
| **Dependencies** | External crate loading | Zero runtime deps |
| **Availability** | Dynamic lookup | Always available |
| **Performance** | Import overhead | Direct call |
| **Deployment** | Complex distribution | Single file |

## Contributing

Contributions welcome! See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

## License

MIT License — See LICENSE file for details.

---

**Status**: ✅ **Production Ready** — All 22 modules fully implemented, comprehensive testing via `stdlib_demo.ax`, zero external dependencies.
