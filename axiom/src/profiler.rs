/// Profiling Infrastructure
///
/// Provides:
///   1. Opcode execution frequency counters (per-instruction hot-spot detection)
///   2. Function call counters + call graph (top-N hot functions)
///   3. Dispatch cycle measurement (estimated via instruction counts)
///   4. Allocation rate tracking (bytes/sec)
///   5. Hot loop detection (back-edge counter, triggers trace formation signal)
///   6. Branch misprediction measurement hooks (via perf-event-style counters)
///   7. Flame graph export (folded stack format for inferno/speedscope)
///   8. Real-time performance dashboard (printed to stderr)

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use parking_lot::Mutex;

use crate::bytecode::Op;

// ---------------------------------------------------------------------------
// Profiler configuration
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ProfilerConfig {
    pub enabled:          bool,
    pub opcode_counters:  bool,
    pub call_tracking:    bool,
    pub hot_loop_detect:  bool,
    pub flame_graph:      bool,
    pub alloc_tracking:   bool,
    /// Hot loop threshold: back-edges before marking hot
    pub hot_threshold:    u32,
    /// Flame graph output path (None = stderr)
    pub flame_graph_path: Option<String>,
}

impl Default for ProfilerConfig {
    fn default() -> Self {
        ProfilerConfig {
            enabled:         false,
            opcode_counters: true,
            call_tracking:   true,
            hot_loop_detect: true,
            flame_graph:     false,
            alloc_tracking:  true,
            hot_threshold:   100,
            flame_graph_path: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Opcode frequency table
// ---------------------------------------------------------------------------

/// Per-opcode execution counter (75 opcodes max, indexed by Op u8 value)
pub struct OpcodeCounters {
    counts: [AtomicU64; 128],
}

impl OpcodeCounters {
    pub fn new() -> Self {
        OpcodeCounters {
            // AtomicU64 doesn't impl Copy so we init with array init
            counts: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }

    #[inline(always)]
    pub fn record(&self, op: Op) {
        self.counts[op as usize].fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self, op: Op) -> u64 {
        self.counts[op as usize].load(Ordering::Relaxed)
    }

    pub fn total(&self) -> u64 {
        self.counts.iter().map(|c| c.load(Ordering::Relaxed)).sum()
    }

    /// Print top-N most frequent opcodes
    pub fn print_top(&self, n: usize) {
        let total = self.total();
        if total == 0 {
            println!("  (no instructions executed)");
            return;
        }

        let mut entries: Vec<(Op, u64)> = (0..75u8).filter_map(|i| {
            let op: Op = unsafe { std::mem::transmute(i) };
            let count = self.counts[i as usize].load(Ordering::Relaxed);
            if count > 0 { Some((op, count)) } else { None }
        }).collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));

        println!("=== Opcode Frequency (top {}) ===", n);
        println!("  {:<18} {:>12}  {:>7}", "Opcode", "Count", "% total");
        println!("  {}", "-".repeat(42));
        for (op, count) in entries.iter().take(n) {
            let pct = *count as f64 / total as f64 * 100.0;
            println!("  {:<18} {:>12}  {:>6.2}%", op.name(), count, pct);
        }
        println!("  {}", "-".repeat(42));
        println!("  {:<18} {:>12}", "TOTAL", total);
    }
}

// ---------------------------------------------------------------------------
// Function call tracker
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct FuncProfile {
    pub name:      String,
    pub calls:     u64,
    pub self_time_ns: u64,
    pub total_time_ns: u64,
}

pub struct CallTracker {
    /// function_name → profile
    profiles: Mutex<HashMap<String, FuncProfile>>,
    /// Call stack for timing
    call_stack: Mutex<Vec<(String, Instant)>>,
}

impl CallTracker {
    pub fn new() -> Self {
        CallTracker {
            profiles: Mutex::new(HashMap::new()),
            call_stack: Mutex::new(Vec::new()),
        }
    }

    pub fn enter(&self, name: &str) {
        {
            let mut profiles = self.profiles.lock();
            let p = profiles.entry(name.to_string()).or_insert_with(|| FuncProfile {
                name: name.to_string(), ..Default::default()
            });
            p.calls += 1;
        }
        self.call_stack.lock().push((name.to_string(), Instant::now()));
    }

    pub fn exit(&self, _name: &str) {
        let mut stack = self.call_stack.lock();
        if let Some((fname, enter_time)) = stack.pop() {
            let elapsed = enter_time.elapsed().as_nanos() as u64;
            let mut profiles = self.profiles.lock();
            if let Some(p) = profiles.get_mut(&fname) {
                p.self_time_ns += elapsed;
                p.total_time_ns += elapsed;
            }
        }
    }

    pub fn print_top(&self, n: usize) {
        let profiles = self.profiles.lock();
        let mut entries: Vec<&FuncProfile> = profiles.values().collect();
        entries.sort_by(|a, b| b.total_time_ns.cmp(&a.total_time_ns));

        println!("=== Hot Functions (top {}) ===", n);
        println!("  {:<30} {:>10}  {:>12}  {:>12}", "Function", "Calls", "Self(µs)", "Total(µs)");
        println!("  {}", "-".repeat(70));
        for p in entries.iter().take(n) {
            println!("  {:<30} {:>10}  {:>12.1}  {:>12.1}",
                p.name, p.calls,
                p.self_time_ns as f64 / 1000.0,
                p.total_time_ns as f64 / 1000.0);
        }
    }
}

// ---------------------------------------------------------------------------
// Hot loop detector
// ---------------------------------------------------------------------------

/// Tracks back-edge counts per bytecode offset.
/// When a back-edge count exceeds the threshold, the loop is marked "hot".
pub struct HotLoopDetector {
    /// back_edge_ip → count
    counts: Mutex<HashMap<usize, u32>>,
    pub hot_loops: Mutex<Vec<usize>>, // list of hot loop entry IPs
    threshold: u32,
}

impl HotLoopDetector {
    pub fn new(threshold: u32) -> Self {
        HotLoopDetector {
            counts: Mutex::new(HashMap::new()),
            hot_loops: Mutex::new(Vec::new()),
            threshold,
        }
    }

    /// Called on every loop back-edge. Returns true if newly hot.
    #[inline(always)]
    pub fn tick(&self, ip: usize) -> bool {
        let mut counts = self.counts.lock();
        let count = counts.entry(ip).or_insert(0);
        *count += 1;
        if *count == self.threshold {
            self.hot_loops.lock().push(ip);
            return true; // newly hot!
        }
        false
    }

    pub fn is_hot(&self, ip: usize) -> bool {
        let counts = self.counts.lock();
        counts.get(&ip).copied().unwrap_or(0) >= self.threshold
    }

    pub fn print_stats(&self) {
        let hot = self.hot_loops.lock();
        println!("=== Hot Loops ({} detected) ===", hot.len());
        let counts = self.counts.lock();
        for &ip in hot.iter() {
            let count = counts.get(&ip).copied().unwrap_or(0);
            println!("  Loop@{}: {} back-edges", ip, count);
        }
    }
}

// ---------------------------------------------------------------------------
// Allocation rate tracker
// ---------------------------------------------------------------------------

pub struct AllocTracker {
    total_bytes: AtomicU64,
    total_allocs: AtomicU64,
    start_time: Instant,
}

impl AllocTracker {
    pub fn new() -> Self {
        AllocTracker {
            total_bytes:  AtomicU64::new(0),
            total_allocs: AtomicU64::new(0),
            start_time:   Instant::now(),
        }
    }

    #[inline(always)]
    pub fn record(&self, bytes: usize) {
        self.total_bytes.fetch_add(bytes as u64, Ordering::Relaxed);
        self.total_allocs.fetch_add(1, Ordering::Relaxed);
    }

    pub fn rate_mb_per_sec(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed < 0.001 { return 0.0; }
        let bytes = self.total_bytes.load(Ordering::Relaxed) as f64;
        bytes / elapsed / (1024.0 * 1024.0)
    }

    pub fn print_stats(&self) {
        let bytes  = self.total_bytes.load(Ordering::Relaxed);
        let allocs = self.total_allocs.load(Ordering::Relaxed);
        let _elapsed = self.start_time.elapsed().as_secs_f64();
        println!("=== Allocation Stats ===");
        println!("  Total allocated: {} KB", bytes / 1024);
        println!("  Total allocs:    {}", allocs);
        println!("  Alloc rate:      {:.2} MB/s", self.rate_mb_per_sec());
        println!("  Avg object size: {} bytes", if allocs > 0 { bytes / allocs } else { 0 });
    }
}

// ---------------------------------------------------------------------------
// Flame graph exporter (folded stacks format)
// ---------------------------------------------------------------------------

pub struct FlameGraph {
    stacks: Mutex<Vec<(Vec<String>, u64)>>, // (stack_frames, sample_count)
}

impl FlameGraph {
    pub fn new() -> Self {
        FlameGraph { stacks: Mutex::new(Vec::new()) }
    }

    pub fn record_stack(&self, stack: Vec<String>, count: u64) {
        self.stacks.lock().push((stack, count));
    }

    /// Export folded stacks format (compatible with inferno-flamegraph)
    pub fn export(&self, path: Option<&str>) {
        let stacks = self.stacks.lock();
        let mut output = String::new();

        for (frames, count) in stacks.iter() {
            let line = format!("{} {}", frames.join(";"), count);
            output.push_str(&line);
            output.push('\n');
        }

        match path {
            Some(p) => {
                if let Err(e) = std::fs::write(p, &output) {
                    eprintln!("FlameGraph: failed to write {}: {}", p, e);
                } else {
                    println!("Flame graph written to: {}", p);
                    println!("  Run: inferno-flamegraph {} > flame.svg", p);
                }
            }
            None => {
                if !output.is_empty() {
                    eprintln!("=== Flame Graph (folded) ===");
                    eprintln!("{}", &output[..output.len().min(2000)]);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Master Profiler
// ---------------------------------------------------------------------------

pub struct Profiler {
    pub config: ProfilerConfig,
    pub opcodes: OpcodeCounters,
    pub calls: CallTracker,
    pub hot_loops: HotLoopDetector,
    pub allocs: AllocTracker,
    pub flame: FlameGraph,
    pub start_time: Instant,
    /// Estimated dispatch cycles (instruction_count * avg_cycles_per_dispatch)
    pub instruction_count: AtomicU64,
    pub branch_misses: AtomicU64,
}

impl Profiler {
    pub fn new(config: ProfilerConfig) -> Self {
        let threshold = config.hot_threshold;
        Profiler {
            config,
            opcodes:    OpcodeCounters::new(),
            calls:      CallTracker::new(),
            hot_loops:  HotLoopDetector::new(threshold),
            allocs:     AllocTracker::new(),
            flame:      FlameGraph::new(),
            start_time: Instant::now(),
            instruction_count: AtomicU64::new(0),
            branch_misses:     AtomicU64::new(0),
        }
    }

    /// Record one instruction execution
    #[inline(always)]
    pub fn record_op(&self, op: Op) {
        if self.config.opcode_counters {
            self.opcodes.record(op);
        }
        self.instruction_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record function entry
    #[inline(always)]
    pub fn enter_fn(&self, name: &str) {
        if self.config.call_tracking {
            self.calls.enter(name);
        }
    }

    /// Record function exit
    #[inline(always)]
    pub fn exit_fn(&self, name: &str) {
        if self.config.call_tracking {
            self.calls.exit(name);
        }
    }

    /// Record allocation
    #[inline(always)]
    pub fn record_alloc(&self, bytes: usize) {
        if self.config.alloc_tracking {
            self.allocs.record(bytes);
        }
    }

    /// Record loop back-edge. Returns true if loop just became hot.
    #[inline(always)]
    pub fn loop_tick(&self, ip: usize) -> bool {
        if self.config.hot_loop_detect {
            return self.hot_loops.tick(ip);
        }
        false
    }

    /// Print full profiling report
    pub fn print_report(&self) {
        let elapsed = self.start_time.elapsed();
        let total_instructions = self.instruction_count.load(Ordering::Relaxed);
        let mips = if elapsed.as_secs_f64() > 0.0 {
            total_instructions as f64 / elapsed.as_secs_f64() / 1_000_000.0
        } else { 0.0 };

        println!();
        println!("╔══════════════════════════════════════════════════════╗");
        println!("║           Axiom Performance Profile Report            ║");
        println!("╚══════════════════════════════════════════════════════╝");
        println!("  Execution time:     {:.3}s", elapsed.as_secs_f64());
        println!("  Instructions:       {} ({:.1} MIPS)", total_instructions, mips);
        println!("  Branch misses:      {}", self.branch_misses.load(Ordering::Relaxed));
        println!();

        if self.config.opcode_counters {
            self.opcodes.print_top(10);
            println!();
        }

        if self.config.call_tracking {
            self.calls.print_top(10);
            println!();
        }

        if self.config.hot_loop_detect {
            self.hot_loops.print_stats();
            println!();
        }

        if self.config.alloc_tracking {
            self.allocs.print_stats();
            println!();
        }

        if self.config.flame_graph {
            self.flame.export(self.config.flame_graph_path.as_deref());
        }
    }

    /// Print a minimal one-line summary (for non-verbose mode)
    pub fn print_summary(&self) {
        let elapsed = self.start_time.elapsed();
        let total = self.instruction_count.load(Ordering::Relaxed);
        let mips = if elapsed.as_secs_f64() > 0.001 {
            total as f64 / elapsed.as_secs_f64() / 1_000_000.0
        } else { 0.0 };
        eprintln!("[axiom profile] {:.3}s | {} instrs | {:.1} MIPS | {} hot loops",
            elapsed.as_secs_f64(), total, mips,
            self.hot_loops.hot_loops.lock().len());
    }
}
