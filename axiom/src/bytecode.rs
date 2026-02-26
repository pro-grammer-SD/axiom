/// Bytecode Instruction Set — Register-Based, 32-bit Fixed-Width
///
/// INSTRUCTION FORMATS (Lua 5.x inspired):
///
///   iABC:  |  C: 8  |  B: 8  |  A: 8  | OP: 8  |  — 3 register operands
///   iABx:  |     Bx: 16      |  A: 8  | OP: 8  |  — 1 reg + 16-bit unsigned
///   iAsBx: |    sBx: 16      |  A: 8  | OP: 8  |  — 1 reg + 16-bit signed
///   iAx:   |       Ax: 24            | OP: 8  |  — 24-bit unsigned operand
///
/// Register 0..N are locals + temporaries in the current frame.
/// Register 255 is a scratch/result accumulator.
///
/// SUPERINSTRUCTIONS (compound opcodes for hot patterns):
///   AddIntImm   = LoadInt(r) + Add(dst, src, r)     → dst = src + imm
///   IncrLocal   = dst = dst + 1                     → common loop counter
///   LoadNilRet  = LoadNil + Return                  → empty function return
///   CmpJmpTrue  = Eq/Lt/Le + JumpIfFalse            → loop condition
///   CallNoRet   = Call where result ignored

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Op {
    // ── Loads ────────────────────────────────────────────────────────────────
    LoadNil    = 0,  // A         → R[A] = nil
    LoadTrue   = 1,  // A         → R[A] = true
    LoadFalse  = 2,  // A         → R[A] = false
    LoadInt    = 3,  // A, sBx   → R[A] = sBx (i16 literal)
    LoadFloat  = 4,  // A, Bx    → R[A] = K[Bx]  (float constant)
    LoadStr    = 5,  // A, Bx    → R[A] = S[Bx]  (string constant)
    LoadConst  = 6,  // A, Bx    → R[A] = K[Bx]  (any constant)
    LoadGlobal = 7,  // A, Bx    → R[A] = G[Bx]
    StoreGlobal= 8,  // A, Bx    → G[Bx] = R[A]
    Move       = 9,  // A, B     → R[A] = R[B]

    // ── Upvalues ─────────────────────────────────────────────────────────────
    LoadUpval  = 10, // A, B     → R[A] = UV[B]
    StoreUpval = 11, // A, B     → UV[B] = R[A]

    // ── Generic Arithmetic (type-dispatching) ─────────────────────────────────
    Add        = 12, // A, B, C  → R[A] = R[B] + R[C]
    Sub        = 13, // A, B, C  → R[A] = R[B] - R[C]
    Mul        = 14, // A, B, C  → R[A] = R[B] * R[C]
    Div        = 15, // A, B, C  → R[A] = R[B] / R[C]
    Mod        = 16, // A, B, C  → R[A] = R[B] % R[C]
    Pow        = 17, // A, B, C  → R[A] = R[B] ^ R[C]
    Neg        = 18, // A, B     → R[A] = -R[B]

    // ── Specialized Arithmetic (quickened — no dispatch branching) ─────────────
    AddInt     = 19, // A, B, C  → R[A] = R[B] + R[C]  (both i32)
    SubInt     = 20, // A, B, C
    MulInt     = 21, // A, B, C
    AddFloat   = 22, // A, B, C  → R[A] = R[B] + R[C]  (both f64)
    SubFloat   = 23, // A, B, C
    MulFloat   = 24, // A, B, C
    DivFloat   = 25, // A, B, C

    // ── String Concat ────────────────────────────────────────────────────────
    Concat     = 26, // A, B, C  → R[A] = R[B] .. R[C]

    // ── Comparison (result: NanVal bool) ──────────────────────────────────────
    Eq         = 27, // A, B, C  → R[A] = R[B] == R[C]
    Ne         = 28, // A, B, C  → R[A] = R[B] != R[C]
    Lt         = 29, // A, B, C  → R[A] = R[B] <  R[C]
    Le         = 30, // A, B, C  → R[A] = R[B] <= R[C]
    Gt         = 31, // A, B, C  → R[A] = R[B] >  R[C]
    Ge         = 32, // A, B, C  → R[A] = R[B] >= R[C]

    // ── Specialized Comparison (quickened) ───────────────────────────────────
    LtInt      = 33, // A, B, C
    LeInt      = 34,
    EqInt      = 35,

    // ── Logic ────────────────────────────────────────────────────────────────
    Not        = 36, // A, B     → R[A] = !R[B]
    And        = 37, // A, B, C
    Or         = 38, // A, B, C

    // ── Control Flow ─────────────────────────────────────────────────────────
    Jump       = 39, // sBx     → ip += sBx
    JumpTrue   = 40, // A, sBx  → if R[A] is truthy: ip += sBx
    JumpFalse  = 41, // A, sBx  → if !R[A] is truthy: ip += sBx
    JumpNil    = 42, // A, sBx  → if R[A] == nil: ip += sBx
    JumpNotNil = 43, // A, sBx  → if R[A] != nil: ip += sBx

    // ── Function Calls ───────────────────────────────────────────────────────
    /// R[A] = call R[A+1](R[A+2]..R[A+1+C])
    Call       = 44, // A, B, C  — A=ret, A+1=func, C=argc
    /// Tail call — does not push frame
    CallTail   = 45, // A, B, C
    Return     = 46, // A        → return R[A]
    ReturnNil  = 47, //           → return nil
    /// Call a known native fn by index
    CallNative = 48, // A, B, C  — A=ret, B=native_idx, C=argc; args in R[A+1..A+C]

    // ── Property Access with Inline Cache ─────────────────────────────────────
    GetProp    = 49, // A, B, Bx → R[A] = R[B].S[Bx]  (IC site)
    SetProp    = 50, // A, B, Bx → R[A].S[Bx] = R[B]  (IC site)
    GetIndex   = 51, // A, B, C  → R[A] = R[B][R[C]]
    SetIndex   = 52, // A, B, C  → R[A][R[B]] = R[C]

    // ── Collections ──────────────────────────────────────────────────────────
    NewList    = 53, // A, Bx   → R[A] = List(R[A+1]..R[A+Bx])
    NewMap     = 54, // A       → R[A] = {}
    ListPush   = 55, // A, B    → R[A].push(R[B])
    ListLen    = 56, // A, B    → R[A] = len(R[B])

    // ── Object / Class ────────────────────────────────────────────────────────
    NewObj     = 57, // A, Bx   → R[A] = new class[Bx]()
    GetSelf    = 58, // A       → R[A] = self (frame.self_val)
    SetSelf    = 59, // A       → frame.self_val = R[A]
    GetMethod  = 60, // A, B, Bx → R[A] = R[B].method[Bx] (bound method lookup + IC)

    // ── Closures ─────────────────────────────────────────────────────────────
    Closure    = 61, // A, Bx   → R[A] = closure(proto[Bx])
    CloseUpval = 62, // A       → close upvalue R[A]

    // ── Superinstructions (hot-pattern compound opcodes) ─────────────────────
    /// R[A] = R[B] + imm16  (add integer immediate — eliminates LoadInt for counters)
    AddIntImm  = 63, // A, B, sBx  → R[A] = R[B] + sBx
    /// R[A] = R[A] + 1  (loop counter increment — ultra-compact)
    IncrLocal  = 64, // A          → R[A] = R[A] + 1
    /// R[A] = R[A] - 1
    DecrLocal  = 65, // A
    /// Load nil + Return  (zero-overhead empty function)
    NilReturn  = 66, //
    /// if !(R[A] < R[B]): jump  (loop exit test — cmp + branch fused)
    CmpLtJmp   = 67, // A, B, sBx  → if R[A] >= R[B]: ip += sBx
    /// Call + store in same register (avoids Move after call)
    CallStore  = 68, // A, B, C
    /// Concatenate + store result (string building)
    ConcatStore= 69, // A, B, C

    // ── Profiling Hooks ───────────────────────────────────────────────────────
    /// Increment opcode counter (elided in opt builds)
    Profile    = 70, // Ax  → profiler.record(Ax)
    /// Loop back-edge — triggers hot-loop detection
    LoopBack   = 71, // sBx → ip += sBx; profiler.loop_tick()

    // ── Misc ─────────────────────────────────────────────────────────────────
    Nop        = 72,
    Halt       = 73,
    // Quickening markers (used during adaptive specialization)
    Unquicken  = 74, // Restore generic opcode (deopt)
}

