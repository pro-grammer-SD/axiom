# Axiom Configuration & Tuning

## Config File: ~/.axiom/conf.txt

Generated automatically by `cargo build --release`. Edit manually or use:

```bash
axiom conf list                    # show all settings
axiom conf get debug               # get one value
axiom conf set debug=on            # set a value
axiom conf reset                   # restore all defaults
axiom conf describe gc_mode        # show docs for a property
```

## Key Settings

```ini
# VM
max_call_depth=500         # Frame limit before AXM_408
register_count=256         # Registers per call frame

# Debug
debug=off                  # Verbose output
opcode_trace=off           # Print each opcode
stack_trace_on_error=on    # Print frames on AXM_408

# GC
gc_mode=generational       # generational | mark-sweep | ref-count
nursery_size_kb=2048       # Young gen size

# Optimization
opt_level=2                # 0=none 1=basic 2=aggressive
constant_folding=on
superinstructions=on       # Fuse opcodes: AddInt+Imm -> AddIntImm
quickening=on              # Type-specialized opcodes

# TUI
# (No conf entries — dashboard FPS is hardcoded to 60 currently)

# JIT (experimental)
jit=off
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `AXIOM_HOME` | Override `~/.axiom` |
| `AXIOM_LIBS` | Override `~/.axiomlibs` |
| `AXIOM_DEBUG` | `1` = VM trace |
| `AXIOM_NO_COLOR` | Disable ANSI |
| `AXIOM_STACK_DEPTH` | Override max_call_depth |

## Binary Install Locations

After `cargo build --release`:

| File | Location |
|------|----------|
| `axiom` | `~/.axiom/bin/axiom` |
| `axm` (alias) | `~/.axiom/bin/axm` |
| config | `~/.axiom/conf.txt` |
| packages | `~/.axiomlibs/<user>/<repo>/` |

## Performance Tips

### Integer Fast Path

`Val::Int(i64)` avoids f64 boxing. Specialized opcodes:
- `AddInt`, `SubInt`, `MulInt` — ~2-5 ns, no alloc
- `AddIntImm` — `R[A] = R[B] + immediate`
- `IncrLocal` / `DecrLocal` — loop counter increment

### TCO Pattern

```axiom
// Frame REUSED — no stack growth
fn loop_fn(n, acc) {
    if n == 0 { return acc }
    return loop_fn(n - 1, acc + n)   // direct tail call
}
```

### Parallel Map

```axiom
let results = alg.parallel_map(large_list, fn(x) { return x * 2 })
// Automatically scales to CPU core count via rayon
```
