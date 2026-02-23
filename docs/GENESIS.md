# Axiom Genesis Syntax - Complete Reference

## Overview

Genesis is the hyper-minimalist syntax evolution of Axiom, designed for maximum ergonomics and ease of use. It builds on Axiom's foundation while eliminating unnecessary verbosity and introducing powerful new features.

**Core Principle**: Write less, express more. Every feature is designed to be intuitive and concise.

---

## 1. Optional Keywords: "Hyper-Minimalism"

### 1.1 Function Declarations

Traditional Axiom:
```axiom
fun greet(name) {
    out "Hello, " + name;
}
```

Genesis (keyword optional):
```axiom
greet(name) {
    out "Hello, " + name;
}
```

Both syntaxes are valid and equivalent.

### 1.2 Class Declarations

Traditional Axiom:
```axiom
cls User {
    let name;
    let age;
    
    fun init(name, age) {
        self.name = name;
        self.age = age;
    }
}
```

Genesis (keyword optional):
```axiom
User {
    let name;
    let age;
    
    init(name, age) {
        .name = name;
        .age = age;
    }
}
```

### 1.3 Enum Declarations

Traditional Axiom:
```axiom
enm Status {
    Active,
    Inactive,
    Pending,
}
```

Genesis:
```axiom
Status {
    Active,
    Inactive,
    Pending,
}
```

---

## 2. The Point Reference (`.` as `self`)

Instead of using `self` extensively, use `.` as a direct synonym for `self` in class methods.

### 2.1 Basic Usage

```axiom
User {
    let id;
    let name;
    
    init(id, name) {
        .id = id;        // Instead of: self.id = id
        .name = name;    // Instead of: self.name = name
    }
    
    display() {
        out .id;         // Access instance field
        out .name;
    }
    
    rename(new_name) {
        .name = new_name;  // Modify instance field
    }
}
```

### 2.2 Method Chaining with `.`

```axiom
obj.rename("Alice");
obj.display();
```

### 2.3 Standalone `.` Reference

In methods, a bare `.` refers to the current instance:

```axiom
authenticate() {
    if .is_verified {
        out "Current object is verified";
    }
}
```

---

## 3. Global Built-in Functions

Axiom Genesis provides universal, zero-import built-in functions for type checking and conversion.

### 3.1 type(var) - Type Inspection

Returns the type name of any value as a string.

```axiom
let x = 42;
let s = "hello";
let b = true;

out type(x);    // "Num"
out type(s);    // "Str"
out type(b);    // "Bol"
out type([]);   // "Lst"
```

**Type Names:**
- `Num` - Floating-point number
- `Str` - String
- `Bol` - Boolean
- `Lst` - List/Array
- `Map` - Dictionary/Hash map
- `Fun` - Function value
- `Nil` - Null/None value
- `Instance` - Class instance
- `EnumVariant` - Enum variant

### 3.2 int(var) - Convert to Number

Converts any value to a numeric value.

```axiom
int("123")     // 123.0
int("3.14")    // 3.14
int(true)      // 1.0
int(false)     // 0.0
int("abc")     // nil (invalid)
```

### 3.3 str(var) - Convert to String

Converts any value to its string representation.

```axiom
str(42)        // "42"
str(3.14)      // "3.14"
str(true)      // "true"
str([1, 2])    // "[1, 2]"
str(nil)       // "nil"
```

### 3.4 bol(var) - Convert to Boolean

Converts any value to a boolean using truthy/falsy logic.

```axiom
bol(1)         // true (any non-zero number is true)
bol(0)         // false
bol("text")    // true (non-empty string is true)
bol("")        // false (empty string is false)
bol([1])       // true (non-empty list is true)
bol([])        // false (empty list is false)
bol(nil)       // false
```

### 3.5 avg(list) - Average of Numeric List

Computes the arithmetic mean of numeric values in a list.

```axiom
let numbers = [1, 2, 3, 4, 5];
let average = avg(numbers);  // 3.0

let temps = [98.6, 99.2, 98.1, 99.8];
let avg_temp = avg(temps);   // 98.925
```

### 3.6 sqrt(n) - Square Root

Computes the square root of a number.