impl Op {
    pub fn name(self) -> &'static str {
        match self {
            Op::LoadNil => "LoadNil",       Op::LoadTrue => "LoadTrue",
            Op::LoadFalse => "LoadFalse",   Op::LoadInt => "LoadInt",
            Op::LoadFloat => "LoadFloat",   Op::LoadStr => "LoadStr",
            Op::LoadConst => "LoadConst",   Op::LoadGlobal => "LoadGlobal",
            Op::StoreGlobal => "StoreGlobal", Op::Move => "Move",
            Op::LoadUpval => "LoadUpval",   Op::StoreUpval => "StoreUpval",
            Op::Add => "Add",               Op::Sub => "Sub",
            Op::Mul => "Mul",               Op::Div => "Div",
            Op::Mod => "Mod",               Op::Pow => "Pow",
            Op::Neg => "Neg",               Op::AddInt => "AddInt",
            Op::SubInt => "SubInt",         Op::MulInt => "MulInt",
            Op::AddFloat => "AddFloat",     Op::SubFloat => "SubFloat",
            Op::MulFloat => "MulFloat",     Op::DivFloat => "DivFloat",
            Op::Concat => "Concat",         Op::Eq => "Eq",
            Op::Ne => "Ne",                 Op::Lt => "Lt",
            Op::Le => "Le",                 Op::Gt => "Gt",
            Op::Ge => "Ge",                 Op::LtInt => "LtInt",
            Op::LeInt => "LeInt",           Op::EqInt => "EqInt",
            Op::Not => "Not",               Op::And => "And",
            Op::Or => "Or",                 Op::Jump => "Jump",
            Op::JumpTrue => "JumpTrue",     Op::JumpFalse => "JumpFalse",
            Op::JumpNil => "JumpNil",       Op::JumpNotNil => "JumpNotNil",
            Op::Call => "Call",             Op::CallTail => "CallTail",
            Op::Return => "Return",         Op::ReturnNil => "ReturnNil",
            Op::CallNative => "CallNative", Op::GetProp => "GetProp",
            Op::SetProp => "SetProp",       Op::GetIndex => "GetIndex",
            Op::SetIndex => "SetIndex",     Op::NewList => "NewList",
            Op::NewMap => "NewMap",         Op::ListPush => "ListPush",
            Op::ListLen => "ListLen",       Op::NewObj => "NewObj",
            Op::GetSelf => "GetSelf",       Op::SetSelf => "SetSelf",
            Op::GetMethod => "GetMethod",   Op::Closure => "Closure",
            Op::CloseUpval => "CloseUpval", Op::AddIntImm => "AddIntImm",
            Op::IncrLocal => "IncrLocal",   Op::DecrLocal => "DecrLocal",
            Op::NilReturn => "NilReturn",   Op::CmpLtJmp => "CmpLtJmp",
            Op::CallStore => "CallStore",   Op::ConcatStore => "ConcatStore",
            Op::Profile => "Profile",       Op::LoopBack => "LoopBack",
            Op::Nop => "Nop",               Op::Halt => "Halt",
            Op::Unquicken => "Unquicken",
        }
    }

    /// Is this a generic opcode that can be quickened (replaced with specialized)?
    pub fn can_quicken(self) -> bool {
        matches!(self, Op::Add | Op::Sub | Op::Mul | Op::Div |
                       Op::Lt | Op::Le | Op::Eq | Op::Ne |
                       Op::GetProp | Op::Call)
    }

    /// Return the specialized (quickened) variant for integer operands
    pub fn quicken_int(self) -> Option<Op> {
        match self {
            Op::Add => Some(Op::AddInt),
            Op::Sub => Some(Op::SubInt),
            Op::Mul => Some(Op::MulInt),
            Op::Lt  => Some(Op::LtInt),
            Op::Le  => Some(Op::LeInt),
            Op::Eq  => Some(Op::EqInt),
            _       => None,
        }
    }

    /// Return the specialized (quickened) variant for float operands
    pub fn quicken_float(self) -> Option<Op> {
        match self {
            Op::Add => Some(Op::AddFloat),
            Op::Sub => Some(Op::SubFloat),
            Op::Mul => Some(Op::MulFloat),
            Op::Div => Some(Op::DivFloat),
            _       => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Instruction encoding — 32 bits, fixed width
// ---------------------------------------------------------------------------

/// A 32-bit instruction.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Instr(pub u32);

impl Instr {
    // iABC: op=8, A=8, B=8, C=8
    #[inline] pub fn abc(op: Op, a: u8, b: u8, c: u8) -> Self {
        Instr((op as u32) | ((a as u32) << 8) | ((b as u32) << 16) | ((c as u32) << 24))
    }

    // iABx: op=8, A=8, Bx=16 (unsigned)
    #[inline] pub fn abx(op: Op, a: u8, bx: u16) -> Self {
        Instr((op as u32) | ((a as u32) << 8) | ((bx as u32) << 16))
    }

    // iAsBx: op=8, A=8, sBx=16 (signed, stored with +32768 bias)
    #[inline] pub fn asbx(op: Op, a: u8, sbx: i16) -> Self {
        let biased = (sbx as i32 + 32768) as u16;
        Instr((op as u32) | ((a as u32) << 8) | ((biased as u32) << 16))
    }

    // iAx: op=8, Ax=24
    #[inline] pub fn ax(op: Op, ax: u32) -> Self {
        Instr((op as u32) | (ax << 8))
    }

    // Decoders
    #[inline] pub fn op(self) -> Op {
        unsafe { std::mem::transmute(self.0 as u8) }
    }
    #[inline] pub fn a(self) -> u8 { ((self.0 >> 8) & 0xFF) as u8 }
    #[inline] pub fn b(self) -> u8 { ((self.0 >> 16) & 0xFF) as u8 }
    #[inline] pub fn c(self) -> u8 { ((self.0 >> 24) & 0xFF) as u8 }
    #[inline] pub fn bx(self) -> u16 { ((self.0 >> 16) & 0xFFFF) as u16 }

    // Fixed sbx: Using wrapping arithmetic with i16::MIN
    #[inline] pub fn sbx(self) -> i16 {
        ((self.0 >> 16) as i16).wrapping_sub(i16::MIN)
    }

    #[inline] pub fn get_ax(self) -> u32 { self.0 >> 8 }

    // Correct sbx decoder (bias of 32768)
    #[inline] pub fn get_sbx(self) -> i16 {
        let raw = ((self.0 >> 16) & 0xFFFF) as u16;
        // Perform arithmetic in i32 to avoid literal range errors
        (raw as i32 - 32768) as i16
    }

    // Patch the jump offset in-place (for back-patching)
    #[inline] pub fn patch_sbx(&mut self, sbx: i16) {
        // Bias the signed value back into an unsigned range
        let biased = (sbx as i32 + 32768) as u16;
        self.0 = (self.0 & 0x0000_FFFF) | ((biased as u32) << 16);
    }
}

