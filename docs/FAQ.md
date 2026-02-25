# Frequently Asked Questions (FAQ)

Quick answers to common questions about Axiom.

## Getting Started

### Q: What is Axiom?
**A:** Axiom is a modern, compiled programming language designed for performance, concurrency, and developer productivity. It features static type checking with type inference, a clean syntax, and a comprehensive standard library.

### Q: How do I install Axiom?
**A:** Build from source:
```bash
git clone <repo>
cd axiom
cargo build --release --bin axm
export PATH="$PATH:$(pwd)/target/release"
```

See [Getting Started](GETTING_STARTED.md) for detailed instructions.

### Q: Is Axiom free to use?
**A:** Yes, Axiom is open-source software. See the LICENSE file for details.

### Q: Can I use Axiom in production?
**A:** Axiom is currently in 0.1.0 (early development). Wait for 1.0 release before production use. The language and APIs may change.

### Q: What platforms does Axiom support?
**A:** Currently: Linux, macOS, Windows (via MSVC toolchain).

## Language Features

### Q: Is Axiom statically or dynamically typed?
**A:** Statically typed with automatic type inference. Types are checked at compile time.

### Q: Can I use dynamic typing in Axiom?
**A:** Axiom doesn't support dynamic typing in the traditional sense. All types must be known at compile time. However, type inference makes type declarations optional.

### Q: Does Axiom have null/nil?
**A:** Yes, `nil` represents absence of value. Axiom encourages checking for `nil` explicitly:
```axiom
if value != nil {
  // use value safely
}
```

### Q: How do I handle errors in Axiom?
**A:** Currently via explicit checking. Planned features include Result<T, E> and try-catch blocks.

### Q: Does Axiom have classes/OOP?
**A:** No, Axiom uses functions and data structures instead. Objects are planned for later versions.

### Q: Can I do functional programming in Axiom?
**A:** Yes, Axiom supports first-class functions, closures (planned), and higher-order functions (planned).

## Type System

### Q: What types does Axiom have?
**A:** Axiom has:
- `Num` - floating-point numbers
- `Str` - strings
- `Bool` - booleans (true/false)
- `List` - homogeneous collections
- `Map` - key-value pairs (planned)
- `Nil` - absence of value
- Custom types (planned)

See [Types](TYPES.md) for details.

### Q: Can I create custom types?
**A:** Not yet. Custom types are planned for future releases.

### Q: How do I work with integers?
**A:** All numbers in Axiom are floating-point (`Num`). You can use them as integers:
```axiom
let i = 42;        // stored as 42.0
let j = i % 2;     // modulo operator works
```

### Q: Can I create type aliases?
**A:** Not yet. Type aliases are planned.

### Q: Does Axiom support generics?
**A:** Not yet. Generics are planned for a future release.

## Standard Library

### Q: What's in the standard library?
**A:** Currently:
- Core functions (print, debug, type checking)
- String functions (planned)
- List functions (planned)
- Math functions (planned)

See [Standard Library](STDLIB.md) for complete reference.

### Q: When will more standard library functions be available?
**A:** Prioritized based on user feedback. Submit feature requests for functions you need.

### Q: Can I use external libraries?
**A:** The package system is in development. Currently you can't import external packages.

### Q: How do I contribute to the language?
**A:** See [Contributing](CONTRIBUTING.md) guide. We're actively looking for contributions to the language and toolchain.

## Input and Output

### Q: How do I print values in Axiom?
**A:** Use the `out()` function:
```axiom
out("Hello, World!");
out(42);
out([1, 2, 3]);
out(true);
```

The `out()` function works with all value types and always returns `nil`.

### Q: How do I read user input?
**A:** Use the `in()` function:
```axiom
let name = in();
out(name);
```

Or use `in()` with a prompt for cleaner interaction:
```axiom
let name = in("What is your name? ");
out(name);
```

The `in()` function reads a line from stdin and returns it as a string (trimmed). **Important:** User input is NOT echoed/printed to the screen - it's invisible when typed.

### Q: What's the difference between `in()` with and without a prompt?
**A:** Both read input:
- `in()` - Just waits for input (no prompt printed)
- `in(prompt)` - Prints the prompt first, then waits for input

**Key point:** In both cases, user input is NOT echoed/printed. Only the prompt (if provided) is displayed.

**Without prompt:** You need separate `out()` calls
```axiom
out("Enter your name:");
let name = in();
```

**With prompt:** Cleaner, all-in-one
```axiom
let name = in("Enter your name: ");
```

### Q: Can I use any value as a prompt for `in()`?
**A:** Yes! Any type can be a prompt:
```axiom
let num = in(42);              // Prompt: "42"
let flag = in(true);           // Prompt: "true"
let list = in([1, 2, 3]);      // Prompt: "[1, 2, 3]"
let text = in("Name: ");       // Prompt: "Name: "
```