```axiom
sqrt(16)    // 4.0
sqrt(2)     // 1.41421356...
sqrt(0)     // 0.0
sqrt(-1)    // NaN (not a number - floating-point behavior)
```

---

## 4. String Methods

All strings support the following methods via dot notation.

### 4.1 .len() - String Length

```axiom
"hello".len()      // 5
"Axiom".len()      // 5
"".len()           // 0
```

### 4.2 .upper() and .lower() - Case Conversion

```axiom
"hello".upper()    // "HELLO"
"WORLD".lower()    // "world"
"AxIoM".upper()    // "AXIOM"
```

### 4.3 .trim() - Remove Whitespace

```axiom
"  hello  ".trim() // "hello"
"\n  text\n".trim()// "text"
```

### 4.4 .align(width, [side]) - String Padding & Alignment

Aligns a string to a specified width with optional alignment.

```axiom
// Left align (default)
"hi".align(10)              // "hi        "

// Right align
"hi".align(10, "right")     // "        hi"
"hi".align(10, "r")         // "        hi"

// Center align
"hi".align(10, "center")    // "    hi    "
"hi".align(10, "c")         // "    hi    "
```

**Parameters:**
- `width` (required): Target width as a number
- `side` (optional): Alignment direction - "left"/"l" (default), "right"/"r", "center"/"c"

### 4.5 .split(delimiter) - Split into List

Splits a string by a delimiter and returns a list of substrings.

```axiom
"a,b,c".split(",")     // ["a", "b", "c"]
"hello world".split(" ")// ["hello", "world"]
"abc".split("")        // ["a", "b", "c"]
"x".split("-")         // ["x"]  (no delimiter found)
```

### 4.6 .contains(substring) - Check Substring

Returns true if the string contains the given substring.

```axiom
"hello".contains("ell")     // true
"world".contains("xyz")     // false
"Axiom".contains("Axiom")   // true
"test".contains("")         // true  (empty string is in all strings)
```

---

## 5. List Methods

All lists support the following methods via dot notation.

### 5.1 .len() - List Length

```axiom
[1, 2, 3].len()    // 3
[].len()           // 0
```

### 5.2 .push(item) - Add to End

Adds an element to the end of the list (mutates in-place).

```axiom
let items = [1, 2];
items.push(3);
out items;         // [1, 2, 3]
```

### 5.3 .pop() - Remove from End

Removes and returns the last element from the list.

```axiom
let items = [1, 2, 3];
let last = items.pop();
out last;          // 3
out items;         // [1, 2]
```

---

## 6. Clean Match Syntax

### 6.1 Wildcard with `els` Keyword

Use `els` (instead of just `_`) for wildcard/default matching:

```axiom
let value = 42;
match value {
    1 => out "One",
    2 => out "Two",
    els => out "Something else",
}
```

Both `_` and `els` work as wildcards. `els` is more explicit and readable.

### 6.2 Implicit Enum Scoping with `.`

Match enum variants using dot notation for implicit scoping:

```axiom
Status {
    Active,
    Inactive,
    Pending,
}

let status = Active;
match status {
    .Active => out "Currently active",
    .Inactive => out "Currently inactive",
    .Pending => out "Awaiting confirmation",
    els => out "Unknown status",
}
```

### 6.3 Enum Variant Matching with `:`

Use `:` for explicit enum variant matching:

```axiom
match status {
    Status:Active => out "Active",
    Status:Inactive => out "Inactive",
    els => out "Unknown",
}
```

---

## 7. String Interpolation

Use `@()` for expression interpolation in strings:

```axiom
let name = "Alice";
let age = 30;
out "Name: @(name), Age: @(age)";  // "Name: Alice, Age: 30"

let sum = 2 + 3;
out "2 + 3 = @(sum)";              // "2 + 3 = 5"
```

For simple variables, use `@varname`:

```axiom
out "Hello @name";                 // "Hello Alice"
```

---

## 8. Complete Example: Genesis in Action