impl std::fmt::Debug for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = self.op();
        write!(f, "{:<12} A={:3} B={:3} C={:3} | sBx={:6} Bx={:5}",
            op.name(), self.a(), self.b(), self.c(), self.get_sbx(), self.bx())
    }
}

// ---------------------------------------------------------------------------
// Prototype — compiled function/chunk metadata
// ---------------------------------------------------------------------------

/// A compiled function (prototype).
/// All bytecode lives in contiguous Vec<Instr> — cache-friendly.
#[derive(Debug, Clone)]
pub struct Proto {
    /// Bytecode — the hot path
    pub code: Vec<Instr>,
    /// Floating-point constant pool (indexed by LoadFloat/LoadConst Bx)
    pub float_consts: Vec<f64>,
    /// String constant pool (indexed by LoadStr Bx)
    pub str_consts: Vec<String>,
    /// Nested function prototypes (indexed by Closure Bx)
    pub protos: Vec<Proto>,
    /// Number of register slots (locals + temporaries)
    pub reg_count: u8,
    /// Number of parameters
    pub param_count: u8,
    /// Number of upvalues
    pub upval_count: u8,
    /// Has variable arguments?
    pub is_vararg: bool,
    /// Source name (for error messages)
    pub source: String,
    /// Line info — maps instruction index → source line
    pub line_info: Vec<u32>,
    /// Upvalue descriptors
    pub upvals: Vec<UpvalDesc>,
    /// Opcode execution counters (for adaptive specialization)
    pub counters: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct UpvalDesc {
    pub name: String,
    pub in_stack: bool,  // true = captured from local, false = from outer upvalue
    pub idx: u8,
}

impl Proto {
    pub fn new(source: impl Into<String>) -> Self {
        Proto {
            code: Vec::new(),
            float_consts: Vec::new(),
            str_consts: Vec::new(),
            protos: Vec::new(),
            reg_count: 0,
            param_count: 0,
            upval_count: 0,
            is_vararg: false,
            source: source.into(),
            line_info: Vec::new(),
            upvals: Vec::new(),
            counters: Vec::new(),
        }
    }