Note: The prompt is displayed, but the user's input is NOT echoed.

### Q: What's the difference between `out()` and `print()`?
**A:** Both print values. `out()` is the primary built-in function. `print()` is an older name (planned for compatibility). Use `out()` for new code.

### Q: Can `out()` print any value?
**A:** Yes! `out()` handles all types:
- Numbers: `out(42)` → `42`
- Strings: `out("hello")` → `hello`
- Booleans: `out(true)` → `true`
- Lists: `out([1, 2, 3])` → `[1, 2, 3]`
- Maps: `out({"a": 1})` → `{a: 1}`
- Nil: `out(nil)` → (prints nothing)

### Q: Why is my input not showing when I use `in()`?
**A:** This is normal and secure! The `in()` function intentionally does NOT echo/print user input. Only the prompt (if provided) is displayed. User typing is invisible.

This is the standard behavior for input functions across most programming languages and is useful for:
- Passwords and sensitive data (not visible on screen)
- Clean, uncluttered output
- Security and privacy

### Q: Why doesn't my REPL output show the result?
**A:** It should now! The REPL automatically prints non-nil values returned by expressions:
```
axm> 42
42
```

If you're using an older version, upgrade to the latest build.

### Q: How do I make my program interactive?
**A:** Follow the three-step pattern:
1. Prompt with `in(prompt_str)`
2. Read with the return value
3. Process the input

Example with prompts (recommended):
```axiom
let name = in("Enter your name: ");
let age = in("Enter your age: ");
out("Hello, ");
out(name);
out("! You are ");
out(age);
out(" years old.");
```

Or without prompts:
```axiom
out("Enter your name:");
let name = in();
out("Hello, ");
out(name);
```

### Q: What happens if the user types nothing?
**A:** `in()` returns an empty string:
```axiom
let input = in();
if input == "" {
  out("You typed nothing");
}
```

### Q: Can I read multiple lines?
**A:** Call `in()` multiple times:
```axiom
let line1 = in();
let line2 = in();
let line3 = in();
```

### Q: Is input type-safe?
**A:** Yes! `in()` always returns a string. Convert to other types as needed:
```axiom
let age_str = in("Enter age: ");
let age = num(age_str);  // convert to number

if age >= 18 {
  out("Adult");
}
```

Note: The prompt is displayed but user input is not echoed.

### Q: What if the user enters invalid data?
**A:** Type conversion functions return appropriate values or handle errors:
```axiom
let num_str = in();
let num = num(num_str);  // handles conversion
```

### Q: Can I use `out()` and `in()` in REPL?
**A:** Yes! They work the same way as in scripts:
```
axm> out("Test")
Test
axm> let x = in();
> value
axm> out(x)
value
```

### Q: Why is there a `>` prompt when I use `in()`?
**A:** That's your terminal showing you've entered input mode. It's normal behavior when reading from stdin.

### Q: Are `out()` and `in()` battle-tested?
**A:** Yes! They use standard Rust I/O with proper error handling:
- Never panic on bad input
- Handle EOF gracefully
- Works with pipes and redirection
- Thread-safe and production-grade

### Q: Can I redirect input from a file?
**A:** Yes! `in()` reads from stdin, which can be redirected:
```bash
axm run script.ax < input.txt
```

### Q: Can I redirect output to a file?
**A:** Yes! `out()` writes to stdout, which can be redirected:
```bash
axm run script.ax > output.txt
```

## Performance

### Q: Is Axiom fast?
**A:** Axiom compiles to native code with LTO and aggressive optimizations. It should be comparable to Rust for most tasks. See performance comparisons in documentation.

### Q: How do I optimize my code?
**A:** 
1. Use release builds: `cargo build --release --bin axm`
2. Profile with system tools: `time axm run script.ax`
3. Avoid unnecessary list operations
4. Use tail recursion when possible

### Q: What's the startup time?
**A:** REPL startup: ~100-200ms. Pure script execution: ~50-100ms (debug) or ~20-50ms (release).

### Q: How much memory does Axiom use?
**A:** Depends on script size and runtime. A simple script uses ~5-10MB. Arc and DashMap keep memory overhead low.

## Syntax and Grammar

### Q: How do I write comments?
**A:** Comments are planned:
```axiom
// line comment (not yet implemented)
/* block comment (not yet implemented) */
```

Currently unavailable but coming soon.

### Q: Can I import other files?
**A:** The module system is in development. Currently you can't import other `.ax` files.

### Q: Does Axiom have string interpolation?
**A:** Not yet. Use concatenation instead:
```axiom
let name = "Alice";
let msg = "Hello, " + name + "!";
```

String interpolation is planned.

### Q: How do I format strings?
**A:** Use concatenation or the `format` function (planned):
```axiom
let x = 42;
let s = "Answer: " + to_str(x);  // to_str planned
```

### Q: What's the maximum function arguments?
**A:** No hard limit, but keep functions focused. More than 5-7 parameters suggests you should refactor.

