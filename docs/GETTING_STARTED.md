# Getting Started with Axiom

This guide will help you install Axiom and write your first program.

## Installation

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Cargo (included with Rust)

### Build from Source

```bash
git clone https://github.com/your-repo/axiom.git
cd axiom
cargo build --release --bin axiom
```

The compiled binary will be at `target/release/axiom`.

### Add to PATH

On Linux/macOS:
```bash
export PATH="$PATH:$(pwd)/target/release"
```

On Windows:
```powershell
$env:Path += ";$(pwd)\target\release"
```

Or copy the binary directly:
```bash
cp target/release/axiom ~/.local/bin/  # Linux/macOS
```

## Your First Program

### Create a File

Create `hello.ax`:
```axiom
let msg = "Hello, World!";
```

### Run It

```bash
axiom run hello.ax
```

## The REPL

For interactive exploration, start the REPL:

```bash
axiom repl
```

This gives you an interactive prompt where you can experiment:

```
axiom> let x = 42;
axiom> let y = x + 8;
axiom> let nums = [1, 2, 3];
axiom> if x > 40 { true } else { false }
true
axiom> exit
```

## Basic Concepts

### Variables

Variables are immutable by default:

```axiom
let name = "Axiom";
let version = "0.1.0";
let count = 42;
```

### Numbers

Axiom has a unified numeric type (floating-point):

```axiom
let pi = 3.14159;
let age = 25;  // stored as 25.0
let large = 1000000;
```

### Strings

String literals use double quotes:

```axiom
let greeting = "Hello, Axiom!";
let escape = "Line 1\nLine 2";
let quoted = "He said \"Hello!\"";
```

### Booleans

```axiom
let is_valid = true;
let is_empty = false;
```

### Lists

Homogeneous collections:

```axiom
let numbers = [1, 2, 3, 4, 5];
let empty = [];
let mixed = ["apple", "banana", "cherry"];
```

### Maps

Key-value collections:

```axiom
let config = {};  // empty map
```

## Operations

### Arithmetic

```axiom
let sum = 10 + 5;       // 15
let diff = 10 - 5;      // 5
let product = 10 * 5;   // 50
let quotient = 10 / 5;  // 2
let remainder = 10 % 3; // 1
```

### Comparison

```axiom
let equal = 5 == 5;         // true
let not_equal = 5 != 3;     // true
let less = 3 < 5;           // true
let less_equal = 5 <= 5;    // true
let greater = 5 > 3;        // true
let greater_equal = 5 >= 5; // true
```

### Logical

```axiom
let and_result = true && false;  // false
let or_result = true || false;   // true
let not_result = !true;          // false
```

## Control Flow

### If Statements

```axiom
if x > 10 {
  // x is greater than 10
}
```

With else:

```axiom
if x > 10 {
  // x is greater than 10
} else {
  // x is 10 or less
}
```

### While Loops

```axiom
let i = 0;
while i < 5 {
  let i = i + 1;
}
```

### For Loops

```axiom
for item in [1, 2, 3, 4, 5] {
  // item is each element
}
```

## Functions

Define functions with `fun`:

```axiom
fun greet(name) {
  ret "Hello, " + name;
}

let greeting = greet("Alice");
```

### Multiple Parameters

```axiom
fun add(a, b) {
  ret a + b;
}

let result = add(10, 20);  // 30
```

### Nested Functions

Functions can contain other functions:

```axiom
fun outer(x) {
  fun inner(y) {
    ret x + y;
  }
  ret inner(5);
}
```

## Input and Output

Axiom provides two built-in functions for interacting with users.

### Your First Output

Use `out()` to print values:

```axiom
out("Hello, World!");
```

The `out()` function prints any value and returns `nil`:

```axiom
// Print different types
out(42);
out(true);
out([1, 2, 3]);
out({"color": "blue"});
```

**In the REPL:**
```
axiom> out("Testing")
Testing
axiom> let x = 42;
axiom> out(x)
42
```

### Reading User Input

Use `in()` to read what the user types:

```axiom
let user_input = in();
out(user_input);
```

The `in()` function waits for the user to type a line and press Enter:

