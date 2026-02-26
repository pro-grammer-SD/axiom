/// Inline Cache Infrastructure
///
/// Inspired by V8 and CPython 3.11 adaptive interpreter.
///
/// IC States:
///   UNINITIALIZED  — first execution, no cache
///   MONOMORPHIC    — cached for exactly 1 shape
///   POLYMORPHIC    — cached for 2–4 shapes (PIC)
///   MEGAMORPHIC    — too many shapes, fall back to hash lookup
///
/// For property access: cache shape + slot offset
/// For method calls: cache (receiver shape, method ptr)
/// For binary ops: cache (lhs_type, rhs_type, specialized_op)

use std::sync::atomic::{AtomicU32, Ordering};
use crate::bytecode::Op;

// ---------------------------------------------------------------------------
// Shape (Hidden Class)
// ---------------------------------------------------------------------------

/// A shape describes the layout of an object's properties.
/// Objects with the same shape share a property→slot mapping.
/// Changing a property transitions the object to a new shape.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Shape {
    /// Unique ID (assigned sequentially)
    pub id: u32,
    /// Property name → slot index mapping
    pub props: Vec<(String, u16)>,
}

impl Shape {
    pub fn empty() -> Self {
        Shape { id: NEXT_SHAPE_ID.fetch_add(1, Ordering::Relaxed), props: Vec::new() }
    }

    pub fn with_prop(&self, name: &str) -> (Shape, u16) {
        let slot = self.props.len() as u16;
        let mut props = self.props.clone();
        props.push((name.to_string(), slot));
        (Shape { id: NEXT_SHAPE_ID.fetch_add(1, Ordering::Relaxed), props }, slot)
    }

    pub fn get_slot(&self, name: &str) -> Option<u16> {
        self.props.iter().find(|(n, _)| n == name).map(|(_, s)| *s)
    }

    pub fn num_slots(&self) -> usize {
        self.props.len()
    }
}

static NEXT_SHAPE_ID: AtomicU32 = AtomicU32::new(1);

// ---------------------------------------------------------------------------
// Property IC Entry
// ---------------------------------------------------------------------------

const IC_MAX_POLY: usize = 4; // PIC holds up to 4 entries before megamorphic

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcState {
    Uninitialized,
    Monomorphic,
    Polymorphic,
    Megamorphic,
}

/// One entry in a polymorphic inline cache.
#[derive(Debug, Clone, Copy)]
pub struct IcEntry {
    /// Shape ID for this entry
    pub shape_id: u32,
    /// Property slot (offset in object's value array)
    pub slot: u16,
    /// Is this a method (callable) vs field?
    pub is_method: bool,
}

/// An inline cache site (attached to each GetProp/SetProp bytecode).
/// Kept small (32 bytes) to fit in cache line.
#[derive(Debug)]
#[repr(C, align(32))]
pub struct PropIC {
    pub state: IcState,
    pub entries: [Option<IcEntry>; IC_MAX_POLY],
    pub hit_count: u32,
    pub miss_count: u32,
}

impl PropIC {
    pub fn new() -> Self {
        PropIC {
            state: IcState::Uninitialized,
            entries: [None; IC_MAX_POLY],
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// Look up the slot for a given shape ID. Returns slot if hit.
    #[inline(always)]
    pub fn lookup(&mut self, shape_id: u32) -> Option<u16> {
        // Monomorphic fast path — single comparison
        if self.state == IcState::Monomorphic {
            if let Some(entry) = self.entries[0] {
                if entry.shape_id == shape_id {
                    self.hit_count += 1;
                    return Some(entry.slot);
                }
            }
            self.miss_count += 1;
            return None;
        }

        // Polymorphic — scan 2–4 entries
        if self.state == IcState::Polymorphic {
            for entry_opt in &self.entries {
                if let Some(entry) = entry_opt {
                    if entry.shape_id == shape_id {
                        self.hit_count += 1;
                        return Some(entry.slot);
                    }
                }
            }
            self.miss_count += 1;
            return None;
        }

        // Megamorphic or uninitialized → miss
        self.miss_count += 1;
        None
    }

    /// Update the IC with a new (shape, slot) observation.
    pub fn update(&mut self, shape_id: u32, slot: u16, is_method: bool) {
        let new_entry = IcEntry { shape_id, slot, is_method };

        match self.state {
            IcState::Uninitialized => {
                self.entries[0] = Some(new_entry);
                self.state = IcState::Monomorphic;
            }
            IcState::Monomorphic => {
                if let Some(e) = self.entries[0] {
                    if e.shape_id == shape_id { return; } // already cached
                }
                self.entries[1] = Some(new_entry);
                self.state = IcState::Polymorphic;
            }
            IcState::Polymorphic => {
                for slot_opt in &mut self.entries {
                    if slot_opt.is_none() {
                        *slot_opt = Some(new_entry);
                        return;
                    }
                    if let Some(e) = slot_opt {
                        if e.shape_id == shape_id { return; }
                    }
                }
                // All 4 slots filled → go megamorphic
                self.state = IcState::Megamorphic;
            }
            IcState::Megamorphic => {} // no-op
        }
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 { 0.0 } else { self.hit_count as f64 / total as f64 }
    }
}

// ---------------------------------------------------------------------------
// Call IC Entry — caches (receiver_shape, method_proto_idx)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct CallICEntry {
    pub shape_id: u32,
    pub proto_idx: u32, // index into VM's proto table
    pub is_native: bool,
    pub native_idx: u32,
}

#[derive(Debug)]
#[repr(C, align(32))]
pub struct CallIC {
    pub state: IcState,
    pub entry: Option<CallICEntry>, // monomorphic only for calls
    pub hit_count: u32,
    pub miss_count: u32,
}

impl CallIC {
    pub fn new() -> Self {
        CallIC {
            state: IcState::Uninitialized,
            entry: None,
            hit_count: 0,
            miss_count: 0,
        }
    }

