# AXIOM MONOLITH — QUICK START GUIDE

## Installation & Setup

### Automated Setup (Recommended)

```powershell
# Navigate to Axiom root directory
cd C:\Users\ADMIN\Desktop\programming\github_repos\axiom

# Run the automation script
.\setup.ps1 -Clean -Build -Verify -AddToPath
```

**This will:**
1. ✅ Delete deprecated crates (`axiom_sdk`, `axiom_macros`, `modules/`)
2. ✅ Build release binary (`cargo build --release`)
3. ✅ Add to Windows PATH
4. ✅ Verify all 22 modules are functioning

### Manual Setup

```bash
cargo build --release
# Binary location: target/release/axm.exe
```

---

## Running Axiom Programs

After setup, use the monolithic binary directly:

```bash
axm your_program.ax
```

### Example 1: Hello World with Intrinsics

**File:** `hello.ax`
```axiom
out "=== AXIOM MONOLITH ===";
out "";

// ALG module: generate range
let numbers = alg.range(5);
out "Numbers:";
out numbers;

// ANN module: check types
out "";
out "Type of 42:";
out ann.type_of(42);

out "Type of [1, 2]:";
out ann.type_of([1, 2]);

out "";
out "Done!";
```

**Run:**
```bash
axm hello.ax
```

**Output:**
```
=== AXIOM MONOLITH ===

Numbers:
[0, 1, 2, 3, 4]

Type of 42:
Num

Type of [1, 2]:
Lst

Done!
```

---

## Module Access

All 22 modules are **instantly available** — no import statements needed:

```axiom
// ALG — Algorithms
let range_result = alg.range(10);

// ANN — Annotations
let type_name = ann.type_of("hello");

// AUT — Automation
let current_time = aut.now();

// CLR — Colors
let color = clr.rgb(255, 128, 0);

// COL — Collections
let map = col.new();
col.set(map, "key", "value");

// CON — Concurrency
let task = con.spawn(fun { out "Task"; });

// CSV — CSV Parsing
let rows = csv.parse("name,age\nAlice,30");

// DFM — DataFrames
let df = dfm.from_csv("a,b\n1,2");

// ENV — Environment
let home = env.get("HOME");

// GIT — Git Operations
let branch = git.branch(".");

// IOO — File I/O
let content = ioo.read("file.txt");

// All other modules available similarly...
```

---

## Common Patterns

### Pattern 1: File Read & Parse

```axiom
fun read_and_parse(filepath) {
  // Check file exists
  if ioo.exists(filepath) {
    let content = ioo.read(filepath);
    out "File loaded:";
    out content;
  } else {
    out "File not found:";
    out filepath;
  }
}

read_and_parse("data.csv");
```

### Pattern 2: Timing Operations

```axiom
fun measure_time(func) {
  let start = aut.now();
  
  // Execute function
  func();
  
  let end = aut.now();
  let elapsed = end - start;
  
  out "Execution time:";
  out elapsed;
  out "ms";
}

measure_time(fun {
  aut.sleep(500);
  out "Work done";
});
```

### Pattern 3: Configuration Management

```axiom
fun setup_config() {
  // Load from environment
  env.load();
  
  let config = col.new();
  col.set(config, "api_host", env.get("API_HOST"));
  col.set(config, "api_port", env.get("API_PORT"));
  col.set(config, "debug", true);
  
  out "Configuration:";
  out config;
  
  return config;
}

let cfg = setup_config();
```

### Pattern 4: Type Validation

```axiom
fun validate_input(value) {
  if ann.is_num(value) {
    out "Valid number:";
    out value;
    return true;
  } else if ann.is_str(value) {
    out "Valid string:";
    out value;
    return true;
  } else {
    out "Invalid type:";
    out ann.type_of(value);
    return false;
  }
}

validate_input(42);
validate_input("hello");
validate_input([1, 2, 3]);
```

### Pattern 5: CSV Processing

```axiom
fun process_csv(csv_string) {
  let rows = csv.parse(csv_string);
  
  out "Total rows:";
  out col.len(rows);
  
  out "";
  out "First row:";
  out rows[0];
  
  let keys = ann.fields(rows[0]);
  out "Columns:";
  out keys;
}

let sample = "name,age,city
Alice,30,NYC
Bob,25,LA
Charlie,35,SF";

process_csv(sample);
```

---

## Available Functions by Module

### Quick Reference

| Module | Key Functions |
|--------|---------------|
| **alg** | range, sum, filter, fold, map_parallel |
| **ann** | type_of, is_num, is_str, is_lst, is_map, fields |
| **aut** | now, sleep, timestamp, parse_time, delay |
| **clr** | rgb, hex, hsv |
| **col** | new, get, set, remove, len, keys, values |
| **con** | now, spawn, wait, mutex_new |
| **csv** | parse, write, headers |
| **dfm** | from_csv, shape, select, filter |
| **env** | get, set, load, all |
| **git** | branch, log, status, clone |
| **ioo** | read, write, append, exists, delete, list |

*For complete function reference with signatures and examples, see [docs/MONOLITH_STDLIB.md](./docs/MONOLITH_STDLIB.md)*

---

## Error Handling

Most intrinsic functions return `nil` on error rather than crashing:

```axiom
// File doesn't exist
let content = ioo.read("missing.txt");

if content {
  out "Success:";
  out content;
} else {
  out "File not found";
}

// Type mismatch
let value = "hello";
let as_num = ann.is_num(value);
if as_num {
  out "It's a number";
} else {
  out "Not a number";
}
```

---

## Performance Notes

- **Instant startup:** No module loading overhead
- **Optimized compilation:** Release build with `opt-level=3` and fat LTO
- **Thread-safe collections:** dashmap for concurrent access
- **Parallel operations:** rayon for `alg.map_parallel()`

---

## Troubleshooting

### Problem: "axm not found"

**Solution:** Ensure PATH was updated:
```powershell
# Check if in PATH
$env:Path -split ';' | Where-Object { $_ -like '*axiom*' }

# If empty, add manually:
[Environment]::SetEnvironmentVariable("Path", "$env:Path;C:\Users\ADMIN\Desktop\programming\github_repos\axiom\target\release", "User")
```

Then restart terminal.

### Problem: Script execution disabled

**Solution:** Allow script execution:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Problem: Build fails

**Solution:** Clean and rebuild:
```bash
cargo clean
cargo build --release
```

---

## Examples

Ready-to-run examples in `examples/` directory:

```bash
axm examples/fib.ax
axm examples/io.ax
axm examples/simple.ax
```

---

## Where to Get Help

1. **Module Reference:** [docs/MONOLITH_STDLIB.md](./docs/MONOLITH_STDLIB.md)
2. **Implementation Details:** [axm/src/intrinsics.rs](./axm/src/intrinsics.rs)
3. **Architecture:** [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)

---

## Next Steps

1. ✅ Run `setup.ps1` to install
2. ✅ Try example: `axm examples/fib.ax`
3. ✅ Read [Module Reference](./docs/MONOLITH_STDLIB.md)
4. ✅ Build your own programs!

---

Welcome to the **Axiom Monolith**! 🚀
