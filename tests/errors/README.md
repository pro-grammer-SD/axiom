# tests/errors/ — AXM Error Validation Suite

Each `.ax` file in this directory deliberately triggers a specific `[AXM_NNN]` error code.
Run them with `axiom run <file>` to verify the diagnostic engine outputs the correct code and span.

## File Index

| File | Code | Error Name | Trigger |
|------|------|------------|---------|
| `axm_101.ax` | AXM_101 | UnexpectedToken | Illegal character `§` |
| `axm_102.ax` | AXM_102 | UnterminatedString | String never closed |
| `axm_103.ax` | AXM_103 | InvalidNumber | `12.3.4` double decimal |
| `axm_105.ax` | AXM_105 | UnexpectedEof | Block never closed |
| `axm_200.ax` | AXM_200 | UndefinedIdentifier | Typo with Levenshtein hint |
| `axm_201.ax` | AXM_201 | UndefinedVariable | Used before declaration |
| `axm_202.ax` | AXM_202 | ArityMismatch | Too many arguments |
| `axm_203.ax` | AXM_203 | TypeMismatch | `int - str` |
| `axm_401.ax` | AXM_401 | NotCallable | Integer called as function |
| `axm_402.ax` | AXM_402 | NilCall | Nil closure binding |
| `axm_403.ax` | AXM_403 | DivisionByZero | `x / 0` |
| `axm_404.ax` | AXM_404 | IndexOutOfBounds | `list[99]` on len-3 list |
| `axm_408.ax` | AXM_408 | StackOverflow | Infinite recursion |
| `axm_502.ax` | AXM_502 | UsbError | Non-existent USB device |
| `axm_601.ax` | AXM_601 | ModuleNotFound | Unknown `@phantom_module` |
| `axm_603_a.ax` | AXM_603 | CircularImport | A→B→A cycle (run A) |

## Expected Output Format

```
  × [AXM_402] Attempt to call nil value — check parent-scope binding
   ╭─[axm_402.ax:12:16]
12 │         return adder(x)   // adder is nil here — triggers AXM_402
   ·                ^^^^^
   ╰─
  help: Ensure the identifier is defined before use. Closures capture by value.
```
