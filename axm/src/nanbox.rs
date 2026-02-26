/// NaN-Boxing — Zero-Cost Tagged Value Representation
///
/// DESIGN: Every value fits in 64 bits. No heap allocation for primitives.
///
/// IEEE 754 double-precision NaN has:
///   - Exponent bits [62:52] all 1  (0x7FF)
///   - Mantissa bits [51:0]  non-zero  → "quiet NaN" when bit 51 = 1
///
/// We use "quiet NaN" space (bit 63=0, bits [62:52]=0x7FF, bit 51=1) as a tag:
///
///   63        51        47                              0
///   0 11111111111 1 [TAG:3] [PAYLOAD:48]
///
/// Tag space (bits 50..48 of mantissa):
///   000 = Not a special NaN → regular f64 double  (just stored as-is)
///   001 = NIL
///   010 = BOOL  (payload: 0=false, 1=true)
///   011 = INT   (payload: i32 in low 32 bits)
///   100 = HEAP  (payload: 48-bit heap pointer)
///   101 = SYM   (payload: string-intern table index u32)
///   110 = NATIVE_FN (payload: function table index u32)
///   111 = reserved
///
/// Regular doubles are stored as-is (not NaN → falls through to f64 path).
/// All NaN values we generate have the quiet-NaN bit set (bit 51=1).

use std::fmt;

// ---------------------------------------------------------------------------
// Tag constants
// ---------------------------------------------------------------------------
const NANBOX_NAN_MASK:   u64 = 0x7FFC_0000_0000_0000; // bit63=0, exp=0x7FF, bit51=1
const NANBOX_TAG_MASK:   u64 = 0x0003_0000_0000_0000; // bits [49:48]
const NANBOX_PAYLOAD:    u64 = 0x0000_FFFF_FFFF_FFFF; // bits [47:0]

const TAG_NIL:       u64 = 0x0001_0000_0000_0000;
const TAG_BOOL:      u64 = 0x0002_0000_0000_0000;
const TAG_INT:       u64 = 0x0003_0000_0000_0000;
const TAG_HEAP:      u64 = 0x0000_0000_0000_0000; // uses different NaN pattern
const TAG_SYM:       u64 = 0x0001_8000_0000_0000;
const TAG_NATIVE:    u64 = 0x0002_8000_0000_0000;

// Canonical NaN base for each type
const NIL_VALUE:     u64 = NANBOX_NAN_MASK | TAG_NIL;
const TRUE_VALUE:    u64 = NANBOX_NAN_MASK | TAG_BOOL | 1;
const FALSE_VALUE:   u64 = NANBOX_NAN_MASK | TAG_BOOL | 0;
// Heap uses: 0x7FF8_xxxx_xxxx_xxxx (different NaN canonical — bit 51=0, bit 50=0)
const HEAP_NAN_MASK: u64 = 0x7FF8_0000_0000_0000;

/// A NaN-boxed value. 8 bytes, no indirection for primitives.
/// Heap-allocated objects are pointed to via 48-bit pointers.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NanVal(u64);

impl NanVal {
    // ---------------------------------------------------------------------------
    // Constructors
    // ---------------------------------------------------------------------------

    #[inline(always)]
    pub const fn nil() -> Self {
        NanVal(NIL_VALUE)
    }

    #[inline(always)]
    pub const fn bool_val(b: bool) -> Self {
        if b { NanVal(TRUE_VALUE) } else { NanVal(FALSE_VALUE) }
    }

    #[inline(always)]
    pub fn from_f64(n: f64) -> Self {
        let bits = n.to_bits();
        // If it's a quiet NaN we might collide — rebox via a canonical form
        if (bits & NANBOX_NAN_MASK) == NANBOX_NAN_MASK && (bits != NANBOX_NAN_MASK) {
            // This is already a user quiet NaN — box it as float-NaN payload
            NanVal(NANBOX_NAN_MASK | 0x0000_8000_0000_0000 | (bits & 0x0000_7FFF_FFFF_FFFF))
        } else {
            NanVal(bits)
        }
    }

    #[inline(always)]
    pub fn from_i32(n: i32) -> Self {
        NanVal(NANBOX_NAN_MASK | TAG_INT | (n as u32 as u64))
    }

    #[inline(always)]
    pub fn from_heap_ptr(ptr: u64) -> Self {
        debug_assert!(ptr & !NANBOX_PAYLOAD == 0, "heap pointer > 48 bits");
        NanVal(HEAP_NAN_MASK | ptr)
    }

    #[inline(always)]
    pub fn from_sym(idx: u32) -> Self {
        NanVal(NANBOX_NAN_MASK | TAG_SYM | idx as u64)
    }

    #[inline(always)]
    pub fn from_native_fn(idx: u32) -> Self {
        NanVal(NANBOX_NAN_MASK | TAG_NATIVE | idx as u64)
    }

