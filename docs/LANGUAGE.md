# Axiom Language Reference

Complete language specification for Axiom.

## Grammar Overview

```
program        := (item)*
item           := function_decl | lib_decl | statement
function_decl  := "fun" identifier "(" params ")" "{" block "}"
lib_decl       := "lib" identifier ";"
statement      := expr_stmt | let_stmt | return_stmt | if_stmt | while_stmt | for_stmt | block
block          := "{" statement* "}"
```

## Lexical Elements

### Keywords

| Keyword | Purpose | Example |
|---------|---------|---------|
| `let` | Variable binding | `let x = 42;` |
| `fun` | Function declaration | `fun add(a, b)` |
| `ret` | Return from function | `ret x + 1;` |
| `if` | Conditional branch | `if x > 0 { }` |
| `else` | Alternative branch | `else { }` |
| `while` | Loop while condition | `while x < 10 { }` |
| `for` | Loop over collection | `for x in list { }` |
| `in` | Iteration target | `for x in [1,2,3]` |
| `true` | Boolean literal | `let b = true;` |
| `false` | Boolean literal | `let b = false;` |
| `lib` | Library import | `lib std;` |
| `go` | Async task spawn | `go task();` |
| `async` | Async function (planned) | `async fun task() { }` |
| `await` | Wait for async (planned) | `await future;` |
| `std` | Standard library | `lib std;` |

### Identifiers

Names for variables, functions, parameters:

```
identifier := [a-zA-Z_][a-zA-Z0-9_]*
```

Valid identifiers:
- `x`, `y`, `name`
- `my_variable`
- `_private`
- `func1`, `var_2_3`

### Literals

#### Numbers

Floating-point numbers:
```axiom
42
3.14
0.5
1000000
```

#### Strings

Double-quoted with escape sequences:
```axiom
"hello"
"line1\nline2"
"tab\there"
"quote: \"hello\""
"backslash: \\"
```

Escape sequences:
- `\n` - newline
- `\t` - tab
- `\r` - carriage return
- `\\` - backslash
- `\"` - double quote

#### Booleans

```axiom
true
false
```

#### Collections

Lists:
```axiom
[]
[1, 2, 3]
["a", "b", "c"]
[[1, 2], [3, 4]]
```

Maps:
```axiom
{}
// Key-value pairs (planned)
```

## Operators

### Precedence (High to Low)

| Level | Type | Operators | Associativity |
|-------|------|-----------|---------------|
| 1 | Primary | `(expr)`, `[expr]`, `.member` | n/a |
| 2 | Postfix | `func()`, `obj[idx]`, `obj.member` | Left |
| 3 | Unary | `!`, `-` | Right |
| 4 | Multiplicative | `*`, `/`, `%` | Left |
| 5 | Additive | `+`, `-` | Left |
| 6 | Comparison | `<`, `<=`, `>`, `>=` | Left |
| 7 | Equality | `==`, `!=` | Left |
| 8 | Logical AND | `&&` | Left |
| 9 | Logical OR | `\|\|` | Left |
| 10 | Assignment | `=` | Right |

### Arithmetic Operators

```axiom
// Addition
let sum = 10 + 5;      // 15

// Subtraction
let diff = 10 - 5;     // 5

// Multiplication
let prod = 10 * 5;     // 50

// Division
let quot = 10 / 5;     // 2.0

// Modulo
let rem = 10 % 3;      // 1
```

### Comparison Operators

```axiom
// Less than
let lt = 3 < 5;        // true

// Less than or equal
let lte = 5 <= 5;      // true

// Greater than
let gt = 5 > 3;        // true

// Greater than or equal
let gte = 5 >= 5;      // true

// Equal
let eq = 5 == 5;       // true

// Not equal
let ne = 5 != 3;       // true
```

### Logical Operators

```axiom
// AND
let and_val = true && true;    // true

// OR
let or_val = false || true;    // true

// NOT
let not_val = !true;           // false
```

### Assignment Operator

```axiom
let x = 42;            // bind variable
let y = x + 1;         // expression
```

## Statements

### Expression Statement

Any expression can be a statement:

```axiom
let x = 5;
x + 10;
func_call();
```

### Let Binding

Declare and initialize a variable:

```axiom
let name = "Axiom";
let count = 42;
let nums = [1, 2, 3];
```

Variables are immutable (rebinding in inner scopes is allowed):

```axiom
let x = 10;
{
  let x = 20;  // shadows outer x
  // x is 20 here
}
// x is still 10 here
```

### Return Statement

Exit from a function with a value:

```axiom
fun double(n) {
  ret n * 2;
}
```

Implicit return of last expression:

```axiom
fun double(n) {
  n * 2
}
```

### If Statement

Conditional execution:

```axiom
if condition {
  // true branch
}
```

With else:

```axiom
if x > 10 {
  // true branch
} else {
  // false branch
}
```

Conditions must be boolean:

```axiom
let value = if x > 0 { "positive" } else { "non-positive" };
```

### While Loop

Repeat while condition is true:

```axiom
let i = 0;
while i < 10 {
  let i = i + 1;
}
```

Variable shadowing in loop:

```axiom
while condition {
  let x = new_value;  // shadows outer x
}
```

### For Loop

Iterate over a collection:

