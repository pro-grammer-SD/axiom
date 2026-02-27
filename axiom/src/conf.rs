/// Axiom Configuration System
///
/// All configuration is persisted to ~/.axiom/conf.txt
/// Format: property=value (one per line, comments with #)
///
/// CLI:
///   axm conf set property=value
///   axm conf get property
///   axm conf list
///   axm conf reset
///
/// Properties are grouped by subsystem and documented extensively.

use std::collections::HashMap;
use std::path::PathBuf;
use std::fmt;

// ---------------------------------------------------------------------------
// Configuration property definitions
// ---------------------------------------------------------------------------

/// A configuration property with full documentation.
#[derive(Debug, Clone)]
pub struct PropDef {
    pub name: &'static str,
    pub default: &'static str,
    pub description: &'static str,
    pub performance_impact: &'static str,
    pub memory_impact: &'static str,
    pub category: Category,
    pub production_recommended: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Debug,
    Cache,
    GC,
    Optimization,
    Specialization,
    Profiling,
    Parallelism,
    Experimental,
    Allocator,
    Bytecode,
    VM,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// All known configuration properties with full documentation.
pub static ALL_PROPS: &[PropDef] = &[
    // ── Debug ────────────────────────────────────────────────────────────────
    PropDef {
        name: "debug",
        default: "off",
        description: "Master debug switch. Enables runtime assertions, opcode tracing, \
                      GC event logging, and bounds checking. Significant performance overhead.",
        performance_impact: "HIGH (-40% to -80% throughput in debug builds)",
        memory_impact: "MEDIUM (+20% for debug metadata)",
        category: Category::Debug,
        production_recommended: "off",
    },
    PropDef {
        name: "opcode_trace",
        default: "off",
        description: "Trace every executed opcode to stderr. Only active when debug=on. \
                      Prints: IP, opcode name, register values, and timing.",
        performance_impact: "EXTREME (-90% throughput, I/O bound)",
        memory_impact: "LOW",
        category: Category::Debug,
        production_recommended: "off",
    },
    PropDef {
        name: "gc_verbose",
        default: "off",
        description: "Print GC events (minor/major collections, pause times, nursery stats). \
                      Useful for diagnosing GC pressure issues.",
        performance_impact: "LOW (minimal overhead from I/O on GC events)",
        memory_impact: "LOW",
        category: Category::Debug,
        production_recommended: "off",
    },
    PropDef {
        name: "bounds_check",
        default: "on",
        description: "Enable array/list bounds checking. Prevents out-of-bounds reads/writes. \
                      Can be disabled for proven-safe numeric code.",
        performance_impact: "LOW (-5% for array-heavy workloads)",
        memory_impact: "NONE",
        category: Category::Debug,
        production_recommended: "on",
    },
    PropDef {
        name: "stack_trace_on_error",
        default: "on",
        description: "Print a full call stack trace when a runtime error occurs. \
                      Includes file, line, and function name for each frame.",
        performance_impact: "NONE (only on error path)",
        memory_impact: "LOW (keeps frame names in call stack)",
        category: Category::Debug,
        production_recommended: "on",
    },

    // ── Inline Caching ───────────────────────────────────────────────────────
    PropDef {
        name: "inline_cache",
        default: "on",
        description: "Enable inline caches for property access (GetProp/SetProp). \
                      Monomorphic cache hits avoid hash-table lookup entirely. \
                      PIC handles 2–4 shapes before going megamorphic.",
        performance_impact: "HIGH (+30–60% for OOP-heavy code)",
        memory_impact: "LOW (+32 bytes per IC site)",
        category: Category::Cache,
        production_recommended: "on",
    },
    PropDef {
        name: "poly_ic_size",
        default: "4",
        description: "Maximum shapes in a polymorphic inline cache (PIC) before going \
                      megamorphic. Range 1–8. Higher values help diverse OOP code but \
                      increase IC memory and scan cost.",
        performance_impact: "MEDIUM (diminishing returns above 4)",
        memory_impact: "LOW",
        category: Category::Cache,
        production_recommended: "4",
    },
    PropDef {
        name: "call_ic",
        default: "on",
        description: "Enable inline caches for method calls. Caches (receiver_shape, method_ptr) \
                      to avoid dynamic dispatch on hot call sites.",
        performance_impact: "HIGH (+20–40% for method-heavy code)",
        memory_impact: "LOW",
        category: Category::Cache,
        production_recommended: "on",
    },

    // ── Garbage Collector ────────────────────────────────────────────────────
    PropDef {
        name: "gc_mode",
        default: "generational",
        description: "GC mode: 'none' (no GC, leak), 'simple' (mark-sweep), \
                      'generational' (young+old gen), 'incremental' (future). \
                      'generational' gives best throughput for typical workloads.",
        performance_impact: "HIGH (generational 5–15x faster than simple for short-lived objects)",
        memory_impact: "MEDIUM (generational uses 2x nursery memory for semi-space)",
        category: Category::GC,
        production_recommended: "generational",
    },
    PropDef {
        name: "nursery_size_kb",
        default: "2048",
        description: "Young generation nursery size in KB. Larger = fewer minor GCs but \
                      worse cache behavior (nursery should fit in L3). Range: 256–65536.",
        performance_impact: "MEDIUM (too small → GC thrash; too large → cache miss)",
        memory_impact: "DIRECT (configured value + copy space = 2x)",
        category: Category::GC,
        production_recommended: "2048",
    },
    PropDef {
        name: "gc_parallel",
        default: "off",
        description: "Run GC on a background thread (concurrent/parallel GC). \
                      Reduces stop-the-world pauses. Experimental.",
        performance_impact: "MEDIUM (reduces pause time, slight overhead for write barriers)",
        memory_impact: "LOW (+thread stack)",
        category: Category::GC,
        production_recommended: "off",
    },

    // ── Optimization ─────────────────────────────────────────────────────────
    PropDef {
        name: "constant_folding",
        default: "on",
        description: "Fold constant arithmetic expressions at compile time. \
                      E.g., `2 + 3` becomes `5` in the bytecode, never executed at runtime.",
        performance_impact: "HIGH (eliminates runtime arithmetic for literal computations)",
        memory_impact: "LOW (slightly smaller bytecode)",
        category: Category::Optimization,
        production_recommended: "on",
    },
    PropDef {
        name: "peephole",
        default: "on",
        description: "Peephole optimization: replaces short sequences of instructions \
                      with equivalent but cheaper forms. E.g., `Move + Move` round-trip, \
                      `Mul * 1`, `Add + 0`, `Not Not`, etc.",
        performance_impact: "MEDIUM (+5–15% for general code)",
        memory_impact: "LOW (reduces bytecode size)",
        category: Category::Optimization,
        production_recommended: "on",
    },
    PropDef {
        name: "dead_code",
        default: "on",
        description: "Remove unreachable instructions (code after unconditional jumps/returns). \
                      Keeps bytecode compact and improves instruction cache behavior.",
        performance_impact: "LOW (i-cache benefit on hot paths)",
        memory_impact: "LOW (smaller bytecode)",
        category: Category::Optimization,
        production_recommended: "on",
    },
    PropDef {
        name: "jump_threading",
        default: "on",
        description: "Redirect jump chains: if a Jump targets another Jump, redirect \
                      to the final destination. Eliminates wasted dispatch iterations.",
        performance_impact: "LOW–MEDIUM (significant on deep if-else chains)",
        memory_impact: "NONE",
        category: Category::Optimization,
        production_recommended: "on",
    },
    PropDef {
        name: "superinstructions",
        default: "on",
        description: "Fuse common 2–3 opcode patterns into single superinstructions. \
                      E.g., LoadInt+Add → AddIntImm; Lt+JumpFalse → CmpLtJmp. \
                      Reduces dispatch overhead on hot patterns.",
        performance_impact: "HIGH (+15–30% on arithmetic/loop-heavy code)",
        memory_impact: "LOW",
        category: Category::Optimization,
        production_recommended: "on",
    },
    PropDef {
        name: "opt_level",
        default: "2",
        description: "Optimization level: 0=none, 1=peephole only, 2=full pipeline, \
                      3=aggressive (experimental). Level 2 is production-ready.",
        performance_impact: "HIGH (0→2: typically +30–50%)",
        memory_impact: "LOW",
        category: Category::Optimization,
        production_recommended: "2",
    },

    // ── Type Specialization (Adaptive/Quickening) ─────────────────────────────
    PropDef {
        name: "quickening",
        default: "on",
        description: "Adaptive opcode specialization (quickening). After 16 executions \
                      of a binary op with stable types (both int or both float), \
                      replace the generic opcode with a type-specialized version. \
                      Eliminates type dispatch on hot arithmetic.",
        performance_impact: "HIGH (+20–40% for numeric loops)",
        memory_impact: "LOW (per-site type feedback = 12 bytes each)",
        category: Category::Specialization,
        production_recommended: "on",
    },
    PropDef {
        name: "shape_optimization",
        default: "on",
        description: "Use hidden class shapes for object property layout. Objects with \
                      identical property structures share a Shape and can use IC slot-offsets \
                      directly instead of hash lookup.",
        performance_impact: "HIGH (+40–80% for property access in OOP code)",
        memory_impact: "MEDIUM (+shape table per unique object layout)",
        category: Category::Specialization,
        production_recommended: "on",
    },
    PropDef {
        name: "deopt_on_type_change",
        default: "on",
        description: "When a quickened (specialized) opcode encounters a type mismatch, \
                      fall back to the generic opcode (deoptimize). Ensures correctness. \
                      The mispredict cost is acceptable since type changes are rare.",
        performance_impact: "NONE in steady state; rare deopt = ~100ns penalty",
        memory_impact: "NONE",
        category: Category::Specialization,
        production_recommended: "on",
    },
    PropDef {
        name: "quicken_threshold",
        default: "16",
        description: "Number of executions before a generic opcode is quickened. \
                      Lower values quicken faster but risk over-specializing before \
                      types stabilize. Range: 4–256.",
        performance_impact: "MEDIUM (lower = faster specialization, risk of deopt churn)",
        memory_impact: "NONE",
        category: Category::Specialization,
        production_recommended: "16",
    },

    // ── Profiling ─────────────────────────────────────────────────────────────
    PropDef {
        name: "profiling",
        default: "off",
        description: "Enable runtime profiling infrastructure. Activates opcode counters, \
                      call tracking, and hot loop detection. Report printed on exit.",
        performance_impact: "LOW (-2% to -5% for counter overhead)",
        memory_impact: "LOW (+counters per instruction)",
        category: Category::Profiling,
        production_recommended: "off",
    },
    PropDef {
        name: "opcode_counters",
        default: "on",
        description: "Count executions per opcode type (only when profiling=on). \
                      Identifies the top-5% hot opcodes for optimization focus.",
        performance_impact: "VERY LOW (single atomic increment per opcode)",
        memory_impact: "VERY LOW (75 u64 counters = 600 bytes)",
        category: Category::Profiling,
        production_recommended: "on (when profiling=on)",
    },
    PropDef {
        name: "hot_loop_detect",
        default: "on",
        description: "Track loop back-edges and mark loops as hot after N iterations. \
                      Hot loops are candidates for trace formation / JIT compilation. \
                      Enables future JIT integration.",
        performance_impact: "VERY LOW (counter per back-edge)",
        memory_impact: "VERY LOW (hash map of back-edge → count)",
        category: Category::Profiling,
        production_recommended: "on",
    },
    PropDef {
        name: "hot_threshold",
        default: "100",
        description: "Back-edge count before a loop is considered hot. \
                      Lower = detect hot loops faster. Range: 10–10000.",
        performance_impact: "NONE (just a threshold constant)",
        memory_impact: "NONE",
        category: Category::Profiling,
        production_recommended: "100",
    },
    PropDef {
        name: "flame_graph",
        default: "off",
        description: "Export folded-stacks flame graph on exit (inferno format). \
                      Use with: inferno-flamegraph flame.folded > flame.svg",
        performance_impact: "LOW (sampling + I/O on exit)",
        memory_impact: "MEDIUM (stores sampled call stacks)",
        category: Category::Profiling,
        production_recommended: "off",
    },
    PropDef {
        name: "alloc_tracking",
        default: "off",
        description: "Track allocation rate (bytes/sec) and object count. \
                      Reports on exit: total allocations, average object size, rate.",
        performance_impact: "VERY LOW (atomic counter per alloc)",
        memory_impact: "NONE",
        category: Category::Profiling,
        production_recommended: "off",
    },

    // ── Parallelism ───────────────────────────────────────────────────────────
    PropDef {
        name: "parallel_gc",
        default: "off",
        description: "Enable parallel (concurrent) garbage collection. GC work runs \
                      on a background thread to reduce stop-the-world pause times. Experimental.",
        performance_impact: "POSITIVE (reduces pauses) / SLIGHT NEGATIVE (write barriers)",
        memory_impact: "LOW (+GC thread stack)",
        category: Category::Parallelism,
        production_recommended: "off",
    },
    PropDef {
        name: "simd",
        default: "off",
        description: "Enable SIMD acceleration for numeric-heavy intrinsic operations \
                      (ndarray, matrix math, sum). Uses CPU SIMD where available.",
        performance_impact: "HIGH for bulk numeric ops (+2–8x)",
        memory_impact: "NONE",
        category: Category::Parallelism,
        production_recommended: "off (experimental)",
    },
    PropDef {
        name: "thread_pool_size",
        default: "0",
        description: "Size of rayon thread pool for parallel operations. \
                      0 = auto-detect (num CPUs). Used by alg.map_parallel and GC.",
        performance_impact: "HIGH for parallel workloads",
        memory_impact: "MEDIUM (+stack per thread)",
        category: Category::Parallelism,
        production_recommended: "0 (auto)",
    },

    // ── Experimental / JIT ────────────────────────────────────────────────────
    PropDef {
        name: "jit",
        default: "off",
        description: "Enable experimental tracing JIT compilation. Hot loops are traced \
                      and compiled to native code. UNSTABLE — do not use in production.",
        performance_impact: "EXTREME when working (+10–100x for numeric loops)",
        memory_impact: "HIGH (native code pages, trace caches)",
        category: Category::Experimental,
        production_recommended: "off",
    },
    PropDef {
        name: "trace_formation",
        default: "off",
        description: "Enable trace recording for hot loops (prerequisite for JIT). \
                      Records the sequence of instructions for the first 100 iterations, \
                      then compiles the trace.",
        performance_impact: "MEDIUM during trace recording; fast once compiled",
        memory_impact: "MEDIUM (trace buffers)",
        category: Category::Experimental,
        production_recommended: "off",
    },
    PropDef {
        name: "aot_specialization",
        default: "off",
        description: "Ahead-of-time bytecode specialization: analyze entire program before \
                      execution and pre-specialize based on type inference. Reduces JIT warmup.",
        performance_impact: "HIGH (faster startup for typed programs)",
        memory_impact: "LOW (type annotations stored with bytecode)",
        category: Category::Experimental,
        production_recommended: "off",
    },

    // ── Allocator ─────────────────────────────────────────────────────────────
    PropDef {
        name: "allocator",
        default: "bump",
        description: "Heap allocator strategy: 'bump' (fast arena), 'system' (malloc), \
                      'pool' (object pool by size class). 'bump' is fastest for short-lived objects.",
        performance_impact: "HIGH ('bump' is 10–50x faster than 'system' for small objects)",
        memory_impact: "MEDIUM ('bump' reserves nursery upfront)",
        category: Category::Allocator,
        production_recommended: "bump",
    },
    PropDef {
        name: "string_interning",
        default: "on",
        description: "Intern string literals at compile time. Two identical string literals \
                      share a single allocation. Identity comparison replaces equality for interned strings.",
        performance_impact: "HIGH for string-heavy code (+10–30%)",
        memory_impact: "LOW (deduplication saves memory for repeated strings)",
        category: Category::Allocator,
        production_recommended: "on",
    },

    // ── Bytecode ──────────────────────────────────────────────────────────────
    PropDef {
        name: "bytecode_compression",
        default: "off",
        description: "Compress serialized bytecode with LZ4 when caching to disk. \
                      Reduces disk usage and load time for large programs. Small runtime overhead.",
        performance_impact: "NONE at runtime (decompressed once on load)",
        memory_impact: "POSITIVE (70–80% size reduction on disk)",
        category: Category::Bytecode,
        production_recommended: "off (enable for large programs with bytecode caching)",
    },
    PropDef {
        name: "bytecode_cache",
        default: "off",
        description: "Cache compiled bytecode to ~/.axiom/cache/<hash>.axc. \
                      Skip re-compilation if source is unchanged. Speeds up repeated runs.",
        performance_impact: "POSITIVE (eliminates compile time on repeated runs)",
        memory_impact: "DISK (cache files)",
        category: Category::Bytecode,
        production_recommended: "on for production scripts",
    },

    // ── Feature Toggles (master switches) ────────────────────────────────────
    PropDef {
        name: "nan_boxing",
        default: "true",
        description: "Enable NaN-boxing value representation. All primitives (nil, bool, \
                      int, float, heap-ptr) are stored as 64-bit NaN-boxed values. \
                      Disabling falls back to tagged-union AxValue (slower, for debugging).",
        performance_impact: "HIGH (+20–40% for VM dispatch)",
        memory_impact: "POSITIVE (every value = 8 bytes flat, no indirection)",
        category: Category::VM,
        production_recommended: "true",
    },
    PropDef {
        name: "bytecode_format",
        default: "true",
        description: "Use the optimised register-based bytecode format (32-bit fixed-width \
                      instructions). When false the interpreter falls back to tree-walk mode. \
                      Strongly recommended for all production use.",
        performance_impact: "EXTREME (bytecode is 5–20x faster than tree-walk)",
        memory_impact: "LOW (compact bytecode)",
        category: Category::Bytecode,
        production_recommended: "true",
    },
    PropDef {
        name: "ic_enabled",
        default: "true",
        description: "Master toggle for the entire inline-cache subsystem. Covers property \
                      access ICs, method call ICs, and binary-op type-specialisation caches. \
                      Equivalent to setting inline_cache=on and call_ic=on simultaneously.",
        performance_impact: "HIGH (+30–60% for OOP-heavy code)",
        memory_impact: "LOW (+~32 bytes per IC site)",
        category: Category::Cache,
        production_recommended: "true",
    },
    PropDef {
        name: "gc_enabled",
        default: "true",
        description: "Master toggle for the garbage collector. When false all objects are \
                      leaked (useful only for very short-lived scripts or benchmarking). \
                      Setting false in long-running programs causes unbounded memory growth.",
        performance_impact: "POSITIVE when false (no GC overhead) — DANGEROUS for long runs",
        memory_impact: "UNBOUNDED when false",
        category: Category::GC,
        production_recommended: "true",
    },
    PropDef {
        name: "peephole_optimizer",
        default: "true",
        description: "Master toggle for the full static optimisation pipeline (constant \
                      folding, peephole, jump threading, dead-code elimination, \
                      superinstruction fusion). Disable only when debugging raw bytecode.",
        performance_impact: "HIGH (+30–50% overall throughput)",
        memory_impact: "LOW (reduces bytecode size)",
        category: Category::Optimization,
        production_recommended: "true",
    },
    PropDef {
        name: "profiling_enabled",
        default: "true",
        description: "Master toggle for the runtime profiling subsystem. Activates opcode \
                      counters, hot-loop detection, and call-site tracking. Overhead is \
                      minimal so this can stay on in production for observability.",
        performance_impact: "VERY LOW (-1% to -3%)",
        memory_impact: "VERY LOW (+counters per opcode class)",
        category: Category::Profiling,
        production_recommended: "true",
    },

    // ── VM ────────────────────────────────────────────────────────────────────
    PropDef {
        name: "max_call_depth",
        default: "500",
        description: "Maximum call stack depth before stack overflow error. \
                      Increase for deeply recursive programs. Decrease to catch runaway recursion.",
        performance_impact: "NONE (only checked on frame push)",
        memory_impact: "DIRECT (each frame = ~4KB stack + registers)",
        category: Category::VM,
        production_recommended: "500",
    },
    PropDef {
        name: "register_count",
        default: "256",
        description: "Default register count per function frame. \
                      255 max (1 byte operands). Increase for functions with many locals.",
        performance_impact: "LOW (larger frames → more memory, slightly worse cache)",
        memory_impact: "DIRECT (8 bytes × register_count per frame)",
        category: Category::VM,
        production_recommended: "256",
    },
];

// ---------------------------------------------------------------------------
// AxConf — live configuration state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AxConf {
    values: HashMap<String, String>,
}

