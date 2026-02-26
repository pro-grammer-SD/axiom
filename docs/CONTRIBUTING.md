# Contributing to Axiom

Thank you for your interest in contributing to Axiom! This guide explains how to get involved.

## Code of Conduct

- Be respectful and inclusive
- Assume good intent in discussions
- Focus on ideas, not people
- Welcome feedback and different perspectives

## Getting Started

### Prerequisites

- Rust 1.70+
- Cargo
- Git

### Build Development Setup

```bash
git clone <repo-url>
cd axiom
cargo build
```

### Run Tests

```bash
cargo test
cargo test --release
```

### Check Code Quality

```bash
cargo clippy
cargo fmt --check
```

## How to Contribute

### 1. Reporting Bugs

**Before reporting:**
- Check existing issues
- Reproduce with minimal example
- Try latest version

**When reporting:**
- Use clear title
- Include minimal reproducible example
- Note error message and output
- List OS, Rust version, Axiom version

**Example issue:**
```
Title: Parser crashes on empty function arguments

Expected: Parse `fun f() { ret 0; }` successfully
Actual: Panic at parser.rs:247
Error: thread 'main' panicked at 'index out of bounds'

Steps:
1. Create file test.ax with: `fun f() { ret 0; }`
2. Run: `axiom run test.ax`
3. Observe: Crash

Axiom version: 0.1.0
OS: Linux
Rust: 1.80.0
```

### 2. Suggesting Features

**Before suggesting:**
- Check existing feature requests
- Ensure feature aligns with language goals
- Consider implementation complexity

**Feature request template:**
```
Title: Add string interpolation support

Motivation: String concatenation is verbose

Use case:
let name = "Alice";
let msg = f"Hello, {name}!";

Implementation approach:
- Extend lexer to recognize f-strings
- Parse interpolation expressions
- Evaluate expressions at runtime
```

### 3. Documentation Improvements

**Types of improvements:**
- Fix typos and grammar
- Add missing examples
- Clarify confusing sections
- Add code samples

**Submit via:**
1. Edit file in `docs/`
2. Submit pull request
3. Reference issue if applicable

### 4. Code Contributions

#### Code Style

Follow Rust conventions:

```rust
// Use descriptive names
fn parse_function_declaration(tokens: &[Token]) -> Result<Item, ParserError> {
    // Implementation
}

// Doc comments for public items
/// Parse binary operation with precedence climbing
pub fn parse_binary_op(&mut self, min_prec: i32) -> Result<Expr, ParserError> {
    // Implementation
}

// Comments explain why, not what
// Optimize: pre-allocate Vec to avoid reallocations
let mut items = Vec::with_capacity(expected_count);
```

#### Testing

Add tests for new features:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = "fun add(a, b) { ret a + b; }";
        let mut parser = Parser::new(source, 0);
        let items = parser.parse().unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_negative_numbers() {
        let source = "let x = -42;";
        let mut parser = Parser::new(source, 0);
        let items = parser.parse().unwrap();
        // assertions
    }
}
```

#### Commit Messages

Follow conventional commits:

```
feat: add string methods (upper, lower, trim)

Add string manipulation functions to standard library:
- upper() converts to uppercase
- lower() converts to lowercase
- trim() removes whitespace

Fixes #123

Breaking changes: none
```

Types:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `style:` Formatting (no logic change)
- `refactor:` Code restructuring
- `perf:` Performance improvement
- `test:` Add/fix tests
- `build:` Build system changes

#### Pull Request Process

1. **Fork and branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make changes**
   - Follow code style
   - Add tests
   - Update docs
   - Run clippy: `cargo clippy`
   - Format: `cargo fmt`

3. **Commit**
   ```bash
   git commit -m "feat: add feature"
   ```

4. **Push**
   ```bash
   git push origin feature/my-feature
   ```

5. **Create PR**
   - Use PR template
   - Reference related issues
   - Describe changes
   - Request reviews

6. **Address feedback**
   - Test suggestions
   - Respond to comments
   - Push updates
   - Re-request review

### Priority Areas

Help needed in these areas:

#### High Priority

1. **Standard Library Implementation**
   - `std::net` - HTTP, networking
   - `std::json` - JSON parsing
   - `std::sys` - System information

2. **Parser Improvements**
   - Type annotations (planned)
   - Comments (// and /* */)
   - String interpolation
   - Pattern matching

3. **Runtime Features**
   - Async/await support
   - `go` keyword task spawning
   - Borrow checker

#### Medium Priority

1. **Tools**
   - LSP server
   - Language server protocol
   - VS Code extension

2. **Diagnostics**
   - Better error messages
   - Suggestions for fixes
   - Syntax highlighting

3. **Optimization**
   - Constant folding
   - Dead code elimination
   - Inline hints

#### Lower Priority

1. **Backend**
   - WASM compilation
   - LLVM backend
   - Incremental compilation

2. **Documentation**
   - Tutorial expansion
   - Example gallery
   - API documentation

## Architecture Overview

### Compiler Pipeline

```
Source Code
    ↓
