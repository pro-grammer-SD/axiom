# Axiom Interpreter — Refactoring Changelog

## Production Refactoring — Complete Audit

### 1. Keyword Migration (`lexer.rs`)

**Both `ret` and `return` recognized as `Token::Return`**
```rust
"ret" | "return" => Token::Return,
```
- `ret` is the canonical Axiom keyword
- `return` accepted as an alias for compatibility with legacy scripts
- Neither `ret` nor `return` can be used as identifiers — they emit `Token::Return`
- Prevents `[AXM_201] Undefined variable: 'return'` errors
- Added regression tests: `test_return_keyword()`, `test_nil_keyword()`, `test_hex_literal()`

**`nil` keyword recognized**
```rust
"nil" => Token::Nil,
```

**Hex literals parsed correctly**
```
0xDEAD → 57005.0
0xFF   → 255.0
```

---

### 2. Closure & Scope Resolution (`runtime.rs`, `core/oop.rs`)

**`AxCallable::UserDefined` now carries a `captured` environment**
```rust
UserDefined {
    params: Vec<String>,
    body: Vec<Stmt>,
    captured: HashMap<String, AxValue>,   // ← new field
}
```

**`Expr::Lambda` evaluation snapshots the entire env stack**
```rust
Expr::Lambda { params, body, .. } => {
    let mut captured = HashMap::new();
    for frame in &env.frames {
        for (k, v) in frame {
            captured.insert(k.clone(), v.clone());
        }
    }
    Ok(AxValue::Fun(Arc::new(AxCallable::UserDefined { params, body, captured })))
}
```

**`call_value_inner` injects captured vars before params (params shadow captured)**
```rust
AxCallable::UserDefined { params, body, captured } => {
    env.push_frame();
    for (k, v) in captured { env.define(k.clone(), v.clone()); }
    for (p, a) in params.iter().zip(args.iter()) { env.define(p.clone(), a.clone()); }
    let ret = self.exec_block_in_env(body, env)?;
    env.pop_frame();
    Ok(ret.unwrap_or(AxValue::Nil))
}
```

**Variable resolution order: Local → Enclosing (captured) → Global**
- `env.get(name)` traverses frames in reverse order
- `self.globals.get(name)` as final fallback

---

### 3. Parser: Nested `fn` → `let` Rewrite (`parser.rs`)

**Nested function declarations inside blocks rewritten as `Stmt::Let(Lambda)`**
```axiom
fn make_adder(x) {
    fn adder(y) {   // ← becomes: let adder = fn(y) { ... }
        ret x + y
    }
    ret adder
}
```

This enables proper closure capture — `adder` is a lambda that snapshots `x` from the
enclosing `make_adder` scope at definition time.

Key method: `parse_nested_fn_as_let()` — converts `fn name(params) { body }` inside any
block into `Stmt::Let { name, value: Expr::Lambda { params, body } }`.

---

### 4. Higher-Order Function Intercept (`runtime.rs`)

**`alg.map` and `alg.filter` now work with user-defined functions**

Native stdlib functions cannot call user-defined lambdas (they lack runtime context).
The runtime intercepts calls to these higher-order methods and executes user-defined
functions directly with proper scope:

```rust
if matches!(&obj, AxValue::Map(_)) {
    match method.as_str() {
        "map" => {
            // If fn arg is UserDefined, iterate with self.call_value(...)
        }
        "filter" => {
            // If fn arg is UserDefined, filter with self.call_value(...)
        }
    }
}
```

---

### 5. Standard Library Fixes (`intrinsics.rs`)

**Added missing `col.new_map` and `col.new_set` aliases**
```rust
col_map.insert("new_map".to_string(), native("col.new_map", col_new));  // alias
col_map.insert("new_set".to_string(), native("col.new_set", col_new));  // alias
```

Fixes `lambdas.ax`'s `col.new_map()` call.

---

### 6. Diagnostic System (`diagnostics.rs`)

**Fixed double-bracket bug in `render_rustc_style`**

Before (broken):
```
error[[AXM_101]]: message   ← double brackets!
```

After (correct):
```
error[AXM_101]: message
```

Fix:
```rust
writeln!(out, "\x1b[1;31merror\x1b[0m\x1b[1m[AXM_{:03}]\x1b[0m: {}", code.as_u32(), message)
```

**Miette-powered runtime error output in `main.rs`**

Runtime errors now render through `DiagnosticEngine` → `AxiomDiagnostic` → miette
graphical renderer, producing output like:
```
  × [AXM_201] Undefined variable
   ╭─[script.ax:5:9]
 5 │     print(x)
   ·           ^
   ╰─
  help: Declare the variable with `let name = value` before referencing it.
```

---

### 7. Stack Overflow Detection (`runtime.rs`)

```rust
const MAX_CALL_DEPTH: usize = 1000;

pub fn call_value(&self, func: AxValue, args: Vec<AxValue>, env: &mut Env) -> Result<...> {
    let depth = self.call_depth.get();
    if depth >= MAX_CALL_DEPTH {
        return Err(RuntimeError::GenericError {
            message: "[AXM_408] Call stack overflow — frame limit reached.".into(),
            span: Default::default(),
        });
    }
    self.call_depth.set(depth + 1);
    let result = self.call_value_inner(func, args, env);
    self.call_depth.set(depth);
    result
}
```

---

### 8. Arity Checking (`runtime.rs`)

```rust
if args.len() != params.len() {
    return Err(RuntimeError::ArityMismatch {
        expected: params.len(),
        found: args.len(),
    });
}
```

---

### 9. Regression Tests

**`src/parser.rs`** — 20 new tests covering:
- Nested fn → let/lambda rewrite
- Anonymous lambda in let
- Lambda returning lambda  
- Shadowed variables
- Multiple environment layers
- `ret` / `return` equivalence
- `nil` in expressions
- Hex literals
- Malformed lambda detection
- Class declarations, match, for loops
- Binary op precedence
- Chained method calls

**`tests/integration_closures.rs`** — 12 integration tests covering:
- Closure capture of outer variables
- Multiple independent closures
- Lambda returning lambda (currying)
- Three-level closure chains
- `ret` / `return` keyword parity
- Nil call error
- Arity mismatch error
- Fibonacci correctness
- `alg.range` + `alg.sum`
- `alg.map` with user-defined lambda
- String methods
- Stack overflow detection

---

### Files Changed

| File | Changes |
|------|---------|
| `src/lexer.rs` | `"ret"\|"return"` → `Token::Return`; `"nil"` keyword; hex literals |
| `src/parser.rs` | Nested fn → let/lambda; 20 new regression tests |
| `src/runtime.rs` | Closure capture; higher-order intercept; arity check; stack depth |
| `src/core/oop.rs` | `AxCallable::UserDefined.captured` field |
| `src/intrinsics.rs` | `col.new_map`, `col.new_set` aliases |
| `src/diagnostics.rs` | Fix double-bracket header; update test |
| `src/main.rs` | Miette-rendered runtime errors |
| `tests/integration_closures.rs` | New integration test suite |
| `examples/tests/test_closures.ax` | Closure regression script |
| `examples/tests/test_stdlib.ax` | Stdlib smoke test script |