    // ---------------------------------------------------------------------------
    // Type checks
    // ---------------------------------------------------------------------------

    #[inline(always)]
    pub fn is_nil(self) -> bool {
        self.0 == NIL_VALUE
    }

    #[inline(always)]
    pub fn is_bool(self) -> bool {
        (self.0 & (NANBOX_NAN_MASK | NANBOX_TAG_MASK)) == (NANBOX_NAN_MASK | TAG_BOOL)
    }

    #[inline(always)]
    pub fn is_true(self) -> bool {
        self.0 == TRUE_VALUE
    }

    #[inline(always)]
    pub fn is_int(self) -> bool {
        (self.0 & (NANBOX_NAN_MASK | NANBOX_TAG_MASK)) == (NANBOX_NAN_MASK | TAG_INT)
    }

    #[inline(always)]
    pub fn is_float(self) -> bool {
        // Not a NaN payload → raw f64
        (self.0 & NANBOX_NAN_MASK) != NANBOX_NAN_MASK
    }

    #[inline(always)]
    pub fn is_number(self) -> bool {
        self.is_float() || self.is_int()
    }

    #[inline(always)]
    pub fn is_heap(self) -> bool {
        (self.0 & 0xFFFF_0000_0000_0000) == HEAP_NAN_MASK
    }

    #[inline(always)]
    pub fn is_sym(self) -> bool {
        (self.0 & (NANBOX_NAN_MASK | 0x0001_8000_0000_0000)) == (NANBOX_NAN_MASK | TAG_SYM)
    }

    #[inline(always)]
    pub fn is_native_fn(self) -> bool {
        (self.0 & (NANBOX_NAN_MASK | 0x0002_8000_0000_0000)) == (NANBOX_NAN_MASK | TAG_NATIVE)
    }

    #[inline(always)]
    pub fn is_truthy(self) -> bool {
        if self.is_nil() { return false; }
        if self.is_bool() { return self.is_true(); }
        if self.is_int() { return self.as_i32() != 0; }
        if self.is_float() { return self.as_f64() != 0.0; }
        true // heap objects are truthy
    }

    // ---------------------------------------------------------------------------
    // Extractors
    // ---------------------------------------------------------------------------

    #[inline(always)]
    pub fn as_f64(self) -> f64 {
        f64::from_bits(self.0)
    }

    #[inline(always)]
    pub fn as_number_f64(self) -> f64 {
        if self.is_int() {
            self.as_i32() as f64
        } else {
            self.as_f64()
        }
    }

    #[inline(always)]
    pub fn as_i32(self) -> i32 {
        (self.0 & 0xFFFF_FFFF) as i32
    }

    #[inline(always)]
    pub fn as_bool(self) -> bool {
        (self.0 & 1) != 0
    }

    #[inline(always)]
    pub fn as_heap_ptr(self) -> u64 {
        self.0 & NANBOX_PAYLOAD
    }

    #[inline(always)]
    pub fn as_sym(self) -> u32 {
        (self.0 & 0xFFFF_FFFF) as u32
    }

    #[inline(always)]
    pub fn as_native_fn(self) -> u32 {
        (self.0 & 0xFFFF_FFFF) as u32
    }

    #[inline(always)]
    pub fn raw(self) -> u64 {
        self.0
    }

    pub fn type_name(self) -> &'static str {
        if self.is_nil()    { "nil" }
        else if self.is_bool() { "bool" }
        else if self.is_int()  { "int" }
        else if self.is_float() { "float" }
        else if self.is_heap()  { "object" }
        else if self.is_sym()   { "string" }
        else if self.is_native_fn() { "native_fn" }
        else { "unknown" }
    }

    // ---------------------------------------------------------------------------
    // Arithmetic (specialized — no branching for same-type ops)
    // ---------------------------------------------------------------------------

    /// Fast path: both known int
    #[inline(always)]
    pub fn add_int(self, rhs: NanVal) -> NanVal {
        NanVal::from_i32(self.as_i32().wrapping_add(rhs.as_i32()))
    }

    /// Fast path: both known float
    #[inline(always)]
    pub fn add_float(self, rhs: NanVal) -> NanVal {
        NanVal::from_f64(self.as_f64() + rhs.as_f64())
    }

    #[inline(always)]
    pub fn sub_int(self, rhs: NanVal) -> NanVal {
        NanVal::from_i32(self.as_i32().wrapping_sub(rhs.as_i32()))
    }

    #[inline(always)]
    pub fn sub_float(self, rhs: NanVal) -> NanVal {
        NanVal::from_f64(self.as_f64() - rhs.as_f64())
    }

    #[inline(always)]
    pub fn mul_int(self, rhs: NanVal) -> NanVal {
        NanVal::from_i32(self.as_i32().wrapping_mul(rhs.as_i32()))
    }

    #[inline(always)]
    pub fn mul_float(self, rhs: NanVal) -> NanVal {
        NanVal::from_f64(self.as_f64() * rhs.as_f64())
    }

    #[inline(always)]
    pub fn eq_val(self, rhs: NanVal) -> NanVal {
        NanVal::bool_val(self.0 == rhs.0 || (
            self.is_number() && rhs.is_number() &&
            self.as_number_f64() == rhs.as_number_f64()
        ))
    }

    #[inline(always)]
    pub fn lt_num(self, rhs: NanVal) -> NanVal {
        NanVal::bool_val(self.as_number_f64() < rhs.as_number_f64())
    }

    #[inline(always)]
    pub fn le_num(self, rhs: NanVal) -> NanVal {
        NanVal::bool_val(self.as_number_f64() <= rhs.as_number_f64())
    }
}

