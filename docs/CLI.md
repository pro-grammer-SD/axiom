# Command-Line Interface (CLI)

The `axiom` tool is the Axiom compiler and runtime. This guide covers all CLI commands.

## Installation

After building, `axiom` binary is at `target/debug/axiom` or `target/release/axiom`.

Add to PATH:

```bash
export PATH="$PATH:$(pwd)/target/release"  # Linux/macOS
set PATH=%PATH%;%cd%\target\release         # Windows
```

## Commands

### Run

Execute an Axiom script.

**Syntax:**
```bash
axiom run <FILE> [ARGS...]
```

**Arguments:**
- `FILE` - Path to `.ax` file to execute
- `ARGS` - Arguments to pass to script (planned)

**Examples:**
```bash
# Run a simple script
axiom run hello.ax

# Run with arguments (planned)
axiom run process.ax input.txt output.txt

# Run from different directory
axiom run ../examples/greet.ax
```

**Output:**
By default, scripts output any print functions and implicit return values.

### Check

Parse and type-check a file without executing.

**Syntax:**
```bash
axiom check <FILE>
```

**Arguments:**
- `FILE` - Path to `.ax` file to check

**Examples:**
```bash
# Check for syntax errors
axiom check main.ax

# Validate without running
axiom check mylib.ax
```

**Output:**
```
✓ No syntax errors found in main.ax
```

Or displays errors:
```
Error:   × Unexpected token
        │ expected 'fun', 'lib', or statement
         ╭─────────────────────────────────
    12   │ if x == 5
         │           ^ missing '{'
         ╰─────────────────────────────────
```

### Format

Reformat code to consistent style.

**Syntax:**
```bash
axiom format [--in-place] <FILE>
```

**Arguments:**
- `FILE` - Path to `.ax` file to format
- `--in-place` / `-i` - Modify file in-place (default: print to stdout)

**Examples:**
```bash
# Show formatted output
axiom format messy.ax

# Format in-place
axiom format --in-place messy.ax

# Short form
axiom format -i messy.ax
```

**Output:**
Normalizes whitespace and indentation.

### REPL

Interactive read-eval-print loop for experimentation.

**Syntax:**
```bash
axiom repl
```

**No arguments.**

**Examples:**
```bash
$ axiom repl
Axiom REPL (type 'exit' to quit)
axiom> let x = 42;
axiom> let y = x + 8;
axiom> if y > 40 { true } else { false }
true
axiom> exit
Goodbye!
```

**Features:**

- **Line editing** - Readline support (arrow keys, Ctrl+A/E, etc.)
- **History** - Access with up/down arrows
- **Multi-line** - Continue expressions across lines (planned)
- **Variables persist** - Variables defined in REPL remain for later lines

**Commands:**

- `exit` or `quit` - Exit REPL
- Blank line - Ignored; continue with next input
- Any expression - Evaluated and printed

### Input and Output Functions

The REPL and scripts support two built-in I/O functions for type-safe input/output.

#### out() - Print Values

Print any value to standard output.

**Syntax:**
```
out(expression)
```

**Returns:** `nil`

**Examples:**
```bash
axiom> out("Hello, World!")
Hello, World!

axiom> let x = 42;
axiom> out(x)
42

axiom> out(true)
true

axiom> let items = [1, 2, 3];
axiom> out(items)
[1, 2, 3]

axiom> let person = {"name": "Alice", "age": 30};
axiom> out(person)
{name: Alice, age: 30}
```

**Behavior:**
- Prints the value followed by newline
- Works with all types: numbers, strings, booleans, lists, maps, nil
- Returns `nil` (not printed in REPL)
- Useful for debugging and user output

#### in() - Read User Input

Read a line from standard input with an optional prompt.

**Syntax:**
```
in()              // Read without prompt
in(prompt_str)    // Read with prompt printed first
```

**Returns:** `String` (trimmed of whitespace and newline)

**Examples:**
```bash
axiom> let name = in();
> Alice
axiom> out(name)
Alice

axiom> let age = in("Enter your age: ");
Enter your age: 30
axiom> out(age)
30

axiom> let color = in("Favorite color: ");
Favorite color: blue
axiom> out(color)
blue
```

**Behavior:**
- Waits for user to type input (input is NOT echoed/printed)
- If no prompt provided: just waits for input
- If prompt provided: prints the prompt immediately (no newline), then waits for input
- Returns the input as a string (trimmed of trailing newline and whitespace)
- User's typing is NOT displayed on screen (normal stdin behavior)
- Can be used in assignments or as function arguments
- Useful for interactive programs with clear prompts

**Type Safety:**
- Prompt can be any value type (numbers, booleans, etc.)
- Input is always returned as a string
- No type coercion or implicit conversions
- Never panics on EOF or I/O errors
- Input echoing is disabled for security (passwords, sensitive data)

#### Interactive Example

