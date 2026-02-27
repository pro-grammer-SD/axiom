# 🔡 Axiom Language — Intrinsic Monolith Edition

Axiom is a **statically-linked, monolithic high-performance language** with zero external module dependencies. All 22 standard library modules are directly embedded in the core binary as intrinsic Rust implementations, providing instant startup and guaranteed availability. It can be upto 2-3x faster than Python!

## 🎯 Benchmark It!

### 🧑‍🚀 Hyperfine (Recommended):
```
cargo install hyperfine # or skip this step if you got it installed
hyperfine --warmup 3 --runs 10 ".\target\release\axiom.exe run examples\core\fib_2.ax" "python .\local\fib_2.py" # run the benchmarks

```

### 🌟 Powershell:

```
$results = 1..10 | ForEach-Object {
    $a = Measure-Command {
        & ./target/release/axiom run examples/core/fib_2.ax > $null
    }

    $p = Measure-Command {
        python local/fib_2.py > $null
    }

    [PSCustomObject]@{
        Run       = $_
        Axiom_ms  = $a.TotalMilliseconds
        Python_ms = $p.TotalMilliseconds
    }
}
$results | Measure-Object Axiom_ms  -Average -Minimum -Maximum
$results | Measure-Object Python_ms -Average -Minimum -Maximum

```

### 🌿 Bash:

```
#!/usr/bin/env bash

runs=10
axiom_times=()
python_times=()

for i in $(seq 1 $runs); do
    start=$(date +%s%N)
    ./target/release/axiom run examples/core/fib_2.ax > /dev/null
    end=$(date +%s%N)
    axiom_times+=($(( (end - start) / 1000000 )))

    start=$(date +%s%N)
    python local/fib_2.py > /dev/null
    end=$(date +%s%N)
    python_times+=($(( (end - start) / 1000000 )))
done

compute_stats() {
    local arr=("$@")
    local sum=0
    local min=${arr[0]}
    local max=${arr[0]}

    for v in "${arr[@]}"; do
        (( sum += v ))
        (( v < min )) && min=$v
        (( v > max )) && max=$v
    done

    avg=$(( sum / ${#arr[@]} ))
    echo "avg=${avg}ms min=${min}ms max=${max}ms"
}

echo "Axiom:"
compute_stats "${axiom_times[@]}"

echo "Python:"
compute_stats "${python_times[@]}"

```

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

This creates a single, statically-linked `axiom` executable with **all 22 modules compiled in**.

### Run Examples

```bash
# Run a simple example
./target/release/axiom examples/fib.ax

# Run the comprehensive stdlib test
./target/release/axiom examples/stdlib_demo.ax
```

### Comprehensive Stdlib Verification

The `stdlib_demo.ax` script exercises all 22 modules:

```bash
./target/release/axiom examples/stdlib_demo.ax
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
├── axiom/                    # Main language compiler & runtime
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

Output: `target/release/axiom` (statically-linked executable)

### Run Tests

```bash
cargo check
cargo test
```

## Documentation

* **[getting-started.md](docs/getting-started.md)** — Installation and your first Axiom script
* **[syntax-ref.md](docs/syntax-ref.md)** — Quick reference for operators, keywords, and types
* **[config-tuning.md](docs/config-tuning.md)** — Performance optimization and stack configuration
* **[package-management.md](docs/package-management.md)** — Using the Axiom package manager (`pkg`)
* **[monolith-intrinsics.md](docs/monolith-intrinsics.md)** — Documentation for core built-in functions
* **[latest-fixes.md](docs/latest-fixes.md)** — Recent stability updates and bug squashing

---

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

---

## ⚙️ Runtime Configuration

Axiom uses a configuration file at **`~/.axiom/conf.txt`** (created automatically on first build/install).

### Quick Start

```bash
axiom conf list                        # show all properties with current values
axiom conf set nan_boxing=true         # enable NaN-boxing
axiom conf get gc_mode                 # print current gc_mode value
axiom conf describe peephole_optimizer # full documentation for one property
axiom conf reset                       # restore all defaults
```

### Feature Toggle Properties (all default `true` for maximum performance)

| Property | Default | Description |
|---|---|---|
| `nan_boxing` | `true` | NaN-boxed 64-bit value representation for the VM |
| `bytecode_format` | `true` | Register-based bytecode execution (vs tree-walk fallback) |
| `ic_enabled` | `true` | Inline-cache subsystem master switch (property/call ICs) |
| `gc_enabled` | `true` | Garbage-collector master switch |
| `peephole_optimizer` | `true` | Full static optimisation pipeline (fold, peephole, DCE, …) |
| `profiling_enabled` | `true` | Runtime profiling infrastructure (counters, hot-loop detect) |

### `~/.axiom/` Directory Layout

```
~/.axiom/
├── conf.txt          — runtime configuration (all toggles & tuning knobs)
├── bin/
│   └── axiom           — installed binary (populated by `cargo build --release`)
├── lib/              — reserved for future stdlib extensions
└── cache/            — bytecode cache (when bytecode_cache=on)

~/.axiomlibs/         — Axiomide package store
```

### All Configuration Categories

| Category | Key properties |
|---|---|
| Feature Toggles | `nan_boxing`, `bytecode_format`, `ic_enabled`, `gc_enabled`, `peephole_optimizer`, `profiling_enabled` |
| Debug | `debug`, `opcode_trace`, `gc_verbose`, `bounds_check`, `stack_trace_on_error` |
| Inline Cache | `inline_cache`, `poly_ic_size`, `call_ic` |
| Garbage Collector | `gc_mode`, `nursery_size_kb`, `gc_parallel` |
| Optimization | `constant_folding`, `peephole`, `dead_code`, `jump_threading`, `superinstructions`, `opt_level` |
| Specialization | `quickening`, `shape_optimization`, `quicken_threshold` |
| Profiling | `profiling`, `opcode_counters`, `hot_loop_detect`, `hot_threshold`, `flame_graph` |
| VM | `max_call_depth`, `register_count` |

See `axiom conf list` and `axiom conf describe <property>` for full documentation.