```axiom
for item in [1, 2, 3, 4, 5] {
  // item is each element
}
```

Works with strings (planned):

```axiom
for char in "hello" {
  // char is each character
}
```

### Block Statement

Group statements:

```axiom
{
  let x = 10;
  let y = 20;
  x + y
}
```

Creates a new scope for variables.

## Expressions

### Primary Expressions

#### Literals

```axiom
42              // number
3.14            // float
"hello"         // string
true            // boolean
false           // boolean
[]              // empty list
[1, 2, 3]       // list with elements
{}              // empty map
```

#### Variables

```axiom
x
name
count
```

#### Parenthesized

```axiom
(x + y) * z
```

### Compound Expressions

#### Function Call

```axiom
func()
add(1, 2)
greet("Alice")
process([1, 2, 3])
```

#### Built-in Functions

Axiom provides built-in functions for common operations:

##### out() - Output

Print a value to standard output.

```axiom
out(42)
out("Hello, World!")
out([1, 2, 3])
out(true)
out({"name": "Alice"})
```

- **Parameters:** One value of any type
- **Returns:** `nil`
- **Type Safe:** Accepts and displays all types correctly
- **Never Panics:** Safely handles any value

##### in() - Input

Read a line from standard input with optional prompt.

```axiom
let name = in()              // Read without prompt
let age = in("Age: ")        // Read with prompt
let color = in("Color: ")    // Prompt can be any type
```

- **Parameters:** Optional one value of any type (used as prompt)
- **Returns:** String (trimmed of whitespace)
- **Type Safe:** Always returns a string; prompt can be any type
- **Never Panics:** Handles EOF and I/O errors gracefully

**Prompt Behavior:**
- If no argument: waits for user input without prompt
- If argument provided: prints argument as prompt, then waits for input
- Prompt is printed without trailing newline
- User input is NOT echoed/printed (standard, secure behavior)
- Input is trimmed of trailing whitespace and newline

##### Other Built-in Functions

- `len(value)` - Get length of string, list, or map
- `type_of(value)` - Get type name as string
- Additional functions planned for future versions

#### List Indexing

```axiom
numbers[0]
names[2]
matrix[0][1]
```

Bounds checking at runtime:

```axiom
let nums = [1, 2, 3];
nums[0]   // 1
nums[3]   // error: index out of bounds
```

#### Member Access

```axiom
object.field
obj.method()
```

#### Binary Operations

```axiom
// Arithmetic
x + y
x - y
x * y
x / y
x % y

// Comparison
x < y
x <= y
x > y
x >= y
x == y
x != y

// Logical
x && y
x || y
```

#### Unary Operations

```axiom
-x        // negation
!x        // logical not
```

## Functions

### Declaration

```axiom
fun name(param1, param2, ...) {
  // function body
  ret result;
}
```

### Parameters

Functions accept zero or more parameters:

```axiom
fun no_params() {
  ret 42;
}

fun one_param(x) {
  ret x * 2;
}

fun multi_params(a, b, c) {
  ret a + b + c;
}
```

### Return Values

Explicit return:

```axiom
fun is_even(n) {
  if n % 2 == 0 {
    ret true;
  } else {
    ret false;
  }
}
```

Implicit return of last expression:

```axiom
fun is_even(n) {
  n % 2 == 0
}
```

### Scope and Closures

Functions can access outer scope:

```axiom
let multiplier = 10;
fun scale(x) {
  ret x * multiplier;
}
```

Inner function definitions:

```axiom
fun outer(x) {
  fun inner(y) {
    ret x + y;
  }
  ret inner(5);
}
```

## Scope and Binding

### Variable Scope

Variables are lexically scoped:

```axiom
let x = 10;           // outer scope
{
  let x = 20;         // inner scope (shadows outer)
  // x is 20 in this block
}
// x is still 10 in outer scope
```

### Global vs Local

Variables at top level are global:

```axiom
let global_var = 42;

fun use_global() {
  ret global_var;
}
```

Local variables in functions:

```axiom
fun local_scope() {
  let local_var = 99;
  ret local_var;
}
// local_var not accessible here
```

### Function Scope

Functions create their own scope:

```axiom
fun func(param) {
  let local = param + 1;
  ret local;
}
```

Parameters are local variables.

## Comments (Planned)

### Line Comments

```axiom
// This is a comment
let x = 42;  // inline comment
```

### Block Comments

```axiom
/* This is a
   block comment */
let x = 42;
```

Nested block comments:

```axiom
/* outer /* inner */ outer */
```

## Type Annotations (Planned)

```axiom
fun add(a: Num, b: Num) -> Num {
  ret a + b;
}

let x: Num = 42;
let names: List<Str> = ["Alice", "Bob"];
```

## Advanced Features (Planned)

### Async/Await

```axiom
async fun fetch_data() {
  let result = await async_call();
  ret result;
}

go fetch_data();
```

### Pattern Matching

```axiom
match value {
  1 => "one",
  2 => "two",
  _ => "other"
}
```

### Try-Catch (Planned)

```axiom
try {
  risky_operation();
} catch {
  handle_error();
}
```

## Module System (Planned)

```axiom
lib std;
lib std::net;

use std::net::Http;
```

## Macros (Planned)

```axiom
macro assert(condition) {
  if !condition {
    panic!("assertion failed");
  }
}
```