## Functions

### Q: Can I have optional parameters?
**A:** Not yet. All parameters are required.

### Q: Can I have variable arguments (...args)?
**A:** Not yet. Use list parameter as workaround:
```axiom
fun process(items) {
  for item in items {
    // process
  }
}
process([1, 2, 3]);
```

### Q: Can I have default parameter values?
**A:** Not yet. Provide overloaded functions or use explicit logic.

### Q: What's the maximum recursion depth?
**A:** Limited by stack size (~2MB default). Tail recursion (planned) will compile to iteration.

### Q: Can I return multiple values?
**A:** Return a list or map:
```axiom
fun get_coords() {
  ret [10, 20];  // or {x: 10, y: 20} with maps
}
let coords = get_coords();
```

## Async and Concurrency

### Q: Does Axiom support async/await?
**A:** Async/await is planned. Currently you can use `go` keyword (planned).

### Q: How do I spawn concurrent tasks?
**A:** The `go` keyword will spawn tasks:
```axiom
go async_task();  // planned
```

Currently implemented but not fully wired to Tokio.

### Q: Can I use Axiom for web servers?
**A:** Yes, once networking is implemented. Currently working on `std::net` module.

### Q: Does Axiom have green threads?
**A:** Axiom uses Tokio for async tasks. Under the hood are OS threads managed by Tokio.

## Debugging

### Q: How do I debug my code?
**A:** Use:
- `print()` for output
- `debug()` for detailed output
- `axm check` to verify syntax

### Q: Can I use a debugger?
**A:** Not yet. Debugger integration is planned.

### Q: How do I see the generated bytecode?
**A:** Bytecode isn't generated (Axiom compiles directly to machine code). Use `--emit ir` flag (planned).

## Errors

### Q: How do I understand error messages?
**A:** See [Error Reference](ERRORS.md) for complete guide.

### Q: How do I report a bug?
**A:** See [Contributing](CONTRIBUTING.md) for bug report guidelines.

## Projects and Examples

### Q: Are there example projects?
**A:** See `examples/` directory:
- `test_simple.ax` - Variables and arithmetic
- `test_if.ax` - Control flow
- More coming soon

### Q: Can I use Axiom for...?

- **CLI tools**: Yes, once package system is ready
- **Web servers**: Yes, once networking is implemented
- **Data processing**: Yes, right now
- **Games**: Limited
- **System programming**: Limited, not designed for this

## Community and Support

### Q: How do I get help?
**A:** 
1. Check documentation
2. Search existing issues
3. Ask in discussions
4. Create new issue if needed

### Q: How can I contribute?
**A:** See [Contributing](CONTRIBUTING.md) guide.

### Q: Where's the community?
**A:** GitHub discussions, issues, and this documentation.

### Q: How do I report security issues?
**A:** Email security@axiom.dev (create this if role exists) or open private security advisory.

## Roadmap

### Q: What's the development roadmap?
**A:** See roadmap in main README.md

### Q: When will feature X be implemented?
**A:** Check the issues and discussions for discussion on specific features.

### Q: Can I influence what gets implemented?
**A:** Yes! Star issues you care about, participate in discussions, and contribute code.

## Version and Compatibility

### Q: What's the current version?
**A:** 0.1.0 (early development). API is unstable.

### Q: Will Axiom 1.0 be backward compatible?
**A:** We'll strive for stability in 1.0, but early versions may break compatibility.

### Q: How often are releases?
**A:** Currently irregular. Will stabilize with 1.0.

## Comparison with Other Languages

### Q: How does Axiom compare to Rust?
**A:** Axiom is simpler and more readable for beginners. Rust is more powerful and safer for systems programming.

### Q: How does Axiom compare to Python?
**A:** Axiom compiles to native code (faster), has static types (safer). Python is more mature with larger ecosystem.

### Q: How does Axiom compare to Go?
**A:** Both support concurrency. Axiom is newer and smaller. Go has mature ecosystem.

## Technical Questions

### Q: What's the memory model?
**A:** Axiom uses Arc for reference counting and DashMap for concurrent access. No garbage collector.

### Q: Is there a borrow checker?
**A:** Not yet. Borrow checking (compile-time safety) is planned.

### Q: What's the calling convention?
**A:** System calling convention (cdecl on x86-64).

### Q: How are lists implemented?
**A:** `Arc<RwLock<Vec<T>>>` on heap for thread-safe access.

### Q: Can I call C code?
**A:** FFI support is planned for a future release.

## See Also

- [Getting Started](GETTING_STARTED.md) - Tutorial
- [Language Reference](LANGUAGE.md) - Complete syntax
- [Standard Library](STDLIB.md) - Built-in functions
- [Contributing](CONTRIBUTING.md) - How to get involved
- [Error Reference](ERRORS.md) - Understanding errors

Haven't found your answer? Open an issue or discussion!
