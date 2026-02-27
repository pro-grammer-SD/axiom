# Axiom Syntax Reference

## Literals

| Kind | Example | VM Type |
|------|---------|---------|
| Integer | `42` `-7` | `Val::Int(i64)` |
| Float | `3.14` | `Val::Float(f64)` |
| String | `"hello"` | `Val::Str(Arc<str>)` |
| Bool | `true` `false` | `Val::Bool(bool)` |
| Nil | `nil` | `Val::Nil` |
| List | `[1, 2, 3]` | `Val::List(...)` |

## Operators

| Op | Description |
|----|-------------|
| `+` `-` `*` | Arithmetic (int fast-path: AddInt, SubInt, MulInt) |
| `/` | Division (always float; div-by-zero = AXM_403) |
| `%` | Modulo (Euclidean) |
| `**` | Power |
| `==` `!=` `<` `<=` `>` `>=` | Comparison (int fast-path: LtInt etc.) |
| `and` `or` `not` | Logic (short-circuit) |
| `+` on strings | Concatenation |

## Control Flow

```axiom
if x > 0 { print("pos") } else { print("neg") }

while i < 10 { i = i + 1 }

for item in [1, 2, 3] { print(item) }

match x {
    1 => print("one")
    2 => print("two")
    els => print("other")
}
```

## Classes

```axiom
class Animal {
    fn init(name) { self.name = name }
    fn speak()    { print(self.name) }
}
let a = Animal("Dog")
a.speak()

class Dog ext Animal {
    fn fetch(item) { print(self.name + " fetches " + item) }
}
```

## Error Code Taxonomy

### Lexical (AXM_100-199)

| Code | Trigger | Fix |
|------|---------|-----|
| AXM_101 | Illegal character e.g. `§` | Remove it |
| AXM_102 | Unterminated string | Add closing quote |
| AXM_103 | Invalid number `12.3.4` | One decimal point only |
| AXM_105 | EOF inside block | Close all braces |

### Semantic (AXM_200-299)

| Code | Trigger | Fix |
|------|---------|-----|
| AXM_200 | Typo — Levenshtein suggests nearest match | Check spelling |
| AXM_201 | Variable used before `let` | Add declaration |
| AXM_202 | Wrong argument count | Match signature |
| AXM_203 | Type mismatch e.g. `int - str` | Explicit conversion |

### Runtime (AXM_400-499)

| Code | Name | Trigger | Fix |
|------|------|---------|-----|
| AXM_401 | NotCallable | `42(x)` | Verify it's a function |
| **AXM_402** | **NilCall** | Closure captures nil | Define before use |
| AXM_403 | DivisionByZero | `x / 0` | Guard divisor |
| AXM_404 | IndexOutOfBounds | `list[99]` on short list | Check `alg.len()` |
| AXM_408 | StackOverflow | Infinite recursion | Use TCO / iteration |

### System (AXM_500-599)

| Code | Trigger |
|------|---------|
| AXM_501 | I/O failure |
| AXM_502 | USB device error |
| AXM_503 | Network error |

### Module (AXM_600-699)

| Code | Trigger | Fix |
|------|---------|-----|
| AXM_601 | Module not found | `axiom pkg install <n>` |
| AXM_602 | Version conflict | Pin version |
| AXM_603 | Circular import A->B->A | Extract shared module |

## Diagnostic Output (rustc-grade)

```
error[AXM_402]: Attempt to call nil value — check parent-scope binding
 --> axm_402.ax:12:16
  |
11 |     fn inner(x) {
12 |         return adder(x)
  |                ^^^^^ identifier resolves to nil
  |
  = help: Ensure the identifier is defined before use.
```