impl AxConf {
    /// Load configuration from the default config file path.
    /// Falls back to defaults if file not found.
    pub fn load() -> Self {
        let mut conf = AxConf { values: HashMap::new() };
        // Set all defaults first
        for prop in ALL_PROPS {
            conf.values.insert(prop.name.to_string(), prop.default.to_string());
        }

        // Override with file values
        if let Some(path) = Self::config_path() {
            if let Ok(contents) = std::fs::read_to_string(&path) {
                for line in contents.lines() {
                    let line = line.trim();
                    if line.starts_with('#') || line.is_empty() { continue; }
                    if let Some((k, v)) = line.split_once('=') {
                        conf.values.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
            }
        }
        conf
    }

    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".axiom").join("conf.txt"))
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn get_bool(&self, key: &str) -> bool {
        matches!(self.get(key), Some("on") | Some("true") | Some("yes") | Some("1"))
    }

    pub fn get_u32(&self, key: &str, default: u32) -> u32 {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        // Validate key exists
        if !ALL_PROPS.iter().any(|p| p.name == key) {
            return Err(format!("Unknown configuration property: '{}'\nRun `axm conf list` to see all properties.", key));
        }
        self.values.insert(key.to_string(), value.to_string());
        self.save()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("Cannot determine config path")?;
        // Ensure directory exists
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("Cannot create config dir: {}", e))?;
        }

        let mut out = String::new();
        out.push_str("# Axiom Configuration — ~/.axiom/conf.txt\n");
        out.push_str("# Edit manually or use: axm conf set property=value\n");
        out.push_str("# Reset to defaults:   axm conf reset\n\n");

        let mut by_category: Vec<(&PropDef, &str)> = ALL_PROPS.iter()
            .map(|p| (p, self.values.get(p.name).map(|s| s.as_str()).unwrap_or(p.default)))
            .collect();
        by_category.sort_by_key(|(p, _)| format!("{:?}", p.category));

        let mut current_cat: Option<String> = None;
        for (prop, value) in &by_category {
            let cat = format!("{:?}", prop.category);
            if current_cat.as_ref().map(|s| s.as_str()) != Some(cat.as_str()) {
                out.push_str(&format!("\n# ── {} ─────────────────────────\n", cat));
                current_cat = Some(cat);
            }
            out.push_str(&format!("{}={}\n", prop.name, value));
        }

        std::fs::write(&path, &out).map_err(|e| format!("Cannot write config: {}", e))?;
        Ok(())
    }

    /// Reset all properties to defaults
    pub fn reset() -> Result<(), String> {
        let path = Self::config_path().ok_or("Cannot determine config path")?;
        let mut out = String::new();
        out.push_str("# Axiom Configuration — Reset to defaults\n\n");
        for prop in ALL_PROPS {
            out.push_str(&format!("{}={}\n", prop.name, prop.default));
        }
        std::fs::write(&path, &out).map_err(|e| format!("Cannot reset config: {}", e))?;
        println!("✓ Configuration reset to defaults at {}", path.display());
        Ok(())
    }

    /// List all properties with current values
    pub fn list(&self) {
        let mut by_category: Vec<&PropDef> = ALL_PROPS.iter().collect();
        by_category.sort_by_key(|p| format!("{:?}", p.category));

        let mut current_cat: Option<String> = None;

        for prop in by_category {
            let cat = format!("{}", prop.category);
            if current_cat.as_ref().map(|s| s.as_str()) != Some(cat.as_str()) {
                println!();
                println!("── {} ─────────────────────────────────────────────────────────", cat);
                current_cat = Some(cat);
            }
            let current = self.get(prop.name).unwrap_or(prop.default);
            let marker = if current == prop.default { "  " } else { "* " };
            println!("{}  {:<28} = {:<12}  (default: {})", marker, prop.name, current, prop.default);
        }
        println!();
        println!("  * = overridden from default");
        println!("  Config file: {}", Self::config_path().map(|p| p.display().to_string()).unwrap_or_else(|| "N/A".into()));
    }

    /// Show detailed documentation for one property
    pub fn describe(&self, key: &str) {
        let prop = ALL_PROPS.iter().find(|p| p.name == key);
        match prop {
            None => println!("Unknown property: '{}'. Run `axm conf list` to see all.", key),
            Some(p) => {
                let current = self.get(p.name).unwrap_or(p.default);
                println!("┌─ {} ─────────────────────────────────────────────────────────", p.name);
                println!("│  Category:             {}", p.category);
                println!("│  Current value:        {}", current);
                println!("│  Default value:        {}", p.default);
                println!("│  Production default:   {}", p.production_recommended);
                println!("│");
                println!("│  Description:");
                for line in textwrap(p.description, 64) {
                    println!("│    {}", line);
                }
                println!("│");
                println!("│  Performance impact:   {}", p.performance_impact);
                println!("│  Memory impact:        {}", p.memory_impact);
                println!("└────────────────────────────────────────────────────────────────");
            }
        }
    }

    // ── Convenience accessors for VM subsystems ─────────────────────────────

    pub fn debug(&self) -> bool { self.get_bool("debug") }
    pub fn opcode_trace(&self) -> bool { self.debug() && self.get_bool("opcode_trace") }
    pub fn gc_verbose(&self) -> bool { self.get_bool("gc_verbose") }
    pub fn bounds_check(&self) -> bool { self.get_bool("bounds_check") }

    pub fn inline_cache(&self) -> bool { self.get_bool("inline_cache") }
    pub fn call_ic(&self) -> bool { self.get_bool("call_ic") }

    pub fn constant_folding(&self) -> bool { self.get_bool("constant_folding") }
    pub fn peephole(&self) -> bool { self.get_bool("peephole") }
    pub fn dead_code(&self) -> bool { self.get_bool("dead_code") }
    pub fn jump_threading(&self) -> bool { self.get_bool("jump_threading") }
    pub fn superinstructions(&self) -> bool { self.get_bool("superinstructions") }

    pub fn quickening(&self) -> bool { self.get_bool("quickening") }
    pub fn quicken_threshold(&self) -> u32 { self.get_u32("quicken_threshold", 16) }
    pub fn shape_optimization(&self) -> bool { self.get_bool("shape_optimization") }

    pub fn profiling(&self) -> bool { self.get_bool("profiling") }
    pub fn hot_threshold(&self) -> u32 { self.get_u32("hot_threshold", 100) }
    pub fn flame_graph(&self) -> bool { self.get_bool("flame_graph") }
    pub fn alloc_tracking(&self) -> bool { self.get_bool("alloc_tracking") }

    pub fn max_call_depth(&self) -> u32 { self.get_u32("max_call_depth", 500) }

    // ── Feature-toggle accessors ─────────────────────────────────────────────

    /// NaN-boxing value representation enabled.
    pub fn nan_boxing(&self) -> bool { self.get_bool("nan_boxing") }
    /// Bytecode-format execution enabled (vs tree-walk).
    pub fn bytecode_format(&self) -> bool { self.get_bool("bytecode_format") }
    /// Inline-cache subsystem master switch.
    pub fn ic_enabled(&self) -> bool { self.get_bool("ic_enabled") }
    /// Garbage-collector master switch.
    pub fn gc_enabled(&self) -> bool { self.get_bool("gc_enabled") }
    /// Full static optimisation pipeline master switch.
    pub fn peephole_optimizer(&self) -> bool { self.get_bool("peephole_optimizer") }
    /// Runtime profiling subsystem master switch.
    pub fn profiling_enabled(&self) -> bool { self.get_bool("profiling_enabled") }

    pub fn to_opt_config(&self) -> crate::optimizer::OptConfig {
        let master = self.peephole_optimizer();
        crate::optimizer::OptConfig {
            constant_folding:  master && self.constant_folding(),
            constant_prop:     master && self.constant_folding(),
            peephole:          master && self.peephole(),
            jump_threading:    master && self.jump_threading(),
            dead_code:         master && self.dead_code(),
            nop_removal:       master,
            superinstructions: master && self.superinstructions(),
        }
    }

    pub fn to_profiler_config(&self) -> crate::profiler::ProfilerConfig {
        let master = self.profiling_enabled();
        crate::profiler::ProfilerConfig {
            enabled:         master && self.profiling(),
            opcode_counters: master && self.get_bool("opcode_counters"),
            call_tracking:   master,
            hot_loop_detect: master && self.get_bool("hot_loop_detect"),
            flame_graph:     master && self.flame_graph(),
            alloc_tracking:  master && self.alloc_tracking(),
            hot_threshold:   self.hot_threshold(),
            flame_graph_path: None,
        }
    }
}

