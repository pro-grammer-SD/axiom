# Axiom Intrinsic Monolith Architecture

## Overview

This document describes the new **Static Intrinsic Monolith** architecture for Axiom, which replaces the dynamic `.rax` loading system with direct, compiled-in intrinsic modules.

### Key Changes

1. **FROM**: Dynamic `.rax` modules loaded at runtime via `libloading`
2. **TO**: 22 statically-linked Rust intrinsics compiled directly into `axm` binary
3. **Module Access**: `mth.sqrt()`, `str.split()`, `dfm.read()` etc. work natively
4. **No `std;` prefix needed**: All modules are globally available
5. **Heap-based stack**: `Vec<StackFrame>` eliminates C-stack overflow for deep recursion

---

## Architecture Components

### 1. Intrinsic Registry (`axm/src/intrinsics.rs`)

The **Global Switchboard** that maps module identifiers to their implementations:

```rust
pub struct IntrinsicRegistry {
    modules: HashMap<String, IntrinsicModule>,
}

impl IntrinsicRegistry {
    pub fn call(&self, module: &str, function: &str, args: Vec<AxValue>) -> Result<AxValue, String> {
        // Resolves and invokes intrinsic functions
    }
}
```

**Invocation**:
```rust
// When user calls: str.split("hello", ",")
registry.call("str", "split", vec![string_val, separator_val])?
```

### 2. 22 Standard Library Modules

All modules are registered in `IntrinsicRegistry::new()`:

#### Logic Tier
- **mth** - Trigonometric, logarithmic, exponential math
- **num** - Tensors, matrices, symbolic algebra (ndarray, symrs)
- **alg** - Graph algorithms, parallel sorting (petgraph, rayon)
- **ann** - Runtime type reflection and memory guards
- **tim** - Nanosecond UTC timestamps, benchmarking (chrono)
- **str** - String manipulation, regex patterns (regex, unicode-segmentation)
- **col** - Thread-safe collections (dashmap for HashMap/B-Trees)

#### Data Tier
- **dfm** - DataFrame operations (polars LazyFrames)
- **jsn** - JSON serialization (serde_json)
- **csv** - CSV streaming
- **web** - HTTP requests and DOM scraping (reqwest, scraper)

#### Operations Tier
- **ioo** - Buffered I/O and pipes
- **pth** - Path manipulation and filesystem (walkdir)
- **env** - Environment variables (dotenvy)
- **sys** - System info and subprocess (sysinfo)
- **git** - Native Git operations (git2)
- **aut** - Task scheduling and file watching (croner, notify)

#### Visualization/Async Tier
- **clr** - 24-bit TrueColor output (colored)
- **log** - Progress bars and spinners (indicatif)
- **tui** - Terminal UI layouts (ratatui)
- **plt** - Chart rendering (plotters)
- **con** - Networking and async M:N tasking (tokio)

---

## Implementation Status

### Fully Implemented
- ✅ **mth** (14 functions: sin, cos, tan, sqrt, abs, ln, log10, exp, pow, floor, ceil, round, pi, e)
- ✅ **tim** (3 functions: now_ms, now_s, sleep_ms)
- ✅ **str** (7 functions: len, concat, split, upper, lower, trim)
- ✅ **col** (3 functions: dict, list, set)
- ✅ **ioo** (2 functions: read, write)
- ✅ **pth** (2 functions: abs, exists)
- ✅ **env** (2 functions: get, set)
- ✅ **sys** (1 function: info)
- ✅ **jsn** (2 functions: parse, stringify)

### Stub Implementation (Ready for Extension)
All other modules have placeholder structures with:
- Module registration in registry
- Empty function maps ready for implementations
- Clear extension points for future functionality

---

## Usage Examples

### Before (Old .rax System - DEPRECATED)
```axiom
std;  // Deprecated
result = mth.sqrt(25.0);  // Module not available at compile time
```

### After (New Intrinsic System)
```axiom
result = mth.sqrt(25.0);  // Works natively! No std; needed
items = str.split("hello,world", ",");
now_ms = tim.now_ms();
```

---

## Adding New Intrinsic Functions

### Step 1: Update `IntrinsicRegistry` in `axm/src/intrinsics.rs`

For example, adding `mth.max(a, b)`:

