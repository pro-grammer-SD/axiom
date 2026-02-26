# Command-Line Interface (CLI)

The `axm` tool is the Axiom compiler and runtime. This guide covers all CLI commands.

## Installation

After building, `axm` binary is at `target/debug/axm` or `target/release/axm`.

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
axm run <FILE> [ARGS...]
```

**Arguments:**
- `FILE` - Path to `.ax` file to execute
- `ARGS` - Arguments to pass to script (planned)

**Examples:**
```bash
# Run a simple script
axm run hello.ax

# Run with arguments (planned)
axm run process.ax input.txt output.txt

# Run from different directory
axm run ../examples/greet.ax
```

**Output:**
By default, scripts output any print functions and implicit return values.

### Check

Parse and type-check a file without executing.

**Syntax:**
```bash
axm check <FILE>
```

**Arguments:**
- `FILE` - Path to `.ax` file to check

**Examples:**
```bash
# Check for syntax errors
axm check main.ax

# Validate without running
axm check mylib.ax
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
axm format [--in-place] <FILE>
```

**Arguments:**
- `FILE` - Path to `.ax` file to format
- `--in-place` / `-i` - Modify file in-place (default: print to stdout)

**Examples:**
```bash
# Show formatted output
axm format messy.ax

# Format in-place
axm format --in-place messy.ax

# Short form
axm format -i messy.ax
```

**Output:**
Normalizes whitespace and indentation.

### REPL

Interactive read-eval-print loop for experimentation.

**Syntax:**
```bash
axm repl
```

**No arguments.**

**Examples:**
```bash
$ axm repl
Axiom REPL (type 'exit' to quit)
axm> let x = 42;
axm> let y = x + 8;
axm> if y > 40 { true } else { false }
true
axm> exit
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
axm> out("Hello, World!")
Hello, World!

axm> let x = 42;
axm> out(x)
42

axm> out(true)
true

axm> let items = [1, 2, 3];
axm> out(items)
[1, 2, 3]

axm> let person = {"name": "Alice", "age": 30};
axm> out(person)
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
axm> let name = in();
> Alice
axm> out(name)
Alice

axm> let age = in("Enter your age: ");
Enter your age: 30
axm> out(age)
30

axm> let color = in("Favorite color: ");
Favorite color: blue
axm> out(color)
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
axm> out("What is your name?")
What is your name?
axm> let name = in();
> Bob
axm> out("Hello, ")
Hello, 
axm> out(name)
Bob
axm> out("!")
!
```

**With prompts (cleaner):**
```bash
axm> let name = in("What is your name? ");
What is your name? (user types: Bob, but it's NOT echoed)
axm> let age = in("Age: ");
Age: (user types: 25, but it's NOT echoed)
axm> out("Hello, ");
Hello, 
axm> out(name)
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
axm pkg <SUBCOMMAND>
```

#### pkg list

List installed packages.

```bash
axm pkg list
```

Output:
```
No packages installed
```

#### pkg add

Add a package (planned).

```bash
axm pkg add user/repo
```

#### pkg install

Install from manifest (planned).

```bash
axm pkg install
```

Reads from `Axiomite.toml` in current directory.

#### pkg remove

Remove a package (planned).

```bash
axm pkg remove user/repo
```

## Global Flags

**Help:**
```bash
axm --help
axm COMMAND --help
```

**Version:**
```bash
axm --version
```

**Verbose output (planned):**
```bash
axm --verbose run main.ax
```

## Environment Variables

### AXM_PATH

Set library search path:

```bash
export AXM_PATH="/usr/local/lib/axiom:~/mylibs"
```

### AXM_DEBUG

Enable debug output:

```bash
export AXM_DEBUG=1
axm run main.ax
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
axm check script.ax

# Run it
axm run script.ax

# Format before committing
axm format -i script.ax
```

### Interactive Testing

```bash
# Start REPL
axm repl

# Try some code
axm> let x = [1, 2, 3];
axm> for item in x { }
axm> let double(n) { ret n * 2; }
axm> double(21)
42
```

### Running Tests (Planned)

```bash
# Run test files
axm test

# Run specific test
axm test my_test.ax

# Run with coverage
axm test --coverage
```

### Building Libraries (Planned)

```bash
# Build library
axm build

# Build and publish
axm publish
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
cargo build --release --bin axm
./target/release/axm run script.ax
```

Features:
- LTO (Link-Time Optimization)
- Aggressive inlining
- Dead code elimination
- Assert removal (panic = "abort")

### Debug Build

Faster compilation, slower runtime:

```bash
cargo build --bin axm
./target/debug/axm run script.ax
```

## Troubleshooting

### "axm: command not found"

Build from source and add to PATH:

```bash
cargo build --release --bin axm
export PATH="$PATH:$(pwd)/target/release"
```

### "File not found: script.ax"

Check file exists and path is correct:

```bash
ls -la script.ax          # Check existence
axm run ./script.ax       # Use relative path
axm run /full/path/to/script.ax  # Use absolute path
```

### Script Has Syntax Errors

Get detailed error messages:

```bash
axm check script.ax
```

### REPL Won't Start

Ensure Axiom is properly built:

```bash
cargo build --bin axm
./target/debug/axm repl
```

### Out of Memory

Axiom uses Arc and DashMap for efficiency, but large collections consume memory:

```bash
# Use release build for optimizations
cargo build --release --bin axm
./target/release/axm run script.ax
```

## Tips

- Use `axm check` before `axm run` to catch errors early
- Use `axm format -i` to maintain consistent style
- Use `axm repl` to explore language features
- Build in release mode for production code

## Advanced Usage

### Custom Library Paths

Set search path:

```bash
export AXM_PATH="./libs:~/.axiom/libs"
```

### Benchmarking

Run with timing (planned):

```bash
time axm run benchmark.ax
```

### Profiling

Generate profile data (planned):

```bash
axm run --profile script.ax
```

## See Also

- [Getting Started](GETTING_STARTED.md) - Tutorial
- [Language Reference](LANGUAGE.md) - Syntax guide
- [Examples](EXAMPLES.md) - Sample programs
