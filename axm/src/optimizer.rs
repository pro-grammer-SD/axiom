/// Static Optimization Pipeline
///
/// Runs on the bytecode Proto after compilation, before execution:
///
///   1. Constant folding   — fold constant arithmetic at compile time
///   2. Constant propagation — track which registers hold constants
///   3. Peephole optimization — replace bytecode windows with cheaper forms
///   4. Jump threading     — eliminate redundant jump chains
///   5. Dead code removal  — strip unreachable instructions after jumps
///   6. Nop compaction     — remove Nops introduced by other passes
///   7. Superinstruction fusion — already done in bytecode.rs
///
/// All passes are O(N) or O(N²) in bytecode length — fast.

use crate::bytecode::{Instr, Op, Proto};

// ---------------------------------------------------------------------------
// Optimization config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct OptConfig {
    pub constant_folding:   bool,
    pub constant_prop:      bool,
    pub peephole:           bool,
    pub jump_threading:     bool,
    pub dead_code:          bool,
    pub nop_removal:        bool,
    pub superinstructions:  bool,
}

impl Default for OptConfig {
    fn default() -> Self {
        OptConfig {
            constant_folding:  true,
            constant_prop:     true,
            peephole:          true,
            jump_threading:    true,
            dead_code:         true,
            nop_removal:       true,
            superinstructions: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run all enabled optimization passes on a prototype (in-place).
pub fn optimize(proto: &mut Proto, cfg: &OptConfig) {
    if cfg.constant_folding  { fold_constants(proto); }
    if cfg.peephole          { peephole(proto); }
    if cfg.jump_threading    { thread_jumps(proto); }
    if cfg.dead_code         { remove_dead_code(proto); }
    if cfg.nop_removal       { compact_nops(proto); }
    if cfg.superinstructions { crate::bytecode::apply_superinstructions(proto); }

    // Recurse into nested protos
    for i in 0..proto.protos.len() {
        let mut inner = proto.protos[i].clone();
        optimize(&mut inner, cfg);
        proto.protos[i] = inner;
    }
}

// ---------------------------------------------------------------------------
// Pass 1: Constant folding
// ---------------------------------------------------------------------------

/// Fold constant arithmetic at the bytecode level.
/// If both operands of Add/Sub/Mul/Div are known integers (via LoadInt),
/// replace the three instructions with a single LoadInt result.
fn fold_constants(proto: &mut Proto) {
    // Track register → constant value (for known LoadInt regs)
    let len = proto.code.len();
    let mut int_vals: Vec<Option<i32>> = vec![None; 256];

    for i in 0..len {
        let instr = proto.code[i];
        match instr.op() {
            Op::LoadInt => {
                let a = instr.a() as usize;
                let v = instr.get_sbx();
                int_vals[a] = Some(v as i32);
            }
            Op::LoadNil | Op::LoadTrue | Op::LoadFalse => {
                int_vals[instr.a() as usize] = None;
            }
            Op::Add => {
                let a = instr.a() as usize;
                let b = instr.b() as usize;
                let c = instr.c() as usize;
                if let (Some(bv), Some(cv)) = (int_vals[b], int_vals[c]) {
                    let result = bv.wrapping_add(cv);
                    if result >= -32768 && result <= 32767 {
                        proto.code[i] = Instr::asbx(Op::LoadInt, a as u8, result as i16);
                        int_vals[a] = Some(result);
                        continue;
                    }
                }
                int_vals[a] = None;
            }
            Op::Sub => {
                let a = instr.a() as usize;
                let b = instr.b() as usize;
                let c = instr.c() as usize;
                if let (Some(bv), Some(cv)) = (int_vals[b], int_vals[c]) {
                    let result = bv.wrapping_sub(cv);
                    if result >= -32768 && result <= 32767 {
                        proto.code[i] = Instr::asbx(Op::LoadInt, a as u8, result as i16);
                        int_vals[a] = Some(result);
                        continue;
                    }
                }
                int_vals[a] = None;
            }
            Op::Mul => {
                let a = instr.a() as usize;
                let b = instr.b() as usize;
                let c = instr.c() as usize;
                if let (Some(bv), Some(cv)) = (int_vals[b], int_vals[c]) {
                    let result = bv.wrapping_mul(cv);
                    if result >= -32768 && result <= 32767 {
                        proto.code[i] = Instr::asbx(Op::LoadInt, a as u8, result as i16);
                        int_vals[a] = Some(result);
                        continue;
                    }
                }
                int_vals[a] = None;
            }
            Op::Neg => {
                let a = instr.a() as usize;
                let b = instr.b() as usize;
                if let Some(bv) = int_vals[b] {
                    let result = bv.wrapping_neg();
                    if result >= -32768 && result <= 32767 {
                        proto.code[i] = Instr::asbx(Op::LoadInt, a as u8, result as i16);
                        int_vals[a] = Some(result);
                        continue;
                    }
                }
                int_vals[a] = None;
            }
            Op::Move => {
                let a = instr.a() as usize;
                let b = instr.b() as usize;
                // Propagate constant through Move
                int_vals[a] = int_vals[b];
            }
            _ => {
                // Any other op invalidates the destination
                let a = instr.a() as usize;
                if a < 256 { int_vals[a] = None; }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Pass 2: Peephole optimizer
// ---------------------------------------------------------------------------

/// 2-instruction window peephole optimizations.
fn peephole(proto: &mut Proto) {
    let len = proto.code.len();
    if len < 2 { return; }

    for i in 0..len - 1 {
        let curr = proto.code[i];
        let next = proto.code[i + 1];

        match (curr.op(), next.op()) {
            // Move A, B followed by Move B, A → remove the second (redundant roundtrip)
            (Op::Move, Op::Move) => {
                if curr.a() == next.b() && curr.b() == next.a() {
                    proto.code[i + 1] = Instr::abc(Op::Nop, 0, 0, 0);
                }
            }

            // Not A, B followed by Not A, A → double negation → Move A, B
            (Op::Not, Op::Not) => {
                if curr.a() == next.b() && next.a() == next.b() {
                    proto.code[i]     = Instr::abc(Op::Move, curr.b(), curr.a(), 0);
                    proto.code[i + 1] = Instr::abc(Op::Nop, 0, 0, 0);
                }
            }

            // Jump sBx=0 → Nop (zero-offset jump goes nowhere)
            (Op::Jump, _) => {
                if curr.get_sbx() == 0 {
                    proto.code[i] = Instr::abc(Op::Nop, 0, 0, 0);
                }
            }

            // LoadInt A, 0 followed by Add dst, src, A → AddIntImm dst, src, 0 → Nop (add zero)
            (Op::LoadInt, Op::Add) => {
                if curr.get_sbx() == 0 && curr.a() == next.c() {
                    // src + 0 = src, so replace Add with Move
                    proto.code[i]     = Instr::abc(Op::Move, next.a(), next.b(), 0);
                    proto.code[i + 1] = Instr::abc(Op::Nop, 0, 0, 0);
                }
            }

            // LoadInt A, 1 followed by Mul dst, src, A → Move dst, src (mul by 1 = identity)
            (Op::LoadInt, Op::Mul) => {
                if curr.get_sbx() == 1 && curr.a() == next.c() {
                    proto.code[i]     = Instr::abc(Op::Move, next.a(), next.b(), 0);
                    proto.code[i + 1] = Instr::abc(Op::Nop, 0, 0, 0);
                }
                // Mul by 0
                if curr.get_sbx() == 0 && curr.a() == next.c() {
                    proto.code[i]     = Instr::asbx(Op::LoadInt, next.a(), 0);
                    proto.code[i + 1] = Instr::abc(Op::Nop, 0, 0, 0);
                }
            }

            // JumpTrue/JumpFalse with known constant condition
            (Op::LoadTrue, Op::JumpFalse) => {
                if curr.a() == next.a() {
                    // JumpFalse(true) → never taken → Nop both
                    proto.code[i]     = Instr::abc(Op::Nop, 0, 0, 0);
                    proto.code[i + 1] = Instr::abc(Op::Nop, 0, 0, 0);
                }
            }
            (Op::LoadFalse, Op::JumpTrue) => {
                if curr.a() == next.a() {
                    proto.code[i]     = Instr::abc(Op::Nop, 0, 0, 0);
                    proto.code[i + 1] = Instr::abc(Op::Nop, 0, 0, 0);
                }
            }
            (Op::LoadTrue, Op::JumpTrue) => {
                if curr.a() == next.a() {
                    // Always taken → replace both with unconditional Jump
                    let sbx = next.get_sbx();
                    proto.code[i]     = Instr::asbx(Op::Jump, 0, sbx);
                    proto.code[i + 1] = Instr::abc(Op::Nop, 0, 0, 0);
                }
            }

            // Nop followed by anything → already handled by compact_nops
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Pass 3: Jump threading
// ---------------------------------------------------------------------------

/// If a Jump target is another Jump, redirect to the final destination.
fn thread_jumps(proto: &mut Proto) {
    let len = proto.code.len();

    for i in 0..len {
        if !matches!(proto.code[i].op(), Op::Jump | Op::JumpTrue | Op::JumpFalse |
                     Op::JumpNil | Op::JumpNotNil) {
            continue;
        }

        let sbx = proto.code[i].get_sbx() as i32;
        let mut target = i as i32 + 1 + sbx;

        // Follow the chain
        let mut hops = 0;
        while hops < 8 && target >= 0 && (target as usize) < len {
            let t_instr = proto.code[target as usize];
            if t_instr.op() == Op::Jump {
                let next_sbx = t_instr.get_sbx() as i32;
                target = target + 1 + next_sbx;
                hops += 1;
            } else {
                break;
            }
        }

        // Patch the jump
        let new_sbx = target - i as i32 - 1;
        if new_sbx >= i16::MIN as i32 && new_sbx <= i16::MAX as i32 {
            proto.code[i].patch_sbx(new_sbx as i16);
        }
    }
}

// ---------------------------------------------------------------------------
// Pass 4: Dead code removal
// ---------------------------------------------------------------------------

/// Mark instructions unreachable after unconditional jumps/returns.
fn remove_dead_code(proto: &mut Proto) {
    let len = proto.code.len();
    let mut reachable = vec![false; len];
    let mut worklist = vec![0usize];

    // BFS reachability analysis
    while let Some(i) = worklist.pop() {
        if i >= len || reachable[i] { continue; }
        reachable[i] = true;

        let instr = proto.code[i];
        match instr.op() {
            Op::Jump => {
                let target = i as i32 + 1 + instr.get_sbx() as i32;
                if target >= 0 { worklist.push(target as usize); }
                // No fallthrough
            }
            Op::Return | Op::ReturnNil | Op::NilReturn | Op::Halt => {
                // No fallthrough, no branch
            }
            Op::JumpTrue | Op::JumpFalse | Op::JumpNil | Op::JumpNotNil |
            Op::CmpLtJmp => {
                // Fallthrough
                if i + 1 < len { worklist.push(i + 1); }
                // Branch target
                let target = i as i32 + 1 + instr.get_sbx() as i32;
                if target >= 0 { worklist.push(target as usize); }
            }
            Op::LoopBack => {
                // Back edge — marks loop
                let target = i as i32 + 1 + instr.get_sbx() as i32;
                if target >= 0 { worklist.push(target as usize); }
            }
            _ => {
                if i + 1 < len { worklist.push(i + 1); }
            }
        }
    }

    // Replace unreachable instructions with Nop
    for i in 0..len {
        if !reachable[i] {
            proto.code[i] = Instr::abc(Op::Nop, 0, 0, 0);
        }
    }
}

// ---------------------------------------------------------------------------
// Pass 5: Nop compaction
// ---------------------------------------------------------------------------

/// Remove all Nop instructions, rebuilding the code vector.
/// Also rebuilds line_info and patches jump offsets.
fn compact_nops(proto: &mut Proto) {
    let old_code = proto.code.clone();
    let old_lines = proto.line_info.clone();
    let len = old_code.len();

    // Build mapping: old_idx → new_idx
    let mut old_to_new = vec![0i32; len + 1];
    let mut new_idx = 0i32;
    for (i, instr) in old_code.iter().enumerate() {
        old_to_new[i] = new_idx;
        if instr.op() != Op::Nop {
            new_idx += 1;
        }
    }
    old_to_new[len] = new_idx; // sentinel

    // Rebuild code and lines
    let mut new_code = Vec::with_capacity(new_idx as usize);
    let mut new_lines = Vec::with_capacity(new_idx as usize);
    let mut new_counters = Vec::with_capacity(new_idx as usize);

    for (i, mut instr) in old_code.into_iter().enumerate() {
        if instr.op() == Op::Nop { continue; }

        // Patch jump offsets
        match instr.op() {
            Op::Jump | Op::JumpTrue | Op::JumpFalse | Op::JumpNil |
            Op::JumpNotNil | Op::LoopBack | Op::CmpLtJmp => {
                let old_target = i as i32 + 1 + instr.get_sbx() as i32;
                let clamped = old_target.max(0).min(len as i32);
                let new_target = old_to_new[clamped as usize];
                let new_src    = old_to_new[i];
                let new_sbx   = new_target - new_src - 1;
                if new_sbx >= i16::MIN as i32 && new_sbx <= i16::MAX as i32 {
                    instr.patch_sbx(new_sbx as i16);
                }
            }
            _ => {}
        }

        let line = old_lines.get(i).copied().unwrap_or(0);
        let cnt  = proto.counters.get(i).copied().unwrap_or(0);
        new_code.push(instr);
        new_lines.push(line);
        new_counters.push(cnt);
    }

    proto.code = new_code;
    proto.line_info = new_lines;
    proto.counters = new_counters;
}

// ---------------------------------------------------------------------------
// Optimization stats
// ---------------------------------------------------------------------------

pub struct OptStats {
    pub instructions_before: usize,
    pub instructions_after: usize,
    pub constants_folded: usize,
    pub nops_removed: usize,
    pub dead_instrs: usize,
    pub jumps_threaded: usize,
}

impl OptStats {
    pub fn compute(before: &Proto, after: &Proto) -> Self {
        let dead = before.code.iter().filter(|i| i.op() == Op::Nop).count();
        OptStats {
            instructions_before: before.code.len(),
            instructions_after:  after.code.len(),
            constants_folded:    0, // set by fold pass
            nops_removed:        dead,
            dead_instrs:         0,
            jumps_threaded:      0,
        }
    }

    pub fn print(&self) {
        println!("=== Optimization Stats ===");
        println!("  Instructions: {} → {} ({}% reduction)",
            self.instructions_before, self.instructions_after,
            if self.instructions_before > 0 {
                (1.0 - self.instructions_after as f64 / self.instructions_before as f64) * 100.0
            } else { 0.0 });
        println!("  Nops removed: {}", self.nops_removed);
    }
}
