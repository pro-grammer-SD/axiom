# Axiom Configuration Reference

All runtime behaviour of the Axiom VM is controlled through a single
key-value configuration file located at **`~/.axiom/conf.txt`**.

The file is seeded with production-safe defaults on first build.  You can
edit it by hand or through the `axiom conf` CLI sub-commands.

---

## `~/.axiom/` Directory Structure

```
~/.axiom/
├── conf.txt          ← runtime configuration (this document)
├── bin/
│   └── axiom           ← installed binary (populated by cargo build --release)
├── lib/              ← reserved for future stdlib extensions
└── cache/            ← bytecode cache (.axc files, when bytecode_cache=on)

~/.axiomlibs/         ← Axiomide package store
```

`~/.axiom/conf.txt` is written by `build.rs` only if it does **not** already
exist, so upgrading the compiler never overwrites user customisations.

---

## CLI Quick Reference

```bash
# Read / write
axiom conf get  nan_boxing             # print current value
axiom conf set  nan_boxing=true        # update a property (saves immediately)
axiom conf list                        # show all properties (current vs default)
axiom conf reset                       # restore every property to its default

# Documentation
axiom conf describe peephole_optimizer # full spec for one property
```

---

## Feature Toggle Properties

These six master switches were introduced to give operators a single on/off
dial for each major subsystem.  All default to **`true`** for maximum
performance.

| Property | Default | Category | Description |
|---|---|---|---|
| `nan_boxing` | `true` | VM | Enable NaN-boxed 64-bit value representation.  Primitives (nil, bool, int, float, heap-ptr) are stored as raw `u64` with IEEE 754 NaN tagging — no heap allocation, no indirection. |
| `bytecode_format` | `true` | Bytecode | Execute using the register-based bytecode format (32-bit fixed-width instructions).  Setting `false` falls back to a slow tree-walk interpreter — useful only for debugging the compiler output. |
| `ic_enabled` | `true` | Cache | Master switch for the entire inline-cache subsystem.  Covers property-access ICs, method-call ICs, and binary-op type-specialisation caches (monomorphic → polymorphic → megamorphic). |
| `gc_enabled` | `true` | GC | Master switch for the garbage collector.  **Never set `false` in long-running programs** — objects will leak without bound.  Useful only for benchmarking raw allocation speed in micro-benchmarks. |
| `peephole_optimizer` | `true` | Optimization | Master switch for the full static optimisation pipeline: constant folding, constant propagation, peephole rewriting, jump threading, dead-code elimination, NOP compaction, and superinstruction fusion. |
| `profiling_enabled` | `true` | Profiling | Master switch for the runtime profiling infrastructure: opcode counters, call-site tracking, hot-loop detection, allocation tracking, and flame-graph export.  Overhead is intentionally minimal (~1–3%) so this can remain enabled in production for observability. |

---

## Full Property Reference

### Debug

| Property | Default | Description |
|---|---|---|
| `debug` | `off` | Master debug switch.  Enables runtime assertions, opcode tracing, GC event logging, and bounds checking. |
| `opcode_trace` | `off` | Trace every executed opcode to stderr (only when `debug=on`). |
| `gc_verbose` | `off` | Print GC events (minor/major collections, pause times, nursery stats). |
| `bounds_check` | `on` | Enable array/list bounds checking. |
| `stack_trace_on_error` | `on` | Print a full call stack trace on runtime errors. |

### Inline Cache

| Property | Default | Description |
|---|---|---|
| `inline_cache` | `on` | Enable inline caches for property access. |
| `poly_ic_size` | `4` | Maximum shapes in a polymorphic IC before going megamorphic (range 1–8). |
| `call_ic` | `on` | Enable inline caches for method calls. |

### Garbage Collector

| Property | Default | Description |
|---|---|---|
| `gc_mode` | `generational` | GC mode: `none`, `simple` (mark-sweep), `generational` (young+old gen). |
| `nursery_size_kb` | `2048` | Young generation nursery size in KB (range 256–65536). |
| `gc_parallel` | `off` | Run GC on a background thread (experimental). |

### Optimization