```rust
fn register_mth_module(&mut self) {
    let mut functions: HashMap<String, IntrinsicFn> = HashMap::new();

    // ... existing functions ...

    // Add max function
    functions.insert("max".to_string(), Arc::new(|args: Vec<AxValue>| {
        if args.len() != 2 { return Err("max expects 2 arguments".to_string()); }
        let a = match &args[0] {
            AxValue::Float(x) => *x,
            AxValue::Int(x) => *x as f64,
            _ => return Err("max expects numbers".to_string()),
        };
        let b = match &args[1] {
            AxValue::Float(x) => *x,
            AxValue::Int(x) => *x as f64,
            _ => return Err("max expects numbers".to_string()),
        };
        Ok(AxValue::Float(a.max(b)))
    }));

    self.modules.insert("mth".to_string(), IntrinsicModule {
        name: "mth".to_string(),
        functions,
    });
}
```

### Step 2: Test in Axiom Code
```axiom
result = mth.max(10.0, 20.0);
out result;  // Output: 20
```

---

## Runtime Integration

The runtime detects intrinsic calls through the parser's **MethodCall** AST node:

```rust
// When parser sees: obj.method(args)
Expr::MethodCall { object, method, arguments, .. }
```

The runtime checks if `object` is an identifier matching a known module:

```rust
if let Expr::Identifier { name } = &**object {
    if let Some(mut module_name) = known_modules.get(name) {
        // Resolve as intrinsic call
        return runtime.intrinsics.call(&module_name, &method, args)?;
    }
}
```

---

## Extending Data Structure Support

The `AxValue` enum supports:
- `Nil`, `Bool(bool)`, `Int(i64)`, `Float(f64)`, `String(String)`
- `List(Arc<RwLock<Vec<AxValue>>>)` - Mutable lists with interior mutability
- `Dict(Arc<RwLock<HashMap<String, AxValue>>>)` - Mutable dictionaries
- `Set(Arc<RwLock<HashSet<String>>>>` - Unique string sets
- All thread-safe for intrinsic operations

---

## Stack Overflow Prevention

The VM uses an explicit **heap-allocated stack**:

```rust
pub struct FlatVM {
    bytecode: Vec<Instruction>,
    stack_frames: Vec<StackFrame>,  // Vec on heap, not Rust call stack
    value_stack: Vec<AxValue>,
}
```

This allows `fib(35)` and deeper recursion **without hitting C-stack limits**.

---

## Build & Deployment

### Build the Binary
```bash
cargo build --release
```

This produces `target/release/axm` with all 22 modules statically linked.

### No External Dependencies
```bash
# Before:
# ~/.axiom/lib/axiom_mth.dll
# ~/.axiom/lib/axiom_str.dll
# ... (22 .dll/$o/dylib files)

# After:
# axm.exe (single binary, all 22 modules inside)
```

---

## Testing fib(35) Recursion

```axiom
fun fib(n) {
    if n <= 1 {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

result = fib(35);
out result;  // Should print 9227465 without stack overflow
```

The heap-based stack handles this easily, even with exponential recursive depth.

---

## Module Completion Checklist

- [ ] **mth** - Completed (14 functions)
- [ ] **num** - Implement ndarray tensor/matrix operations
- [ ] **alg** - Implement petgraph and rayon algorithms
- [ ] **ann** - Implement type reflection introspection
- [ ] **tim** - Completed (3 core functions)
- [ ] **str** - Completed (7 functions, add regex support)
- [ ] **col** - Completed (3 functions, expand with dashmap)
- [ ] **dfm** - Implement polars DataFrames
- [ ] **jsn** - Completed (2 functions, expand handling)
- [ ] **csv** - Implement csv reader/writer
- [ ] **web** - Implement reqwest + scraper
- [ ] **ioo** - Completed (2 functions, add streaming)
- [ ] **pth** - Completed (2 functions, expand walkdir)
- [ ] **env** - Completed (2 functions, add dotenvy parsing)
- [ ] **sys** - Completed (1 function, add more sysinfo),
- [ ] **git** - Implement git2 operations
- [ ] **aut** - Implement croner scheduling + notify watching
- [ ] **clr** - Implement colored output
- [ ] **log** - Implement indicatif progress bars
- [ ] **tui** - Implement ratatui layouts
- [ ] **plt** - Implement plotters charts
- [ ] **con** - Implement tokio networking

---

## Migration Guide (Old to New)

### Lexer Changes
- `std;` keyword is **deprecated** (but parser silently accepts and ignores)
- No need to import modules anymore

### Parser Changes
- Module-qualified calls like `mth.sqrt()` are recognized natively
- No special syntax needed

### Runtime Changes
- All modules available by default
- Call resolution happens in `IntrinsicRegistry`

---

## References

- [LALRPOP Parser](docs/PARSER.md)
- [Core AST](axm/src/ast.rs)
- [Intrinsics Registry](axm/src/intrinsics.rs)
- [Runtime Evaluation](axm/src/runtime.rs)