**Without prompts:**
```bash
axiom> out("What is your name?")
What is your name?
axiom> let name = in();
> Bob
axiom> out("Hello, ")
Hello, 
axiom> out(name)
Bob
axiom> out("!")
!
```

**With prompts (cleaner):**
```bash
axiom> let name = in("What is your name? ");
What is your name? (user types: Bob, but it's NOT echoed)
axiom> let age = in("Age: ");
Age: (user types: 25, but it's NOT echoed)
axiom> out("Hello, ");
Hello, 
axiom> out(name)
Bob
```

**Note:** User input is NOT printed/echoed to the screen. This is secure and clean behavior.

**Features:**
- Type-safe: `out()` handles any type, `in()` returns string
- Optional prompts: Built-in, no need for separate `out()` call
- No echo: User input is not printed (secure, clean)
- No panics: Errors handled gracefully
- Battle-tested: Production-grade I/O handling
- Foolproof: Input errors never crash the program

### Package Management

**Syntax:**
```bash
axiom pkg <SUBCOMMAND>
```

#### pkg list

List installed packages.

```bash
axiom pkg list
```

Output:
```
No packages installed
```

#### pkg add

Add a package (planned).

```bash
axiom pkg add user/repo
```

#### pkg install

Install from manifest (planned).

```bash
axiom pkg install
```

Reads from `Axiomite.toml` in current directory.

#### pkg remove

Remove a package (planned).

```bash
axiom pkg remove user/repo
```

## Global Flags

**Help:**
```bash
axiom --help
axiom COMMAND --help
```

**Version:**
```bash
axiom --version
```

**Verbose output (planned):**
```bash
axiom --verbose run main.ax
```

## Environment Variables

### axiom_PATH

Set library search path:

```bash
export axiom_PATH="/usr/local/lib/axiom:~/mylibs"
```

### axiom_DEBUG

Enable debug output:

```bash
export axiom_DEBUG=1
axiom run main.ax
```

## Configuration

### Axiomite.toml

Project configuration file:

```toml
[project]
name = "my-project"
version = "0.1.0"
authors = ["Your Name"]

[dependencies]
std = "0.1"
net = "0.1"

[dev-dependencies]
# test dependencies

[build]
# build settings

[profile.release]
optimize = true
lto = "fat"
```

See [Package Management](STDLIB.md#packages) for details.

## Common Workflows

### Developing a Script

```bash
# Check for errors
axiom check script.ax

# Run it
axiom run script.ax

# Format before committing
axiom format -i script.ax
```

### Interactive Testing

```bash
# Start REPL
axiom repl

# Try some code
axiom> let x = [1, 2, 3];
axiom> for item in x { }
axiom> let double(n) { ret n * 2; }
axiom> double(21)
42
```

### Running Tests (Planned)

```bash
# Run test files
axiom test

# Run specific test
axiom test my_test.ax

# Run with coverage
axiom test --coverage
```

### Building Libraries (Planned)

```bash
# Build library
axiom build

# Build and publish
axiom publish
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Compilation error |
| 2 | Runtime error |
| 3 | File not found |
| 4 | Invalid arguments |

## Performance

### Release Build

Build for maximum performance:

```bash
cargo build --release --bin axiom
./target/release/axiom run script.ax
```

Features:
- LTO (Link-Time Optimization)
- Aggressive inlining
- Dead code elimination
- Assert removal (panic = "abort")

### Debug Build

Faster compilation, slower runtime:

```bash
cargo build --bin axiom
./target/debug/axiom run script.ax
```

## Troubleshooting

### "axiom: command not found"

Build from source and add to PATH:

```bash
cargo build --release --bin axiom
export PATH="$PATH:$(pwd)/target/release"
```

### "File not found: script.ax"

Check file exists and path is correct:

```bash
ls -la script.ax          # Check existence
axiom run ./script.ax       # Use relative path
axiom run /full/path/to/script.ax  # Use absolute path
```

### Script Has Syntax Errors

Get detailed error messages:

```bash
axiom check script.ax
```

### REPL Won't Start

Ensure Axiom is properly built:

```bash
cargo build --bin axiom
./target/debug/axiom repl
```

### Out of Memory

Axiom uses Arc and DashMap for efficiency, but large collections consume memory:

```bash
# Use release build for optimizations
cargo build --release --bin axiom
./target/release/axiom run script.ax
```

## Tips

- Use `axiom check` before `axiom run` to catch errors early
- Use `axiom format -i` to maintain consistent style
- Use `axiom repl` to explore language features
- Build in release mode for production code

## Advanced Usage

### Custom Library Paths

Set search path:

```bash
export axiom_PATH="./libs:~/.axiom/libs"
```

### Benchmarking

Run with timing (planned):

```bash
time axiom run benchmark.ax
```

### Profiling

Generate profile data (planned):

```bash
axiom run --profile script.ax
```

## See Also

- [Getting Started](GETTING_STARTED.md) - Tutorial
- [Language Reference](LANGUAGE.md) - Syntax guide
- [Examples](EXAMPLES.md) - Sample programs