| Property | Default | Description |
|---|---|---|
| `constant_folding` | `on` | Fold constant arithmetic at compile time. |
| `peephole` | `on` | Peephole instruction rewriting. |
| `dead_code` | `on` | Remove unreachable instructions after unconditional jumps/returns. |
| `jump_threading` | `on` | Redirect jump chains to their final destination. |
| `superinstructions` | `on` | Fuse 2–3 opcode patterns into single superinstructions. |
| `opt_level` | `2` | 0=none, 1=peephole only, 2=full pipeline, 3=aggressive (experimental). |

### Type Specialization

| Property | Default | Description |
|---|---|---|
| `quickening` | `on` | Adaptive opcode specialisation after 16 stable-typed executions. |
| `shape_optimization` | `on` | Use hidden-class shapes for object property layout. |
| `deopt_on_type_change` | `on` | Fall back to generic opcode on type mismatch (safe deoptimisation). |
| `quicken_threshold` | `16` | Executions before quickening (range 4–256). |

### Profiling

| Property | Default | Description |
|---|---|---|
| `profiling` | `off` | Enable per-run profiling report on exit. |
| `opcode_counters` | `on` | Count executions per opcode type (when `profiling=on`). |
| `hot_loop_detect` | `on` | Track loop back-edges and mark loops as hot. |
| `hot_threshold` | `100` | Back-edge count before a loop is considered hot. |
| `flame_graph` | `off` | Export folded-stacks flame graph on exit (inferno format). |
| `alloc_tracking` | `off` | Track allocation rate (bytes/sec) and object count. |

### Parallelism

| Property | Default | Description |
|---|---|---|
| `parallel_gc` | `off` | Concurrent GC on a background thread (experimental). |
| `simd` | `off` | SIMD acceleration for numeric intrinsics (experimental). |
| `thread_pool_size` | `0` | Rayon thread pool size (0 = auto = num CPUs). |

### Experimental / JIT

| Property | Default | Description |
|---|---|---|
| `jit` | `off` | Experimental tracing JIT for hot loops (UNSTABLE). |
| `trace_formation` | `off` | Trace recording for hot loops (prerequisite for JIT). |
| `aot_specialization` | `off` | Ahead-of-time bytecode specialisation via type inference. |

### Allocator

| Property | Default | Description |
|---|---|---|
| `allocator` | `bump` | Heap allocator: `bump` (arena), `system` (malloc), `pool` (size-class). |
| `string_interning` | `on` | Intern string literals at compile time. |

### Bytecode

| Property | Default | Description |
|---|---|---|
| `bytecode_compression` | `off` | Compress serialised bytecode with LZ4 when caching to disk. |
| `bytecode_cache` | `off` | Cache compiled bytecode to `~/.axiom/cache/<hash>.axc`. |

### VM

| Property | Default | Description |
|---|---|---|
| `max_call_depth` | `500` | Maximum call stack depth before stack overflow error. |
| `register_count` | `256` | Default register count per function frame. |

---

## Cross-Platform Notes

`build.rs` resolves `~/.axiom/` via the following priority order so it works
on Windows, Linux, and macOS without any extra dependencies:

1. `$AXIOM_HOME` environment variable (override for CI/containers)
2. `$USERPROFILE\.axiom` (Windows)
3. `$LOCALAPPDATA\axiom` (Windows fallback)
4. `$HOME/.axiom` (Unix)
5. `.axiom` relative to the current directory (last-resort fallback)

If you need reliable cross-platform home-directory resolution in your own
code, consider using the [`dirs`](https://crates.io/crates/dirs) crate (already
a dependency of `axiom`) or `std::env::var("HOME")` / `"USERPROFILE"`.

---

## File Format

```
# This is a comment
property=value

# Blank lines are ignored
debug=off
nan_boxing=true
gc_mode=generational
nursery_size_kb=2048
```

- One property per line
- `=` as delimiter (no spaces required, but allowed)
- Lines starting with `#` are comments
- Unknown properties are silently ignored on load but rejected by `axiom conf set`
- Boolean properties accept: `on`, `off`, `true`, `false`, `yes`, `no`, `1`, `0`