impl PartialEq for NanVal {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl fmt::Debug for NanVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_nil() { write!(f, "nil") }
        else if self.is_bool() { write!(f, "{}", self.as_bool()) }
        else if self.is_int() { write!(f, "{}i", self.as_i32()) }
        else if self.is_float() { write!(f, "{}f", self.as_f64()) }
        else if self.is_sym() { write!(f, "sym#{}", self.as_sym()) }
        else if self.is_native_fn() { write!(f, "native#{}", self.as_native_fn()) }
        else if self.is_heap() { write!(f, "heap@0x{:x}", self.as_heap_ptr()) }
        else { write!(f, "raw:0x{:016x}", self.0) }
    }
}

impl fmt::Display for NanVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_nil() { write!(f, "nil") }
        else if self.is_bool() { write!(f, "{}", self.as_bool()) }
        else if self.is_int() { write!(f, "{}", self.as_i32()) }
        else if self.is_float() {
            let n = self.as_f64();
            if n.fract() == 0.0 && n.abs() < 1e15 { write!(f, "{}", n as i64) }
            else { write!(f, "{}", n) }
        }
        else if self.is_sym() { write!(f, "<sym:{}>", self.as_sym()) }
        else if self.is_native_fn() { write!(f, "<fn:{}>", self.as_native_fn()) }
        else if self.is_heap() { write!(f, "<obj:0x{:x}>", self.as_heap_ptr()) }
        else { write!(f, "<val:0x{:016x}>", self.0) }
    }
}

// ---------------------------------------------------------------------------
// String interner — maps &str → u32 index (cached, arena-stored)
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use parking_lot::RwLock;

pub struct StringInterner {
    table: RwLock<HashMap<String, u32>>,
    strings: RwLock<Vec<String>>,
}

impl StringInterner {
    pub fn new() -> Self {
        StringInterner {
            table: RwLock::new(HashMap::new()),
            strings: RwLock::new(Vec::new()),
        }
    }

    pub fn intern(&self, s: &str) -> u32 {
        {
            let table = self.table.read();
            if let Some(&idx) = table.get(s) {
                return idx;
            }
        }
        let mut table = self.table.write();
        if let Some(&idx) = table.get(s) {
            return idx;
        }
        let idx = {
            let mut strings = self.strings.write();
            let idx = strings.len() as u32;
            strings.push(s.to_string());
            idx
        };
        table.insert(s.to_string(), idx);
        idx
    }

    pub fn get(&self, idx: u32) -> Option<String> {
        self.strings.read().get(idx as usize).cloned()
    }

    pub fn len(&self) -> usize {
        self.strings.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nanbox_nil() {
        let v = NanVal::nil();
        assert!(v.is_nil());
        assert!(!v.is_bool());
    }

    #[test]
    fn test_nanbox_bool() {
        let t = NanVal::bool_val(true);
        let f = NanVal::bool_val(false);
        assert!(t.is_bool() && t.is_true());
        assert!(f.is_bool() && !f.is_true());
    }

    #[test]
    fn test_nanbox_int() {
        let v = NanVal::from_i32(42);
        assert!(v.is_int());
        assert_eq!(v.as_i32(), 42);
        let neg = NanVal::from_i32(-1);
        assert_eq!(neg.as_i32(), -1);
    }

    #[test]
    fn test_nanbox_float() {
        let v = NanVal::from_f64(3.14);
        assert!(v.is_float());
        assert!((v.as_f64() - 3.14).abs() < 1e-10);
    }

    #[test]
    fn test_nanbox_arithmetic() {
        let a = NanVal::from_i32(10);
        let b = NanVal::from_i32(3);
        assert_eq!(a.add_int(b).as_i32(), 13);
        assert_eq!(a.sub_int(b).as_i32(), 7);
        assert_eq!(a.mul_int(b).as_i32(), 30);
    }

    #[test]
    fn test_interner() {
        let s = StringInterner::new();
        let a = s.intern("hello");
        let b = s.intern("world");
        let c = s.intern("hello");
        assert_eq!(a, c);
        assert_ne!(a, b);
        assert_eq!(s.get(a).unwrap(), "hello");
    }
}