    /// Emit an instruction, return its index
    pub fn emit(&mut self, instr: Instr, line: u32) -> usize {
        let idx = self.code.len();
        self.code.push(instr);
        self.line_info.push(line);
        self.counters.push(0);
        idx
    }

    /// Emit a placeholder jump (returns index to back-patch)
    pub fn emit_jump(&mut self, op: Op, a: u8, line: u32) -> usize {
        self.emit(Instr::asbx(op, a, 0), line)
    }

    /// Patch a previously emitted jump with its target offset
    pub fn patch_jump(&mut self, jump_idx: usize) {
        let target = self.code.len() as i32;
        let offset = target - jump_idx as i32 - 1;
        self.code[jump_idx].patch_sbx(offset as i16);
    }

    /// Add float constant, return index
    pub fn add_float(&mut self, f: f64) -> u16 {
        for (i, &v) in self.float_consts.iter().enumerate() {
            if v == f { return i as u16; }
        }
        self.float_consts.push(f);
        (self.float_consts.len() - 1) as u16
    }

    /// Add string constant, return index
    pub fn add_string(&mut self, s: impl Into<String>) -> u16 {
        let s = s.into();
        for (i, v) in self.str_consts.iter().enumerate() {
            if *v == s { return i as u16; }
        }
        self.str_consts.push(s);
        (self.str_consts.len() - 1) as u16
    }

