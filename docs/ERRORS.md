# Error Reference

Complete guide to Axiom error messages and diagnostics.

## Error Format

Axiom provides detailed error messages with source locations:

```
Error:   × Error code and description
        │ explanation
         ╭─────────────────────────────────────
    10   │ let result = "hello" + 5;
         │                      ^
         ╰─────────────────────────────────────
  help: Try converting the types
```

Components:
- **Code** - Unique error identifier (AX###)
- **Message** - Error description
- **Location** - File and source line
- **Context** - Code snippet with pointer
- **Help** - Suggestions for fixing

## Error Codes

### Lexer Errors (AX001-AX005)

#### AX001 - Unexpected Character

**Message:** `Unexpected character in source`

**Cause:** Invalid character in source code.

**Example:**
```axiom
let x = @value;  // @ is unexpected
```

**Fix:** Remove or replace invalid character.

#### AX002 - Unterminated String

**Message:** `Unterminated string literal`

**Cause:** String not closed with quote.

**Example:**
```axiom
let msg = "unclosed string;
```

**Fix:** Close the string with `"`:
```axiom
let msg = "closed string";
```

#### AX003 - Invalid Escape Sequence

**Message:** `Invalid escape sequence in string`

**Cause:** Unknown escape sequence used.

**Example:**
```axiom
let text = "bad\q escape";  // \q is invalid
```

**Fix:** Use valid escape sequences:
- `\n` - newline
- `\t` - tab
- `\\` - backslash
- `\"` - quote

#### AX004 - Unexpected EOF

**Message:** `Unexpected end of file`

**Cause:** Source ends unexpectedly (missing brace, quote, etc.)

**Example:**
```axiom
if x > 0 {
  let y = 1;
// missing closing }
```

**Fix:** Close all blocks:
```axiom
if x > 0 {
  let y = 1;
}
```

#### AX005 - Invalid Number

**Message:** `Invalid number format`

**Cause:** Malformed numeric literal.

**Example:**
```axiom
let x = 123.456.789;  // multiple dots
```

**Fix:** Use valid number format:
```axiom
let x = 123.456;
```

### Parser Errors (AX101-AX106)

#### AX101 - Unexpected Token

**Message:** `Unexpected token 'X' in Y`

**Cause:** Unexpected token in specific context.

**Example:**
```axiom
let x = + 5;  // + unexpected after =
```

**Fix:** Provide valid expression:
```axiom
let x = 5;  // or let x = a + 5;
```

#### AX102 - Expected Token

**Message:** `Expected 'X' but found 'Y'`

**Cause:** Expected token not found.

**Example:**
```axiom
fun func(a b) {  // missing comma
  ret a + b;
}
```

**Fix:** Add missing token:
```axiom
fun func(a, b) {
  ret a + b;
}
```

#### AX103 - Invalid Syntax

**Message:** `Invalid syntax in X`

**Cause:** Statement or expression malformed.

**Example:**
```axiom
let = 10;  // missing variable name
```

**Fix:** Provide valid syntax:
```axiom
let x = 10;
```

#### AX104 - Mismatched Braces

**Message:** `Mismatched braces`

**Cause:** Unclosed or unopened brace, bracket, or paren.

**Example:**
```axiom
if x > 0 {
  let y = [1, 2, 3;  // ] instead of
}
```

**Fix:** Match delimiters:
```axiom
if x > 0 {
  let y = [1, 2, 3];
}
```

#### AX105 - Invalid Assignment

**Message:** `Invalid assignment target`

**Cause:** Cannot assign to this expression.

**Example:**
```axiom
5 = x;  // assigning to literal
```

**Fix:** Assign to valid target:
```axiom
let x = 5;  // or if x is already declared: x = 5;
```

#### AX106 - Invalid Function Declaration

**Message:** `Invalid function declaration`

**Cause:** Function declaration syntax incorrect.

**Example:**
```axiom
fun {  // missing function name
  ret 0;
}
```

**Fix:** Provide function name and parameters:
```axiom
fun my_func(x) {
  ret x;
}
```

### Type Errors (AX201-AX208)

#### AX201 - Type Mismatch

**Message:** `Type mismatch: expected X, found Y`

**Cause:** Operation with incompatible types.

**Example:**
```axiom
let result = "hello" + 5;  // Str + Num invalid
```

**Fix:** Use compatible types:
```axiom
let result = "hello" + "world";  // Str + Str OK
```

#### AX202 - Undefined Variable

**Message:** `Undefined variable: X`

**Cause:** Variable not declared before use.

**Example:**
```axiom
let x = undefined_var + 1;  // undefined_var never declared
```

**Fix:** Declare variable first:
```axiom
let undefined_var = 10;
let x = undefined_var + 1;
```

#### AX203 - Undefined Function

**Message:** `Undefined function: X`

**Cause:** Function called before declaration.

**Example:**
```axiom
let result = unknown_func(5);  // function not defined
```

**Fix:** Define function:
```axiom
fun unknown_func(x) {
  ret x * 2;
}
let result = unknown_func(5);
```

#### AX204 - Invalid Operation

**Message:** `Invalid operation: X Y Z`

**Cause:** Operation not supported for these types.

**Example:**
```axiom
let result = "abc" < "def";  // comparison not supported for Str
```

**Fix:** Use valid operations:
- Numbers: `+`, `-`, `*`, `/`, `%`, comparison, logic
- Strings: `+`, `==`, `!=`
- Booleans: `&&`, `||`, `!`

#### AX205 - Argument Mismatch

**Message:** `Function X expects Y arguments, got Z`

**Cause:** Wrong number of arguments to function.

**Example:**
```axiom
fun add(a, b) {
  ret a + b;
}
let result = add(5);  // missing second argument
```

**Fix:** Provide correct number of arguments:
```axiom
let result = add(5, 10);
```

#### AX206 - Index Out of Bounds

**Message:** `Index out of bounds: X`

**Cause:** List index too large.

**Example:**
```axiom
let nums = [1, 2, 3];
let x = nums[10];  // index 10 doesn't exist
```

**Fix:** Use valid index (0-based):
```axiom
let x = nums[0];   // 0-2 are valid for 3-element list
```

#### AX207 - Invalid Index Type

**Message:** `Invalid index type: expected Num, found X`

**Cause:** List index must be Num.

**Example:**
```axiom
let nums = [10, 20, 30];
let x = nums["first"];  // string index invalid
```

**Fix:** Use Num index:
```axiom
let x = nums[0];
```

#### AX208 - Type Inference Failed

**Message:** `Cannot infer type for X`

**Cause:** Type cannot be determined from context.

**Example:**
```axiom
let x = [];  // empty list - type unknown
```

**Fix:** Provide context or type hint (planned):
```axiom
let x = [1, 2, 3];  // type inferred as List<Num>
```

### Runtime Errors (AX301-AX307)

#### AX301 - Division by Zero

**Message:** `Division by zero`

**Cause:** Attempted division by zero.

**Example:**
```axiom
let x = 10 / 0;  // runtime error
```

**Fix:** Check denominator:
```axiom
if divisor != 0 {
  let result = 10 / divisor;
}
```

#### AX302 - Index Out of Bounds

**Message:** `Index out of bounds: X`

**Cause:** List index out of range at runtime.

**Example:**
```axiom
let nums = [1, 2, 3];
// at runtime if index > 2:
let x = nums[index];  // error
```

**Fix:** Check bounds:
```axiom
if index >= 0 && index < len(nums) {
  let x = nums[index];
}
```

#### AX303 - Invalid Operation

**Message:** `Invalid operation: X Y Z`

**Cause:** Operation failed at runtime.

**Example:**
```axiom
let result = some_function();
// if result is nil:
let x = result + 1;  // error: nil + 1 invalid
```

**Fix:** Check value first:
```axiom
if result != nil {
  let x = result + 1;
}
```

#### AX304 - Stack Overflow

**Message:** `Stack overflow`

**Cause:** Too much recursion.

**Example:**
```axiom
fun infinite() {
  ret infinite();  // infinite recursion
}
```

**Fix:** Add base case:
```axiom
fun factorial(n) {
  if n <= 1 {
    ret 1;  // base case
  }
  ret n * factorial(n - 1);
}
```

#### AX305 - Memory Error

**Message:** `Memory allocation failed`

**Cause:** Out of memory.

**Example:**
```axiom
let huge = [];
// repeatedly:
let huge = huge + [1, 2, 3, 4, 5];  // fill memory
```

**Fix:** Use release build and optimize:
```bash
cargo build --release --bin axm
./target/release/axm run script.ax
```

#### AX306 - Null Pointer

**Message:** `Null pointer dereference`

**Cause:** Accessing nil value.

**Example:**
```axiom
let x = nil;
let y = x + 1;  // error: can't operate on nil
```

**Fix:** Check for nil:
```axiom
if x != nil {
  let y = x + 1;
}
```

#### AX307 - Invalid Conversion

**Message:** `Invalid type conversion`

**Cause:** Cannot convert type.

**Example:**
```axiom
let x = "not a number";
let y = to_num(x);  // invalid conversion
```

**Fix:** Use valid values:
```axiom
let x = "42";
let y = to_num(x);  // valid conversion
```

### Package Errors (AX401-AX406)

#### AX401 - Package Not Found

**Message:** `Package not found: X`

**Cause:** Requested package not installed.

**Example:**
```axiom
lib unknown_package;
```

**Fix:** Install package:
```bash
axm pkg add user/unknown_package
```

Or check spelling:
```axiom
lib std;  // correct package name
```

#### AX402 - Invalid Package Manifest

**Message:** `Invalid package manifest`

**Cause:** Axiomite.toml malformed.

**Example:**
```toml
[project]
name = "myproj
# missing closing ]
```

**Fix:** Fix TOML syntax in Axiomite.toml.

#### AX403 - Dependency Conflict

**Message:** `Dependency conflict: X`

**Cause:** Two packages require conflicting versions.

**Example:**
```toml
[dependencies]
lib_a = "0.1"
lib_b = "0.2"
# lib_a requires lib_c 1.0
# lib_b requires lib_c 2.0
```

**Fix:** Resolve version conflict:
```toml
[dependencies]
lib_a = "0.2"  # compatible version
lib_b = "0.2"
```

#### AX404 - Circular Dependency

**Message:** `Circular dependency detected`

**Cause:** Packages depend on each other.

**Example:**
```
a depends on b
b depends on a
```

**Fix:** Restructure dependencies to break cycle.

#### AX405 - Missing Dependency

**Message:** `Missing dependency: X`

**Cause:** Package required but not declared.

**Example:**
```axiom
lib std::json;  // not in dependencies
```

**Fix:** Add to Axiomite.toml:
```toml
[dependencies]
std = "0.1"
```

#### AX406 - Download Failed

**Message:** `Failed to download package X`

**Cause:** Network error downloading package.

**Example:**
```bash
axm pkg add user/repo  # network error
```

**Fix:** Check network and retry:
```bash
axm pkg add user/repo  # retry
```

## Getting Help

### Check Error Code

Look up error code (AX###) in this document.

### Review Error Location

Error shows exact source line and column.

### Read Help Message

Error includes suggestions for fixing.

### Consult Examples

See [EXAMPLES.md](EXAMPLES.md) for working patterns.

### Check Type Rules

See [TYPES.md](TYPES.md) for type compatibility.

## Common Error Patterns

### Type Mismatch

**Problem:** Operations with incompatible types
```axiom
let x = "number" + 5;  // error: Str + Num
```

**Solution:** Convert types or use compatible operation
```axiom
let x = "number" + "5";  // OK: Str + Str
```

### Index Out of Bounds

**Problem:** Accessing non-existent list element
```axiom
let nums = [1, 2, 3];
let x = nums[5];  // error: index too large
```

**Solution:** Check bounds before access
```axiom
if index >= 0 && index < len(nums) {
  let x = nums[index];
}
```

### Undefined Variable

**Problem:** Using variable before declaration
```axiom
let result = x + 1;  // x not declared yet
```

**Solution:** Declare variable first
```axiom
let x = 10;
let result = x + 1;
```

### Missing Return Value

**Problem:** Function doesn't return value
```axiom
fun get_value() {
  let x = 10;
  // no return statement
}
```

**Solution:** Add return statement
```axiom
fun get_value() {
  let x = 10;
  ret x;
}
```

## Compilation Modes

### Strict Mode

Catch more potential errors:
```bash
axm check script.ax
```

### Debug Mode

More detailed error messages:
```bash
AXM_DEBUG=1 axm run script.ax
```

### Release Mode

Missing some checks for performance:
```bash
cargo build --release --bin axm
./target/release/axm run script.ax
```

## See Also

- [Language Reference](LANGUAGE.md) - Syntax rules
- [Type System](TYPES.md) - Type rules
- [Getting Started](GETTING_STARTED.md) - Tutorial
- [Standard Library](STDLIB.md) - Built-in functions
