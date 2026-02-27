# Getting Started with Axiom

## Installation

```bash
git clone https://github.com/axiom-lang/axiom
cd axiom/axiom
cargo build --release

# The build script automatically:
#   1. Creates ~/.axiom/bin/ and copies the binary
#   2. Installs both `axiom` AND `axm` (short alias) to ~/.axiom/bin/
#   3. Updates your shell profile with a PATH entry
#   4. Generates ~/.axiom/conf.txt with sensible defaults
```

## Your First Script

Create `hello.ax`:

```axiom
print("Hello, Axiom!")
```

Run it:

```bash
axiom run hello.ax
axm run hello.ax    # short alias
```

## Variables & Types

```axiom
let x       = 42       // integer  → Val::Int(i64), no boxing overhead
let pi      = 3.14     // float    → Val::Float(f64)
let name    = "Axiom"  // string   → Val::Str(Arc<str>)
let flag    = true     // bool
let nothing = nil      // nil
let nums    = [1,2,3]  // list     → Val::List(Arc<Mutex<Vec<Val>>>)
```

## Functions & Closures

```axiom
fn add(a, b) { return a + b }
print(add(3, 4))   // 7

// Higher-order — nil-call safe (AXM_402 eliminated)
fn make_adder(x) {
    fn adder(y) { return x + y }   // x captured as upvalue[0]
    return adder
}
let add5 = make_adder(5)
print(add5(10))   // 15
```

## Tail-Call Optimization

```axiom
fn sum_tco(n, acc) {
    if n == 0 { return acc }
    return sum_tco(n - 1, acc + n)   // Op::CallTail reuses current frame
}
print(sum_tco(10000, 0))   // 50005000 — no stack growth
```

## CLI Reference

```bash
axiom run   <file.ax>           # Execute a script
axiom chk   <file.ax>           # Semantic analysis (no execution)
axiom fmt   <file.ax> --write   # Format source in-place
axiom pkg   add    <n>          # Install package
axiom pkg   list                # List installed packages
axiom conf  set    key=value    # Set config property
axiom conf  list                # List all config
axiom conf  reset               # Reset to defaults
```