    /// Pretty-print disassembly
    pub fn disassemble(&self, name: &str) {
        println!("=== {} ({} regs, {} params) ===", name, self.reg_count, self.param_count);
        for (i, instr) in self.code.iter().enumerate() {
            let line = self.line_info.get(i).copied().unwrap_or(0);
            let count = self.counters.get(i).copied().unwrap_or(0);
            println!("  {:4}  [{:4}]  {:?}  (exec:{})", i, line, instr, count);
        }
        println!("  float_consts: {:?}", self.float_consts);
        println!("  str_consts:   {:?}", self.str_consts);
        for (i, p) in self.protos.iter().enumerate() {
            p.disassemble(&format!("{}.proto[{}]", name, i));
        }
    }
}

// ---------------------------------------------------------------------------
// Superinstruction patterns (mined from common sequences)
// ---------------------------------------------------------------------------

/// Known superinstruction fusion patterns.
/// If we see sequence [a, b], replace with super.
pub const SUPER_PATTERNS: &[(&[Op], Op)] = &[
    (&[Op::LoadNil, Op::Return],     Op::NilReturn),
    (&[Op::Add, Op::Jump],           Op::Add),       // not a super but placeholder
    (&[Op::Lt,  Op::JumpFalse],      Op::CmpLtJmp),
    (&[Op::Le,  Op::JumpFalse],      Op::CmpLtJmp),
    (&[Op::IncrLocal, Op::CmpLtJmp], Op::IncrLocal), // keep — just record pattern
];

/// Mine and apply superinstructions to a proto in-place.
pub fn apply_superinstructions(proto: &mut Proto) {
    let len = proto.code.len();
    if len < 2 { return; }

    let mut i = 0;
    while i + 1 < len {
        let op1 = proto.code[i].op();
        let op2 = proto.code[i + 1].op();

        // Pattern: LoadInt A, sBx followed by Add C, D, A
        // → replace with AddIntImm C, D, sBx
        if op1 == Op::LoadInt && op2 == Op::Add {
            let reg_a  = proto.code[i].a();
            let imm    = proto.code[i].get_sbx();
            let dst    = proto.code[i+1].a();
            let src_b  = proto.code[i+1].b();
            let src_c  = proto.code[i+1].c();
            // Only fuse if the loaded int is used as one of the Add operands
            if src_c == reg_a {
                proto.code[i] = Instr::asbx(Op::AddIntImm, dst, imm);
                proto.code[i+1] = Instr::abc(Op::Nop, src_b, 0, 0);
                // patch: Move src_b into dst+1 implicit
            }
        }

        // Pattern: Lt A, B, C followed by JumpFalse A, sBx
        // → replace with CmpLtJmp B, C, sBx; nop
        if op1 == Op::Lt && op2 == Op::JumpFalse {
            let cmp_a = proto.code[i].a();
            let b     = proto.code[i].b();
            let c     = proto.code[i].c();
            let jmp_a = proto.code[i+1].a();
            let sbx   = proto.code[i+1].get_sbx();
            if cmp_a == jmp_a {
                proto.code[i]   = Instr::asbx(Op::CmpLtJmp, b, sbx);
                proto.code[i].0 |= (c as u32) << 24;
                proto.code[i+1] = Instr::abc(Op::Nop, 0, 0, 0);
            }
        }

        // Pattern: LoadNil + Return → NilReturn
        if op1 == Op::LoadNil && op2 == Op::Return {
            let a = proto.code[i].a();
            let ret_a = proto.code[i+1].a();
            if a == ret_a {
                proto.code[i]   = Instr::abc(Op::NilReturn, 0, 0, 0);
                proto.code[i+1] = Instr::abc(Op::Nop, 0, 0, 0);
            }
        }

        i += 1;
    }
}