```axiom
// ============================================================================
// Hyper-minimal syntax demonstration
// ============================================================================

// Function without 'fun' keyword
main() {
    out "=== Axiom Genesis Demo ===";
    
    // Classes and classes without 'cls' keyword
    let user = User("Bob", 25);
    user.greet();
    
    // Enums without 'enm' keyword
    let status = Active;
    check_status(status);
    
    // Built-in functions
    out type(42);           // "Num"
    out type("hello");      // "Str"
    
    // String methods
    let text = "axiom";
    out text.upper();       // "AXIOM"
    out text.align(15, "c");// "     axiom     "
    
    // Math functions
    let avg_val = avg([10, 20, 30]);
    out avg_val;            // 20.0
    
    // Pattern matching with els
    match 42 {
        1 => out "One",
        els => out "Not one",
    };
}

// Class without 'cls'
User {
    let name;
    let age;
    
    init(name, age) {
        .name = name;
        .age = age;
    }
    
    greet() {
        out "Hello, I'm @(.name) and I'm @(.age) years old";
    }
}

// Enum without 'enm'
Status {
    Active,
    Inactive,
}

// Function without 'fun'
check_status(status) {
    match status {
        .Active => out "System is active",
        .Inactive => out "System is inactive",
        els => out "Unknown status",
    };
}

main();
```

---

## 9. Backward Compatibility

All Genesis features are **backward compatible**. You can continue using traditional syntax:

```axiom
// All of these are still valid:
fun my_func() { }
cls MyClass { }
enm MyEnum { }
self.field = value
match x {
    _ => ...
}
```

Genesis encourages the minimal versions but doesn't require them.

---

## 10. Migration Guide: Traditional → Genesis

| Traditional | Genesis | Notes |
|-----------|---------|-------|
| `fun name() {}` | `name() {}` | Keyword optional |
| `cls Name {}` | `Name {}` | Keyword optional |
| `enm Name {}` | `Name {}` | Keyword optional |
| `self.field` | `.field` | Shorter reference in methods |
| `_` wildcard | `els` wildcard | More readable, still `_` works |
| `Status:Active` | `.Active` | Implicit scoping in match |
| No method | `.upper()`, `.len()` | String/List methods |
| type_of(x) | type(x) | Renamed for convenience |

---

## 11. Performance Characteristics

- **Syntax compilation**: Minimal overhead - just syntactic sugar
- **Runtime**: No performance difference from traditional Axiom
- **Memory**: No additional memory footprint
- **Type checking**: Same safety guarantees as traditional syntax

---

## 12. Best Practices

1. **Use `.` instead of `self`** in methods for brevity and clarity
2. **Use `els` instead of `_`** in match expressions for readability
3. **Leverage global built-ins** (`type`, `int`, `str`, `avg`) for common operations
4. **Use string methods** for text manipulation without importing external modules
5. **Apply implicit enum scoping** (`.Variant`) in match expressions for DRY code

---

## 13. Common Patterns

### Pattern 1: Type Validation

```axiom
validate(value) {
    match type(value) {
        "Str" => out "String received",
        "Num" => out "Number received",
        els => out "Other type",
    };
}
```

### Pattern 2: Safe Type Conversion

```axiom
safe_to_int(value) {
    let num = int(value);
    if type(num) == "Num" {
        ret num;
    }
    ret 0;
}
```

### Pattern 3: String Formatting

```axiom
format_table(label, value) {
    let formatted = label.align(20, "right") + " | " + str(value);
    out formatted;
}
```

### Pattern 4: Average with Fallback

```axiom
get_average(numbers) {
    if numbers.len() == 0 {
        ret 0;
    }
    ret avg(numbers);
}
```

---

## 14. Limitations & Known Constraints

1. **Bare `.`**: The `.` reference only works inside methods; standalone `.` may conflict with number literals
2. **Implicit enum scoping**: `.Variant` requires the enum to be in scope
3. **Method overloading**: Not currently supported; avoid naming conflicts
4. **Native code limits**: JIT compilation not yet integrated for advanced benchmarks

---

## 15. Future Extensions

Planned additions to Genesis:

- `|>` pipe operator for function composition
- `match?` for optional pattern matching
- Lambda shorthand: `x => x * 2`
- Destructuring in match arms
- Extension methods for custom types

---

## Summary

**Genesis** is Axiom's answer to modern language design: minimal syntax, maximum expressiveness. Every feature serves a purpose, every keyword is optional, and every method is at your fingertips.

Write less. Achieve more. This is **Axiom Genesis**.