    #[inline(always)]
    pub fn lookup(&mut self, shape_id: u32) -> Option<CallICEntry> {
        if let Some(e) = self.entry {
            if e.shape_id == shape_id {
                self.hit_count += 1;
                return Some(e);
            }
        }
        self.miss_count += 1;
        None
    }

    pub fn update(&mut self, entry: CallICEntry) {
        if self.entry.map_or(false, |e| e.shape_id != entry.shape_id) {
            self.state = IcState::Megamorphic;
            return;
        }
        self.entry = Some(entry);
        self.state = IcState::Monomorphic;
    }
}

// ---------------------------------------------------------------------------
// BinaryOp IC — tracks type feedback for arithmetic quickening
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeFeedback {
    Unknown,
    Int,
    Float,
    String,
    Mixed,
}

impl TypeFeedback {
    pub fn observe(self, is_int: bool, is_float: bool) -> Self {
        let new = if is_int { TypeFeedback::Int }
            else if is_float { TypeFeedback::Float }
            else { TypeFeedback::Mixed };
        match (self, new) {
            (TypeFeedback::Unknown, any) => any,
            (a, b) if a == b => a,
            _ => TypeFeedback::Mixed,
        }
    }
}

/// Per-bytecode-site type feedback for binary operations.
/// Drives adaptive quickening.
#[derive(Debug, Clone)]
pub struct BinopIC {
    pub lhs_feedback: TypeFeedback,
    pub rhs_feedback: TypeFeedback,
    pub exec_count: u32,
    pub quickened_op: Option<Op>,
}

impl BinopIC {
    pub fn new() -> Self {
        BinopIC {
            lhs_feedback: TypeFeedback::Unknown,
            rhs_feedback: TypeFeedback::Unknown,
            exec_count: 0,
            quickened_op: None,
        }
    }

    /// Record one observation. Returns whether quickening should trigger.
    pub fn observe(&mut self, lhs_int: bool, lhs_float: bool, rhs_int: bool, rhs_float: bool, base_op: Op) -> bool {
        self.lhs_feedback = self.lhs_feedback.observe(lhs_int, lhs_float);
        self.rhs_feedback = self.rhs_feedback.observe(rhs_int, rhs_float);
        self.exec_count += 1;

        // Quicken after 16 executions with stable types
        if self.exec_count == 16 && self.quickened_op.is_none() {
            if self.lhs_feedback == TypeFeedback::Int && self.rhs_feedback == TypeFeedback::Int {
                self.quickened_op = base_op.quicken_int();
                return self.quickened_op.is_some();
            }
            if self.lhs_feedback == TypeFeedback::Float && self.rhs_feedback == TypeFeedback::Float {
                self.quickened_op = base_op.quicken_float();
                return self.quickened_op.is_some();
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// IC Table — per-prototype collection of all IC sites
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ICTable {
    pub prop_ics:  Vec<PropIC>,
    pub call_ics:  Vec<CallIC>,
    pub binop_ics: Vec<BinopIC>,
}

impl ICTable {
    pub fn new(prop_count: usize, call_count: usize, binop_count: usize) -> Self {
        ICTable {
            prop_ics:  (0..prop_count).map(|_| PropIC::new()).collect(),
            call_ics:  (0..call_count).map(|_| CallIC::new()).collect(),
            binop_ics: (0..binop_count).map(|_| BinopIC::new()).collect(),
        }
    }

    pub fn print_stats(&self) {
        println!("=== IC Statistics ===");
        let mut total_hits = 0u32;
        let mut total_misses = 0u32;
        for (i, ic) in self.prop_ics.iter().enumerate() {
            if ic.hit_count + ic.miss_count > 0 {
                println!("  PropIC[{}]: {:?}  hit_rate={:.1}%", i, ic.state, ic.hit_rate() * 100.0);
                total_hits += ic.hit_count;
                total_misses += ic.miss_count;
            }
        }
        let total = total_hits + total_misses;
        if total > 0 {
            println!("  Overall prop IC: {}/{} ({:.1}% hit)", total_hits, total, total_hits as f64 / total as f64 * 100.0);
        }
        for (i, ic) in self.binop_ics.iter().enumerate() {
            if ic.exec_count > 0 {
                println!("  BinopIC[{}]: lhs={:?} rhs={:?} quickened={:?}",
                    i, ic.lhs_feedback, ic.rhs_feedback, ic.quickened_op);
            }
        }
    }
}
