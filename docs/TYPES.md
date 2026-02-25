# Type System

Axiom features a static type system with automatic type inference.

## Primitive Types

### Num

Floating-point numbers. All numeric values are `Num`.

```axiom
let x = 42;        // Num (stored as 42.0)
let pi = 3.14159;  // Num
let neg = -10;     // Num
```

#### Operations

| Operation | Result Type | Notes |
|-----------|------------|-------|
| `Num + Num` | Num | Addition |
| `Num - Num` | Num | Subtraction |
| `Num * Num` | Num | Multiplication |
| `Num / Num` | Num | Division |
| `Num % Num` | Num | Modulo |
| `Num < Num` | Bool | Less than |
| `Num <= Num` | Bool | Less than or equal |
| `Num > Num` | Bool | Greater than |
| `Num >= Num` | Bool | Greater than or equal |
| `Num == Num` | Bool | Equality |
| `Num != Num` | Bool | Inequality |

#### Unary Operations

```axiom
-42      // Num negation
```

### Str

Strings are immutable sequences of characters.

```axiom
let greeting = "Hello";
let name = "Alice";
let empty = "";
```

#### Operations

| Operation | Result Type | Notes |
|-----------|------------|-------|
| `Str + Str` | Str | Concatenation |
| `Str == Str` | Bool | Equality |
| `Str != Str` | Bool | Inequality |

#### String Escapes

```axiom
"newline:\n"
"tab:\t"
"quote:\"
"backslash:\\"
```

#### String Coercion

Strings can coerce to Bool:

```axiom
if "" {
  // empty string is false
}
```

### Bool

Boolean values of truth.

```axiom
let active = true;
let done = false;
```

#### Operations

| Operation | Result Type | Notes |
|-----------|------------|-------|
| `Bool && Bool` | Bool | Logical AND |
| `Bool \|\| Bool` | Bool | Logical OR |
| `!Bool` | Bool | Logical NOT |
| `Bool == Bool` | Bool | Equality |
| `Bool != Bool` | Bool | Inequality |

### Nil

Represents the absence of a value. Implicitly returned if no explicit return.

```axiom
fun side_effect() {
  // implicit return Nil
}
```

Nil is falsy:

```axiom
if nil {
  // never executes
}
```

## Composite Types

### List

Homogeneous collection of values.

```axiom
let nums = [1, 2, 3];
let names = ["Alice", "Bob"];
let empty = [];
let nested = [[1, 2], [3, 4]];
```

#### Type

`List<T>` where T is the element type:

```axiom
[1, 2, 3]           // List<Num>
["a", "b"]          // List<Str>
[[1], [2]]          // List<List<Num>>
```

#### Operations

| Operation | Result Type | Notes |
|-----------|------------|-------|
| `List[Num]` | T | Indexing (0-based) |
| `List + List` | List | Concatenation (planned) |
| `List == List` | Bool | Equality |

#### Indexing

```axiom
let items = [10, 20, 30];
let first = items[0];   // 10
let second = items[1];  // 20
let last = items[2];    // 30
```

Out-of-bounds access causes runtime error:

```axiom
let nums = [1, 2, 3];
nums[10];  // error: index out of bounds
```

#### Iteration

```axiom
for item in [1, 2, 3, 4, 5] {
  // item is each element
}
```

### Map

Key-value collections. Currently empty maps only.

```axiom
let config = {};  // empty map
```

Planned features:

```axiom
let user = {
  "name": "Alice",
  "age": 30
};

user["name"]   // "Alice"
```

## Type Inference

Axiom infers types from context without explicit annotations.

### Inference Examples

```axiom
let x = 42;           // inferred: Num
let msg = "hello";    // inferred: Str
let active = true;    // inferred: Bool
let nums = [1, 2, 3]; // inferred: List<Num>
```

### Function Returns

Return types are inferred from function body:

```axiom
fun get_name() {
  ret "Alice";  // inferred: Str
}

fun add(a, b) {
  ret a + b;    // inferred: Num (args are Num)
}
```

## Type Coercion

Axiom follows strict type checking with some coercions:

### Number Coercion

Integers are stored as `Num`:

```axiom
let int_val = 42;      // Num
let float_val = 3.14;  // Num
// both are same type
```

### String Coercion

Strings can be used in boolean context:

```axiom
if "hello" {
  // non-empty string is true
}

if "" {
  // empty string is false
}
```

### Numeric Coercion

Numeric context interprets truthiness:

```axiom
if 0 {
  // false: 0 is falsy
}

if 1 {
  // true: non-zero is truthy
}
```

## Type Checking Rules

### Operator Type Rules

#### Arithmetic

```
Num + Num   -> Num
Num - Num   -> Num
Num * Num   -> Num
Num / Num   -> Num
Num % Num   -> Num
```

Type errors:

```axiom
let x = "hello" + 5;  // error: Str + Num not allowed
```

#### String Concatenation

```
Str + Str   -> Str
```

#### Comparison

```
Num < Num   -> Bool
Num <= Num  -> Bool
Num > Num   -> Bool
Num >= Num  -> Bool
```

Type checking:

```axiom
3 < 5          // Bool (valid)
"a" < "b"      // error: Str comparison not supported
```

#### Equality

```
T == T   -> Bool   (any type)
T != T   -> Bool   (any type)
```

```axiom
5 == 5          // Bool
"abc" == "abc"  // Bool
true == true    // Bool
[1,2] == [1,2]  // Bool
```

#### Logical Operations

```
Bool && Bool  -> Bool
Bool || Bool  -> Bool
!Bool         -> Bool
```

```axiom
true && false           // Bool
let x = 5;
x > 0 && x < 10         // Bool
!(x == 0)               // Bool
```

### Indexing

```
List<T>[Num] -> T
```

```axiom
let nums = [10, 20, 30];
nums[0]        // Num (valid)
nums["x"]      // error: List index must be Num
```

## Generic Types (Planned)

```axiom
fun identity<T>(x: T) -> T {
  ret x;
}

fun first<T>(list: List<T>) -> T {
  ret list[0];
}
```

## Union Types (Planned)

```axiom
fun process(value: Num | Str) {
  // accepts Num or Str
}
```

## Option Type (Planned)

Safe handling of potentially missing values:

```axiom
fun find(list: List<T>, pred: T -> Bool) -> Option<T> {
  // returns Some<T> or None
}
```

## Result Type (Planned)

For error handling:

```axiom
fun divide(a: Num, b: Num) -> Result<Num, Str> {
  if b == 0 {
    ret Err("Division by zero");
  }
  ret Ok(a / b);
}
```

## Type Aliases (Planned)

```axiom
type UserId = Num;
type UserList = List<Num>;

fun get_user(id: UserId) -> User {
  // ...
}
```

## JSON Type (Planned)

For JSON data:

```axiom
lib std::json;

let data: Json = json!({
  "name": "Alice",
  "age": 30
});
```

## Debugging Type Errors

### Example Error

```
Error:   × Type mismatch
        │ expected Num, found Str
         ╭──────────────────────────────────────────────────────────────
    5    │ let result = "hello" + 42;
         │                      ^^
         ╰──────────────────────────────────────────────────────────────
```

### Troubleshooting

1. **Unexpected type mismatch** - Ensure both sides of operator are same type
2. **Undefined variable** - Check spelling and scope
3. **List index error** - Ensure index is Num and in bounds
4. **Type mismatch in function** - Check argument types match declaration

See [ERRORS.md](ERRORS.md) for complete error reference.