```
axiom> let name = in();
> Alice
axiom> out(name)
Alice
```

#### Using Prompts (Recommended)

For better user experience, provide a prompt string to `in()`:

```axiom
let name = in("What is your name? ");
let age = in("How old are you? ");
```

The prompt is printed immediately without a newline, then Axiom waits for input:

```
axiom> let name = in("What is your name? ");
What is your name? (user types but input is NOT echoed)
axiom> let age = in("How old are you? ");
How old are you? (user types but input is NOT echoed)
```

**Important:** User input is NOT displayed on the screen. This is the standard, secure behavior.

**Prompts are type-safe:**
- Any value can be a prompt (numbers, booleans, etc.)
- The prompt is printed as-is
- Input is always returned as a string
- User's typing is invisible (secure, useful for passwords and sensitive data)

### Interactive Greetings Program

Here's a complete program that greets a user:

**greet.ax:**
```axiom
// Ask for the user's name
out("What is your name?");
let name = in();

// Greet them
out("Hello, ");
out(name);
out("! Welcome to Axiom!");
```

**Running it:**
```bash
$ axiom run greet.ax
What is your name?
> Bob
Hello, Bob! Welcome to Axiom!
```

### Input Validation

You can validate input using conditional logic:

**validate.ax:**
```axiom
out("Enter a number:");
let input = in();

// Simple validation
if input == "" {
  out("You didn't enter anything!");
} else {
  out("You entered: ");
  out(input);
}
```

### Building Interactive Programs

Three-step pattern for interactive programs:

1. **Prompt** - Use `out()` to ask the user
2. **Read** - Use `in()` to get their response
3. **Process** - Handle the input

**Example:**

```axiom
// 1. Prompt
out("Guess a number (1-10):");

// 2. Read
let guess = in();

// 3. Process (comparison)
let target = "7";
if guess == target {
  out("Correct!");
} else {
  out("Wrong. The number was 7.");
}
```

### Type Safety in I/O

Both functions are completely type-safe:

- `out()` works with **any** value type
- `in()` always returns a **string**
- Never panics or crashes
- Handles errors gracefully

For example, if the user types nothing:

```
axiom> let x = in();
> 
axiom> out(x)

```

The `in()` function returns an empty string, not an error.

## Type System

Axiom has static type checking with inference. Common types:

- `Num` - floating-point numbers
- `Str` - strings
- `Bool` - booleans
- `List` - homogeneous collections
- `Map` - key-value collections
- `Nil` - absence of value

See [TYPES.md](TYPES.md) for details.

## Error Handling

The compiler will report errors with precise locations:

```
Error:   × Undefined variable: missing_var
        │ expected Num, Str, Bool, List, Map, Function, Ptr, Nil, or Type
         ╭──────────────────────────────────────────────────────────────
    1    │ let x = missing_var;
         │         ^^^^^^^^^^^
         ╰──────────────────────────────────────────────────────────────
```

See [ERRORS.md](ERRORS.md) for error reference.

## Next Steps

1. **Learn the Language** - Read [LANGUAGE.md](LANGUAGE.md)
2. **Understand Types** - Study [TYPES.md](TYPES.md)
3. **Explore Examples** - Check [EXAMPLES.md](EXAMPLES.md)
4. **Use the CLI** - Learn [CLI.md](CLI.md) flags
5. **Build Something** - Create your first project!

## Tips

- Use `axiom check <file>` to verify syntax without running
- Use `axiom format <file>` to normalize code
- Use `axiom run <file>` to execute programs
- Use `axiom repl` for interactive exploration

## Troubleshooting

**"axiom: command not found"**
- Build from source: `cargo build --release --bin axiom`
- Add to PATH or use full path: `./target/release/axiom run hello.ax`

**"Undefined variable" errors**
- Ensure you've declared the variable with `let`
- Check spelling and case sensitivity

**"Type mismatch" errors**
- Ensure operations have compatible types
- See [TYPES.md](TYPES.md) for type rules

## Getting Help

- Check [ERRORS.md](ERRORS.md) for error details
- Review [EXAMPLES.md](EXAMPLES.md) for patterns
- Read [LANGUAGE.md](LANGUAGE.md) for complete reference