[Lexer] - Tokenization with spans
    ↓
[Parser] - Recursive descent, outputs AST
    ↓
[Semantic Analyzer] - Type checking, symbol resolution
    ↓
[IR Generator] - SSA-form intermediate representation
    ↓
[Runtime] - Execution with scope management
```

### Key Modules

- `axiom/src/lexer.rs` - Tokenization
- `axiom/src/parser.rs` - Parsing
- `axiom/src/ast.rs` - AST definitions
- `axiom/src/semantic.rs` - Type checking
- `axiom/src/runtime.rs` - Execution
- `axiom/src/errors.rs` - Error types
- `axiom/src/core/value.rs` - Value representation

### Adding a Feature

Example: Add `abs()` function

1. **Update AST** if needed
   - Modify `axiom/src/ast.rs`

2. **Update Parser**
   - Modify `axiom/src/parser.rs`
   - Add test

3. **Type Check**
   - Modify `axiom/src/semantic.rs`

4. **Implement Runtime**
   - Modify `axiom/src/runtime.rs`
   - Add logic for `abs` evaluation

5. **Add Tests**
   ```rust
   #[test]
   fn test_abs() {
       let source = "let x = abs(-5);";
       // test
   }
   ```

6. **Update Docs**
   - Add to `docs/STDLIB.md`
   - Add examples in `docs/EXAMPLES.md`

## Testing Standards

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_feature() {
        // arrange
        let input = "...";
        
        // act
        let result = parse(input);
        
        // assert
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```bash
# In examples/ directory
$ axiom run test_feature.ax
# verify output
```

### Test Coverage

Run with coverage:
```bash
cargo tarpaulin --out Html
```

## Performance Considerations

When contributing:
- Profile before optimizing
- Avoid unnecessary clones
- Use references where possible
- Benchmark changed algorithms

```bash
# Build release binary
cargo build --release --bin axiom

# Benchmark
time ./target/release/axiom run large_script.ax
```

## Documentation Standards

- Write clear, accessible docs
- Include examples
- Keep up-to-date with code
- Use consistent formatting

**Template for new features:**
```markdown
### My Feature Name

Brief description.

**Syntax:**
```axiom
feature syntax
```

**Example:**
```axiom
code example
```

**Related:**
- Related feature
```

## Licensing

By contributing, you agree your work is licensed under the same license as Axiom.

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- GitHub acknowledgments

## Getting Help

- **Questions** - Open discussion issue
- **Design** - Create RFC (Request for Comment)
- **Debugging** - Ask in issues or discussions
- **Mentoring** - Contact maintainers

## Release Process

### Versioning

Axiom uses semantic versioning: MAJOR.MINOR.PATCH

- MAJOR - Breaking changes
- MINOR - New features (backward compatible)
- PATCH - Bug fixes

### Release Checklist

- [ ] All tests pass
- [ ] Documentation updated
- [ ] Changelog written
- [ ] Version bumped
- [ ] Tag created
- [ ] Release published

## Asking Questions

- Check documentation first
- Search existing issues
- Be specific and concise
- Include minimal examples

## Thank You!

Your contributions make Axiom better. We appreciate:
- Bug reports
- Feature suggestions
- Code improvements
- Documentation help
- Language feedback

Join us in building a great language! 🚀