fn textwrap(s: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let words: Vec<&str> = s.split_whitespace().collect();
    let mut current = String::new();
    for word in words {
        if current.len() + word.len() + 1 > width && !current.is_empty() {
            lines.push(current.trim().to_string());
            current = String::new();
        }
        if !current.is_empty() { current.push(' '); }
        current.push_str(word);
    }
    if !current.is_empty() { lines.push(current); }
    lines
}

// ---------------------------------------------------------------------------
// CLI handlers
// ---------------------------------------------------------------------------

pub fn cmd_conf_set(spec: &str) -> Result<(), String> {
    let (k, v) = spec.split_once('=').ok_or_else(||
        format!("Invalid format. Use: axm conf set property=value\n  Got: '{}'", spec)
    )?;
    let k = k.trim();
    let v = v.trim();
    let mut conf = AxConf::load();
    conf.set(k, v)?;
    println!("✓ Set {}={}", k, v);
    println!("  Config: {}", AxConf::config_path().map(|p| p.display().to_string()).unwrap_or_default());
    Ok(())
}

pub fn cmd_conf_get(key: &str) -> Result<(), String> {
    let conf = AxConf::load();
    let val = conf.get(key).ok_or_else(|| format!("Unknown property: '{}'", key))?;
    let prop = ALL_PROPS.iter().find(|p| p.name == key);
    println!("{}={}", key, val);
    if let Some(p) = prop {
        println!("  default: {}", p.default);
        println!("  category: {}", p.category);
    }
    Ok(())
}

pub fn cmd_conf_list() {
    let conf = AxConf::load();
    conf.list();
}

pub fn cmd_conf_reset() -> Result<(), String> {
    AxConf::reset()
}

pub fn cmd_conf_describe(key: &str) {
    let conf = AxConf::load();
    conf.describe(key);
}
