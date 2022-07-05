// References:
//
// Details about what happens during each cycle while an op is executing:
// http://www.atarihq.com/danb/files/64doc.txt

use std::ops::Range;

use crate::MemoryMapped;

pub const CARRY_MASK: u8 = 1;
pub const ZERO_MASK: u8 = 2;
pub const IRQB_DISABLE_MASK: u8 = 4;
pub const DEC_MASK: u8 = 8;
pub const BRK_MASK: u8 = 16;
pub const ONE_MASK: u8 = 32; // Always 1
pub const OVERFLOW_MASK: u8 = 64;
pub const NEG_MASK: u8 = 128;

#[derive(Debug)]
pub enum AdrMode {
    Abs,       // Absolute: a
    AbsIdxInd, // Absolute Indexed Indirect: (a,x). W65C02S only.
    AbsIdxX,   // Absolute Indexed with X: a,x
    AbsIdxY,   // Absolute Indexed with Y: a,y
    AbsInd,    // Absolute Indirect: (a). 6502/W65C02S timing diff.
    Acc,       // Accumulator: A
    Imm,       // Immediate: #
    Imp,       // Implied: i
    PcRel,     // Program Counter Relative: r
    Stack,     // Stack: s
    Zp,        // Zero Page: zp
    ZpIdxInd,  // Zero Page Indexed Indirect: (zp,x)
    ZpIdxX,    // Zero Page Indexed with X: zp,x
    ZpIdxY,    // Zero Page Indexed with Y: zp,y
    ZpInd,     // Zero Page Indirect: (zp). W65C02S only.
    ZpIndIdxY, // Zero Page Indirected Indexed with Y: (zp),y
}

pub struct Op {
    pub name: OpName,      // 3 letter name of the op
    pub cycles: Range<u8>, // Rough range of cycles to complete op
    pub flags: u8,         // Bitmask of flags affected
    pub adr: AdrMode,      // Addressing mode
}

pub const fn op_len(a: &AdrMode) -> u16 {
    match a {
        AdrMode::Acc | AdrMode::Imp | AdrMode::Stack => 1,
        AdrMode::Imm | AdrMode::PcRel => 2,
        AdrMode::Zp | AdrMode::ZpIdxInd | AdrMode::ZpIdxX => 2,
        AdrMode::ZpIdxY | AdrMode::ZpInd | AdrMode::ZpIndIdxY => 2,
        AdrMode::Abs
        | AdrMode::AbsIdxInd
        | AdrMode::AbsIdxX
        | AdrMode::AbsIdxY
        | AdrMode::AbsInd => 3,
    }
}

const ILLEGAL: Op = Op {
    name: OpName::ILLEGAL,
    cycles: 0..0,
    flags: 0,
    adr: AdrMode::Imp,
};

const C: u8 = 1;
const Z: u8 = 2;
const I: u8 = 4;
const D: u8 = 8;
const _B: u8 = 16;
const V: u8 = 64;
const N: u8 = 128;
const NZ: u8 = N | Z;
const NZC: u8 = N | Z | C;
const NZCV: u8 = N | Z | C | V;
const NZCIDV: u8 = N | Z | C | I | D | V;

#[derive(Debug)]
pub enum OpName {
    ILLEGAL,
    ADC,
    AND,
    ASL,
    BBR0,
    BBR1,
    BBR2,
    BBR3,
    BBR4,
    BBR5,
    BBR6,
    BBR7,
    BBS0,
    BBS1,
    BBS2,
    BBS3,
    BBS4,
    BBS5,
    BBS6,
    BBS7,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRA,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PHX,
    PHY,
    PLA,
    PLP,
    PLX,
    PLY,
    RMB0,
    RMB1,
    RMB2,
    RMB3,
    RMB4,
    RMB5,
    RMB6,
    RMB7,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    SMB0,
    SMB1,
    SMB2,
    SMB3,
    SMB4,
    SMB5,
    SMB6,
    SMB7,
    STA,
    STP,
    STX,
    STY,
    STZ,
    TAX,
    TAY,
    TRB,
    TSB,
    TSX,
    TXA,
    TXS,
    TYA,
    WAI,
}

#[derive(Copy, Clone)]
pub struct CPU {
    // Current value of the address pins
    pub adr: u16,

    // Current value of the data pins
    pub data: u8,

    // True if the current cycle is a write cycle
    pub wr: bool,

    pub a: u8,   // Accumulator Register
    pub x: u8,   // Index Register X
    pub y: u8,   // Index Register Y
    pub p: u8,   // Processor Status (flags)
    pub sp: u8,  // Stack Pointer
    pub pc: u16, // Program Counter
    ir: u8,      // Instruction Register

    // Clock cycle while executing one operation
    pub op_cycle: u8,

    // Value of byte 2 and 3 of current operation
    pub op_a: u8,
    pub op_b: u8,

    // Temporary value used in current operation
    pub op_latch: u8,

    // Bookkeeping for operations that need to store 16-bit pointers
    pub op_ptr: u16,

    // Bookkeeping for branch instructions
    pub op_branch: bool,
    pub op_fixed_pc: u16,

    // For operations such as `BNE` that prefetch the next IR
    next_ir: Option<u8>,

    // Address to the first byte of current op
    pub op_offs: u16,

    // Clock cycle since reset
    pub cycles: usize,

    // Sync works the same as the sync pin:
    // It's true when the opcode is being fetched.
    pub sync: bool,
}

fn lo_hi_u16(lo: u8, hi: u8) -> u16 {
    u16::from(hi) << 8 | u16::from(lo)
}

macro_rules! op {
    ($name:ident, $cycles:expr, $flags:expr, $adr:ident) => {
        Op {
            name: OpName::$name,
            cycles: $cycles,
            flags: $flags,
            adr: AdrMode::$adr,
        }
    };
}

// Throws away the result of the expression.
macro_rules! throw_away {
    ($x:expr) => {
        drop($x)
    };
}

pub const OPS: [Op; 0x100] = [
    // 0x00
    op!(BRK, 7..7, I, Stack),
    op!(ORA, 6..6, NZ, ZpIdxInd),
    ILLEGAL,
    ILLEGAL,
    op!(TSB, 0..0, Z, Zp), // W65C02S only
    op!(ORA, 3..3, NZ, Zp),
    op!(ASL, 5..5, NZC, Zp),
    op!(RMB0, 0..0, 0, Zp), // W65C02S only
    op!(PHP, 3..3, 0, Stack),
    op!(ORA, 2..2, NZ, Imm),
    op!(ASL, 2..2, NZC, Acc),
    ILLEGAL,
    op!(TSB, 0..0, Z, Abs), // W65C02S only
    op!(ORA, 4..4, NZ, Abs),
    op!(ASL, 6..6, NZC, Abs),
    op!(BBR0, 0..0, 0, PcRel), // W65C02S only
    // 0x10
    op!(BPL, 2..3, 0, PcRel),      // Cycles: 3+/2
    op!(ORA, 5..5, NZ, ZpIndIdxY), // Cycles: 5+
    op!(ORA, 0..0, NZ, ZpInd),
    ILLEGAL,
    op!(TRB, 0..0, Z, Zp),
    op!(ORA, 4..4, NZ, ZpIdxX),
    op!(ASL, 6..6, NZC, ZpIdxX),
    op!(RMB1, 0..0, 0, Zp), // W65C02S only
    op!(CLC, 2..2, C, Imp),
    op!(ORA, 4..4, NZ, AbsIdxY), // Cycles: 4+
    op!(INC, 0..0, NZ, Acc),     // Old op with new addr mode
    ILLEGAL,
    op!(TRB, 0..0, Z, Abs),
    op!(ORA, 4..4, NZ, AbsIdxX), // Cycles: 4+
    op!(ASL, 7..7, NZC, AbsIdxX),
    op!(BBR1, 0..0, 0, PcRel), // W65C02S only
    // 0x20
    op!(JSR, 6..6, 0, Abs),
    op!(AND, 6..6, NZ, ZpIdxInd),
    ILLEGAL,
    ILLEGAL,
    op!(BIT, 3..3, NZC, Zp),
    op!(AND, 3..3, NZ, Zp),
    op!(ROL, 5..5, NZC, Zp),
    op!(RMB2, 0..0, 0, Zp), // W65C02S only
    op!(PLP, 4..4, NZCIDV, Stack),
    op!(AND, 2..2, NZ, Imm),
    op!(ROL, 2..2, NZC, Acc),
    ILLEGAL,
    op!(BIT, 4..4, NZC, Abs),
    op!(AND, 4..4, NZ, Abs),
    op!(ROL, 6..6, NZC, Abs),
    op!(BBR2, 0..0, 0, PcRel), // W65C02S only
    // 0x30
    op!(BMI, 2..3, 0, PcRel),      // Cycles: 3+/2
    op!(AND, 5..5, NZ, ZpIndIdxY), // Cycles: 5+
    op!(AND, 0..0, NZ, ZpInd),     // Old op with new addr mode
    ILLEGAL,
    op!(BIT, 0..0, NZC, ZpIdxX), // Old op with new addr mode. Verify flags for all BIT ops!
    op!(AND, 4..4, NZ, ZpIdxX),
    op!(ROL, 6..6, NZC, ZpIdxX),
    op!(RMB3, 0..0, 0, Zp), // W65C02S only
    op!(SEC, 2..2, C, Imp),
    op!(AND, 4..4, NZ, AbsIdxY), // Cycles: 4+
    op!(DEC, 0..0, NZ, Acc),
    ILLEGAL,
    op!(BIT, 0..0, NZC, AbsIdxX),
    op!(AND, 4..4, NZ, AbsIdxX), // Cycles: 4+
    op!(ROL, 7..7, NZC, AbsIdxX),
    op!(BBR3, 0..0, 0, PcRel), // W65C02S only
    // 0x40
    op!(RTI, 6..6, NZCIDV, Stack),
    op!(EOR, 6..6, NZ, ZpIdxInd),
    ILLEGAL,
    ILLEGAL,
    ILLEGAL,
    op!(EOR, 3..3, NZ, Zp),
    op!(LSR, 5..5, NZC, Zp),
    op!(RMB4, 0..0, 0, Zp), // W65C02S only
    op!(PHA, 3..3, 0, Stack),
    op!(EOR, 2..2, NZ, Imm),
    op!(LSR, 2..2, NZC, Acc),
    ILLEGAL,
    op!(JMP, 3..3, 0, Abs),
    op!(EOR, 4..4, NZ, Abs),
    op!(LSR, 7..7, NZC, Abs),
    op!(BBR4, 0..0, 0, PcRel), // W65C02S only
    // 0x50
    op!(BVC, 2..3, 0, PcRel),      // Cycles: 3+/2
    op!(EOR, 5..5, NZ, ZpIndIdxY), // Cycles: 5+
    op!(EOR, 0..0, NZ, ZpInd),
    ILLEGAL,
    ILLEGAL,
    op!(EOR, 4..4, NZ, ZpIdxX),
    op!(LSR, 6..6, NZC, ZpIdxX),
    op!(RMB5, 0..0, 0, Zp), // W65C02S only
    op!(CLI, 2..2, I, Imp),
    op!(EOR, 4..4, NZ, AbsIdxY), // Cycles: 4+
    op!(PHY, 0..0, 0, Stack),    // W65C02S only
    ILLEGAL,
    ILLEGAL,
    op!(EOR, 4..4, NZ, AbsIdxX), // Cycles: 4+
    op!(LSR, 7..7, NZC, AbsIdxX),
    op!(BBR5, 0..0, 0, PcRel), // W65C02S only
    // 0x60
    op!(RTS, 6..6, 0, Stack),
    op!(ADC, 6..6, NZCV, ZpIdxInd),
    ILLEGAL,
    ILLEGAL,
    op!(STZ, 0..0, 0, Zp),
    op!(ADC, 3..3, NZCV, Zp),
    op!(ROR, 5..5, NZC, Zp),
    op!(RMB6, 0..0, 0, Zp), // W65C02S only
    op!(PLA, 4..4, NZ, Stack),
    op!(ADC, 2..2, NZCV, Imm),
    op!(ROR, 2..2, NZC, Acc),
    ILLEGAL,
    op!(JMP, 5..5, 0, AbsInd),
    op!(ADC, 4..4, NZCV, Abs),
    op!(ROR, 6..6, NZC, Abs),
    op!(BBR6, 0..0, 0, PcRel), // W65C02S only
    // 0x70
    op!(BVS, 2..3, 0, PcRel),        // Cycles: 3+/2
    op!(ADC, 5..5, NZCV, ZpIndIdxY), // Cycles: 5+
    op!(ADC, 0..0, NZCV, ZpInd),
    ILLEGAL,
    op!(STZ, 0..0, 0, ZpIdxX), // W65C02S only
    op!(ADC, 4..4, NZCV, ZpIdxX),
    op!(ROR, 6..6, NZC, ZpIdxX),
    op!(RMB7, 0..0, 0, Zp), // W65C02S only
    op!(SEI, 2..2, I, Imp),
    op!(ADC, 4..4, NZCV, AbsIdxY), // Cycles: 4+
    op!(PLY, 0..0, NZ, Stack),     // W65C02S only
    ILLEGAL,
    op!(JMP, 0..0, 0, AbsIdxInd),  // Old op with new addr mode
    op!(ADC, 4..4, NZCV, AbsIdxX), // Cycles: 4+
    op!(ROR, 7..7, NZC, AbsIdxX),
    op!(BBR7, 0..0, 0, PcRel), // W65C02S only
    // 0x80
    op!(BRA, 0..0, 0, PcRel),
    op!(STA, 6..6, 0, ZpIdxInd),
    ILLEGAL,
    ILLEGAL,
    op!(STY, 3..3, 0, Zp),
    op!(STA, 3..3, 0, Zp),
    op!(STX, 3..3, 0, Zp),
    op!(SMB0, 0..0, 0, Zp), // W65C02S only
    op!(DEY, 2..2, NZ, Imp),
    op!(BIT, 0..0, NZC, Imm), // Old op with new addr mode
    op!(TXA, 2..2, NZ, Imp),
    ILLEGAL,
    op!(STY, 4..4, 0, Abs),
    op!(STA, 4..4, 0, Abs),
    op!(STX, 4..4, 0, Abs),
    op!(BBS0, 0..0, 0, PcRel), // W65C02S only
    // 0x90
    op!(BCC, 2..3, 0, PcRel), // Cycles: 3+/2
    op!(STA, 6..6, 0, ZpIndIdxY),
    op!(STA, 0..0, 0, ZpInd), // Old op with new addr mode
    ILLEGAL,
    op!(STY, 4..4, 0, ZpIdxX),
    op!(STA, 4..4, 0, ZpIdxX),
    op!(STX, 4..4, 0, ZpIdxY),
    op!(SMB1, 0..0, 0, Zp), // W65C02S only
    op!(TYA, 2..2, NZ, Imp),
    op!(STA, 5..5, 0, AbsIdxY),
    op!(TXS, 2..2, 0, Imp),
    ILLEGAL,
    op!(STZ, 0..0, 0, Abs), // W65C02S only
    op!(STA, 5..5, 0, AbsIdxX),
    op!(STZ, 0..0, 0, AbsIdxX), // W65C02S only
    op!(BBS1, 0..0, 0, PcRel),  // W65C02S only
    // 0xA0
    op!(LDY, 2..2, NZ, Imm),
    op!(LDA, 6..6, NZ, ZpIdxInd),
    op!(LDX, 2..2, NZ, Imm),
    ILLEGAL,
    op!(LDY, 3..3, NZ, Zp),
    op!(LDA, 3..3, NZ, Zp),
    op!(LDX, 3..3, NZ, Zp),
    op!(SMB2, 0..0, 0, Zp), // W65C02S only
    op!(TAY, 2..2, NZ, Imp),
    op!(LDA, 2..2, NZ, Imm),
    op!(TAX, 2..2, NZ, Imp),
    ILLEGAL,
    op!(LDY, 4..4, NZ, Abs),
    op!(LDA, 4..4, NZ, Abs),
    op!(LDX, 4..4, NZ, Abs),
    op!(BBS2, 0..0, 0, PcRel), // W65C02S only
    // 0xB0
    op!(BCS, 2..3, 0, PcRel),      // Cycles: 3+/2
    op!(LDA, 5..5, NZ, ZpIndIdxY), // Cycles: 5+
    op!(LDA, 0..0, NZ, ZpInd),
    ILLEGAL,
    op!(LDY, 4..4, NZ, ZpIdxX),
    op!(LDA, 4..4, NZ, ZpIdxX),
    op!(LDX, 4..4, NZ, ZpIdxY),
    op!(SMB3, 0..0, 0, Zp), // W65C02S only
    op!(CLV, 2..2, V, Imp),
    op!(LDA, 4..4, NZ, AbsIdxY), // Cycles: 4+
    op!(TSX, 2..2, NZ, Imp),
    ILLEGAL,
    op!(LDY, 4..4, NZ, AbsIdxX), // Cycles: 4+
    op!(LDA, 4..4, NZ, AbsIdxX), // Cycles: 4+
    op!(LDX, 4..4, NZ, AbsIdxY), // Cycles: 4+
    op!(BBS3, 0..0, 0, PcRel),   // W65C02S only
    // 0xC0
    op!(CPY, 2..2, NZC, Imm),
    op!(CMP, 6..6, NZC, ZpIdxInd),
    ILLEGAL,
    ILLEGAL,
    op!(CPY, 3..3, NZC, Zp),
    op!(CMP, 3..3, NZC, Zp),
    op!(DEC, 5..5, NZ, Zp),
    op!(SMB4, 0..0, 0, Zp), // W65C02S only
    op!(INY, 2..2, NZ, Imp),
    op!(CMP, 2..2, NZC, Imm),
    op!(DEX, 2..2, NZ, Imp),
    op!(WAI, 0..0, 0, Imp), // W65C02S only
    op!(CPY, 4..4, NZC, Abs),
    op!(CMP, 4..4, NZC, Abs),
    op!(DEC, 6..6, NZ, Abs),
    op!(BBS4, 0..0, 0, PcRel), // W65C02S only
    // 0xD0
    op!(BNE, 2..3, 0, PcRel),       // Cycles: 3+/2
    op!(CMP, 5..5, NZC, ZpIndIdxY), // Cycles: 5+
    op!(CMP, 0..0, NZC, ZpInd),
    ILLEGAL,
    ILLEGAL,
    op!(CMP, 4..4, NZC, ZpIdxX),
    op!(DEC, 6..6, NZ, ZpIdxX),
    op!(SMB5, 0..0, 0, Zp), // W65C02S only
    op!(CLD, 2..2, D, Imp),
    op!(CMP, 4..4, NZC, AbsIdxY), // Cycles: 4+
    op!(PHX, 0..0, 0, Stack),
    op!(STP, 0..0, 0, Imp),
    ILLEGAL,
    op!(CMP, 4..4, NZC, AbsIdxX), // Cycles: 4+
    op!(DEC, 7..7, NZ, AbsIdxX),
    op!(BBS5, 0..0, 0, PcRel), // W65C02S only
    // 0xE0
    op!(CPX, 2..2, NZC, Imm),
    op!(SBC, 6..6, NZCV, ZpIdxInd),
    ILLEGAL,
    ILLEGAL,
    op!(CPX, 3..3, NZC, Zp),
    op!(SBC, 3..3, NZCV, Zp),
    op!(INC, 5..5, NZ, Zp),
    op!(SMB6, 0..0, 0, Zp), // W65C02S only
    op!(INX, 2..2, NZ, Imp),
    op!(SBC, 2..2, NZCV, Imm),
    op!(NOP, 2..2, 0, Imp),
    ILLEGAL,
    op!(CPX, 4..4, NZC, Abs),
    op!(SBC, 4..4, NZCV, Abs),
    op!(INC, 6..6, NZ, Abs),
    op!(BBS6, 0..0, 0, PcRel), // W65C02S only
    // 0xF0
    op!(BEQ, 2..3, 0, PcRel),        // Cycles: 3+/2
    op!(SBC, 5..5, NZCV, ZpIndIdxY), // Cycles: 5+
    op!(SBC, 0..0, NZCV, ZpInd),     // Old op with new addr mode
    ILLEGAL,
    ILLEGAL,
    op!(SBC, 4..4, NZCV, ZpIdxX),
    op!(INC, 6..6, NZ, ZpIdxX),
    op!(SMB7, 0..0, 0, Zp), // W65C02S only
    op!(SED, 2..2, D, Imp),
    op!(SBC, 4..4, NZCV, AbsIdxY), // Cycles: 4+
    op!(PLX, 0..0, NZ, Stack),
    ILLEGAL,
    ILLEGAL,
    op!(SBC, 4..4, NZCV, AbsIdxX), // Cycles: 4+
    op!(INC, 7..7, NZ, AbsIdxX),
    op!(BBS7, 0..0, 0, PcRel), // W65C02S only
];

pub fn disassemble_one(bus: &impl MemoryMapped, offset: usize, next: &mut usize) -> String {
    let offs = offset as u16;

    let code = bus.read(offset);
    let o = &OPS[usize::from(code)];
    *next = usize::from(offs + op_len(&o.adr));

    let arg = match o.adr {
        AdrMode::Abs => format!("${:02x}{:02x}", bus.read(offset + 2), bus.read(offset + 1)),
        AdrMode::AbsIdxInd => format!(
            "(${:02x}{:02x},X)",
            bus.read(offset + 2),
            bus.read(offset + 1)
        ),
        AdrMode::AbsIdxX => format!(
            "${:02x}{:02x},X",
            bus.read(offset + 2),
            bus.read(offset + 1),
        ),
        AdrMode::AbsIdxY => format!(
            "${:02x}{:02x},Y",
            bus.read(offset + 2),
            bus.read(offset + 1),
        ),
        AdrMode::AbsInd => format!(
            "(${:02x}{:02x})",
            bus.read(offset + 2),
            bus.read(offset + 1)
        ),
        AdrMode::Acc | AdrMode::Imp | AdrMode::Stack => format!(""),
        AdrMode::Imm => format!("#${:02x}", bus.read(offset + 1)),
        AdrMode::PcRel => format!("*{:+}", bus.read(offset + 1) as i8),
        AdrMode::Zp => format!("${:02x}", bus.read(offset + 1)),
        AdrMode::ZpIdxInd => format!("(${:02x},X)", bus.read(offset + 1)),
        AdrMode::ZpIdxX => format!("${:02x},X", bus.read(offset + 1)),
        AdrMode::ZpIdxY => format!("${:02x},Y", bus.read(offset + 1)),
        AdrMode::ZpInd => format!("(${:02x})", bus.read(offset + 1)),
        AdrMode::ZpIndIdxY => format!("(${:02x}),Y", bus.read(offset + 1)),
    };

    format!("{:?}     {}", o.name, arg)
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            p: 0,
            sp: 0,
            pc: 0,
            ir: 0,
            op_cycle: 0,
            op_a: 0,
            op_b: 0,
            op_latch: 0,
            op_ptr: 0,
            op_branch: false,
            op_fixed_pc: 0,
            op_offs: 0,
            next_ir: None,
            cycles: 0,
            sync: false,
            wr: false,
            adr: 0,
            data: 0,
        }
    }

    pub fn reset(&mut self, bus: &impl MemoryMapped) {
        // This is a bit of a hack, fast-forwarding the first
        // 7 cycles after reset.
        //
        // Details: https://www.pagetable.com/?p=410
        //
        // The `perfect6502` emulator, based on the `visual6502`
        // emulator, initialize X, P and SP with these seemingly
        // random values at cycle 7.
        self.a = 0;
        self.x = 0xC0;
        self.y = 0;
        self.p = 0x16;
        self.sp = 0xBD;

        self.pc = lo_hi_u16(bus.read(0xFFFC), bus.read(0xFFFD));
        self.op_offs = self.pc;

        self.ir = 0;
        self.op_cycle = 0;
        self.sync = true;
    }

    fn set_negative_flag(&mut self, en: bool) {
        if en {
            self.p |= NEG_MASK;
        } else {
            self.p &= !NEG_MASK;
        }
    }

    fn set_overflow_flag(&mut self, en: bool) {
        if en {
            self.p |= OVERFLOW_MASK;
        } else {
            self.p &= !OVERFLOW_MASK;
        }
    }

    fn set_carry_flag(&mut self, en: bool) {
        if en {
            self.p |= CARRY_MASK;
        } else {
            self.p &= !CARRY_MASK;
        }
    }

    fn set_decimal_mode_flag(&mut self, en: bool) {
        if en {
            self.p |= DEC_MASK;
        } else {
            self.p &= !DEC_MASK;
        }
    }

    fn set_zero_flag(&mut self, en: bool) {
        if en {
            self.p |= ZERO_MASK;
        } else {
            self.p &= !ZERO_MASK;
        }
    }

    fn set_interrupt_disable_flag(&mut self, b: bool) {
        if b {
            self.p |= IRQB_DISABLE_MASK;
        } else {
            self.p &= !IRQB_DISABLE_MASK;
        }
    }

    fn alu_load(&mut self, a: u8) -> u8 {
        self.set_negative_flag(a & 0x80 != 0);
        self.set_zero_flag(a == 0);
        a
    }

    fn _alu_add(&mut self, a: u8, b: u8) -> u8 {
        let (result, ovf) = a.overflowing_add(b);
        self.set_overflow_flag(ovf);
        self.set_negative_flag(result & 0x80 != 0);
        self.set_zero_flag(result == 0);
        result
    }

    pub fn go_to(&mut self, adr: u16) {
        if self.op_cycle != 0 {
            panic!("go_to forbidden while executing instruction")
        }
        self.pc = adr;
        self.op_offs = adr;
    }

    /*
    pub fn xx_one_cycle(&mut self, data: u8) {
        // Panic when an unimplemented operation is reached
        fn not_implemented(op: &Op) -> ! {
            panic!(
                "Op {:?} in adr mode {:?} is not implemented!",
                op.name, op.adr
            )
        }

        macro_rules! read_next {
            () => {{
                self.pc = self.pc.wrapping_add(1);
                self.adr = self.pc;
            }};
        }

        macro_rules! fetch {
            () => {{
                read_next!();
                self.sync = true;
            }};
        }

        // Panic when an invalid op cycle is reached
        macro_rules! invalid_op_cycle {
            () => {{
                panic!("invalid op cycle: {}", self.op_cycle);
            }};
        }

        self.cycles = self.cycles.wrapping_add(1);
        self.data = data;

        if self.sync {
            println!("Sync. New IR: {:02x}. PC: {}", data, self.pc);
            self.ir = self.data;
            self.op_offs = self.adr;
            self.op_cycle = 1;
            self.sync = false;
            read_next!();
            return;
        }

        let op = &OPS[usize::from(self.ir)];

        match op.adr {
            AdrMode::Imm => match op.name {
                OpName::LDX => match self.op_cycle {
                    0 => {
                        println!("Executing cycle 1.");
                        self.x = self.alu_load(data);
                        fetch!();
                    }
                    _ => invalid_op_cycle!(),
                },
                _ => not_implemented(op),
            },

            AdrMode::Imp => match op.name {
                OpName::DEX => match self.op_cycle {
                    0 => {
                        self.x = self.alu_load(self.x.wrapping_sub(1));
                        fetch!();
                    }
                    _ => invalid_op_cycle!(),
                },

                OpName::NOP => match self.op_cycle {
                    1 => fetch!(),
                    _ => invalid_op_cycle!(),
                },

                _ => not_implemented(op),
            },

            _ => unreachable!("AdrMode not implemented: {:?}", op.adr),
        }

        self.op_cycle += 1;
    }
    */

    pub fn one_cycle(&mut self, bus: &mut impl MemoryMapped) {
        self.cycles = self.cycles.wrapping_add(1);

        // Panic when an unimplemented operation is reached
        fn not_implemented(op: &Op) -> ! {
            panic!(
                "Op {:?} in adr mode {:?} is not implemented!",
                op.name, op.adr
            )
        }

        macro_rules! sync {
            // Set start of next operation to self.pc
            () => {{
                self.sync = true;
                self.op_offs = self.pc;
            }};
        }

        macro_rules! fetch {
            () => {{
                self.pc = self.pc.wrapping_add(1);
                self.adr = self.pc;
                self.sync = true;
            }};
        }

        // Panic when an invalid op cycle is reached
        macro_rules! invalid_op_cycle {
            () => {{
                panic!("invalid op cycle: {}", self.op_cycle);
            }};
        }

        macro_rules! inc_sp {
            () => {{
                self.sp = self.sp.wrapping_add(1);
            }};
        }

        macro_rules! inc_pc {
            () => {{
                self.pc = self.pc.wrapping_add(1);
            }};
        }

        macro_rules! exec_cmp_cpx_cpy {
            ($reg:expr, $mem:expr) => {{
                let (r, ovf) = $reg.overflowing_sub($mem);
                self.set_negative_flag(r & 0x80 != 0);
                self.set_zero_flag(r == 0);
                self.set_carry_flag(!ovf);
            }};
        }

        macro_rules! exec_adc {
            ($mem:expr) => {{
                let (r, ovf) = self.a.overflowing_add($mem);
                self.a = r;
                self.set_negative_flag(r & 0x80 != 0);
                self.set_zero_flag(r == 0);
                self.set_carry_flag(ovf);
                self.set_overflow_flag(ovf);
            }};
        }

        macro_rules! exec_sbc {
            ($mem:expr) => {{
                let (r, ovf) = self.a.overflowing_sub($mem);
                self.a = r;
                self.set_negative_flag(r & 0x80 != 0);
                self.set_zero_flag(r == 0);
                self.set_carry_flag(ovf);
                self.set_overflow_flag(ovf);
            }};
        }

        macro_rules! exec_bit {
            ($mem:expr) => {{
                self.set_zero_flag(self.p & ($mem) == 0);
                self.p = (self.p & 0b0011_1111) | (($mem) & 0b1100_0000);
            }};
        }

        macro_rules! group {
            ($op:ident) => { OpName::$op };
            ($op:ident, $($rest:ident),+) => { OpName::$op | group!($($rest),+) };
        }

        macro_rules! push {
            ($value:expr) => {{
                bus.write(self.sp as usize + 0x100, $value);
                self.sp = self.sp.wrapping_sub(1);
            }};
        }

        // First get the op currently executing
        let op = &OPS[usize::from(self.ir)];

        // If this is a sync cycle, read new operation into IR
        let mut next_op_cycle = self.op_cycle + 1;
        if self.sync {
            self.ir = self.read_pc(bus);
            self.sync = false;
            next_op_cycle = 1;
        }

        match op.adr {
            AdrMode::Acc => not_implemented(op),

            AdrMode::Stack => match op.name {
                OpName::BRK => match self.op_cycle {
                    0 => {}
                    1 => {
                        // self.adr = self.pc;
                        throw_away!(self.read_pc(bus));
                    }
                    2 => bus.write(self.sp as usize + 0x100, self.pch()),
                    3 => bus.write(self.sp as usize + 0x100 - 1, self.pcl()),
                    4 => {
                        bus.write(self.sp as usize + 0x100 - 2, self.p | 0b110000);
                        self.sp = self.sp.wrapping_sub(3);
                    }
                    5 => {
                        self.op_ptr = bus.read(0xFFFE) as u16;
                        self.set_interrupt_disable_flag(true);
                    }
                    6 => {
                        self.pc = (bus.read(0xFFFF) as u16) << 8 | self.op_ptr;
                        sync!();
                    }
                    7 => {}
                    _ => invalid_op_cycle!(),
                },

                OpName::PHA | OpName::PHP => match self.op_cycle {
                    1 => throw_away!(bus.read(self.pc.into())),
                    2 => {
                        push!(match op.name {
                            OpName::PHA => self.a,
                            OpName::PHP => self.p | 0b110000,
                            _ => unreachable!(),
                        });
                        sync!();
                    }
                    3 => {}
                    _ => invalid_op_cycle!(),
                },

                OpName::PLA => match self.op_cycle {
                    1 => throw_away!(bus.read(self.pc.into())),
                    2 => inc_sp!(),
                    3 => {
                        self.a = self.alu_load(self.read_stack(bus));
                        sync!();
                    }
                    4 => {}
                    _ => invalid_op_cycle!(),
                },

                OpName::PLP => match self.op_cycle {
                    1 => throw_away!(bus.read(self.pc.into())),
                    2 => inc_sp!(),
                    3 => {
                        self.p = (self.read_stack(bus) & 0xDF) | 0x10;
                        sync!();
                    }
                    4 => {}
                    _ => invalid_op_cycle!(),
                },

                OpName::RTI => match self.op_cycle {
                    1 => throw_away!(self.read_pc(bus)),
                    2 => {}
                    3 => {
                        self.p = (bus.read(self.sp as usize + 0x101) & 0b1100_1111) | BRK_MASK;
                    }
                    4 => {
                        self.op_ptr = bus.read(self.sp as usize + 0x102) as u16;
                        self.sp = self.sp.wrapping_add(3);
                    }
                    5 => {
                        let hi = bus.read(self.sp as usize + 0x100);
                        self.pc = self.op_ptr | ((hi as u16) << 8);
                        sync!();
                    }
                    6 => {}
                    _ => invalid_op_cycle!(),
                },

                OpName::RTS => match self.op_cycle {
                    // Similar to JSR, RTS juggles values around and bit.
                    // See for example SP and P in cycle 4 and 5.
                    1 => throw_away!(self.read_pc(bus)),
                    2 => {}
                    3 => {
                        self.op_latch = self.p;
                        self.p = self.sp;

                        inc_sp!();
                        self.op_ptr = self.read_stack(bus).into();
                        inc_sp!();
                    }
                    4 => {
                        self.p = self.op_latch;

                        self.pc = self.op_ptr | ((self.read_stack(bus) as u16) << 8);
                    }
                    5 => {
                        inc_pc!();
                        sync!();
                    }
                    6 => {}
                    _ => invalid_op_cycle!(),
                },

                _ => not_implemented(op),
            },

            AdrMode::Abs => match op.name {
                OpName::JMP => match self.op_cycle {
                    1 => self.op_a = self.read_pc(bus),
                    2 => {
                        self.pc = lo_hi_u16(self.op_a, self.read_pc(bus));
                        sync!();
                    }
                    3 => {}
                    _ => invalid_op_cycle!(),
                },
                OpName::JSR => match self.op_cycle {
                    // JSR is a bit crazy in how it seems to juggle with PC and SP.
                    // This implementation has been verified to match visual6502.
                    1 => {
                        // JSR temporarily stores the low address in the stack pointer
                        self.op_latch = self.sp;
                        self.sp = self.read_pc(bus);
                    }
                    2 => self.op_b = bus.read(self.pc.into()),
                    3 => {
                        // Push PCH. Note that SP is currently invalid.
                        bus.write(self.op_latch as usize + 0x100, self.pch());
                        self.op_latch = self.op_latch.wrapping_sub(1);
                    }
                    4 => {
                        // Push PCL. Note that SP is currently invalid.
                        bus.write(self.op_latch as usize + 0x100, self.pcl());
                        self.op_latch = self.op_latch.wrapping_sub(1);
                    }
                    5 => {
                        self.op_b = self.read_pc(bus);
                        self.pc = lo_hi_u16(self.sp, self.op_b);
                        self.sp = self.op_latch;
                        sync!();
                    }
                    6 => {}
                    _ => invalid_op_cycle!(),
                },

                group!(LDA, LDX, LDY, CMP, CPX, CPY) => match self.op_cycle {
                    1 => self.op_a = self.read_pc(bus),
                    2 => self.op_b = self.read_pc(bus),
                    3 => {
                        self.op_a = bus.read(lo_hi_u16(self.op_a, self.op_b).into());
                        match op.name {
                            OpName::LDA => self.a = self.alu_load(self.op_a),
                            OpName::LDX => self.x = self.alu_load(self.op_a),
                            OpName::LDY => self.y = self.alu_load(self.op_a),
                            group!(CMP, CPX, CPY) => {}
                            _ => not_implemented(op),
                        }
                        sync!();
                    }
                    4 => match op.name {
                        OpName::CMP => exec_cmp_cpx_cpy!(self.a, self.op_a),
                        OpName::CPX => exec_cmp_cpx_cpy!(self.x, self.op_a),
                        OpName::CPY => exec_cmp_cpx_cpy!(self.y, self.op_a),
                        group!(LDA, LDX, LDY) => {}
                        _ => not_implemented(op),
                    },
                    _ => invalid_op_cycle!(),
                },

                OpName::STA | OpName::STX | OpName::STY => match self.op_cycle {
                    1 => self.op_a = self.read_pc(bus),
                    2 => self.op_b = self.read_pc(bus),
                    3 => match op.name {
                        OpName::STA => {
                            bus.write(lo_hi_u16(self.op_a, self.op_b).into(), self.a);
                            sync!();
                        }
                        OpName::STX => {
                            bus.write(lo_hi_u16(self.op_a, self.op_b).into(), self.x);
                            sync!();
                        }
                        OpName::STY => {
                            bus.write(lo_hi_u16(self.op_a, self.op_b).into(), self.y);
                            sync!();
                        }
                        _ => not_implemented(op),
                    },
                    4 => {}
                    _ => invalid_op_cycle!(),
                },
                _ => not_implemented(op),
            },

            AdrMode::AbsIdxInd => not_implemented(op),

            AdrMode::AbsIdxX | AdrMode::AbsIdxY => {
                let reg = match op.adr {
                    AdrMode::AbsIdxX => self.x,
                    AdrMode::AbsIdxY => self.y,
                    _ => unreachable!(),
                };

                match op.name {
                    // Read instructions
                    group!(LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT) => {
                        let mut ready = false;

                        match self.op_cycle {
                            1 => self.op_a = self.read_pc(bus),
                            2 => self.op_b = self.read_pc(bus),
                            3 => {
                                let (a, ovf) = self.op_a.overflowing_add(reg);
                                let adr = lo_hi_u16(a, self.op_b);
                                self.op_latch = bus.read(adr as usize);
                                ready = !ovf;
                            }
                            4 => {
                                let adr = lo_hi_u16(self.op_a, self.op_b).wrapping_add(reg.into());
                                self.op_latch = bus.read(adr as usize);
                                ready = true;
                            }
                            5 => match op.name {
                                OpName::ADC => exec_adc!(self.op_latch),
                                OpName::SBC => exec_sbc!(self.op_latch),
                                OpName::CMP => exec_cmp_cpx_cpy!(self.a, self.op_latch),
                                OpName::BIT => exec_bit!(self.op_latch),
                                _ => unreachable!(),
                            },
                            6 => {}
                            _ => invalid_op_cycle!(),
                        }

                        if ready {
                            next_op_cycle = 6;

                            match op.name {
                                OpName::LDA => self.a = self.alu_load(self.op_latch),
                                OpName::LDX => self.x = self.alu_load(self.op_latch),
                                OpName::LDY => self.y = self.alu_load(self.op_latch),
                                OpName::EOR => self.a = self.alu_load(self.a ^ self.op_latch),
                                OpName::AND => self.a = self.alu_load(self.a & self.op_latch),
                                OpName::ORA => self.a = self.alu_load(self.a | self.op_latch),
                                _ => next_op_cycle = 5,
                            }

                            sync!();
                        }
                    }

                    // Write instructions
                    group!(STA, STX, STY) => match self.op_cycle {
                        1 => self.op_a = self.read_pc(bus),
                        2 => {
                            self.op_b = self.read_pc(bus);
                            self.op_ptr = lo_hi_u16(self.op_a.wrapping_add(reg), self.op_b);
                        }
                        3 => {
                            throw_away!(bus.read(self.op_ptr.into()));
                            self.op_ptr = lo_hi_u16(self.op_a, self.op_b).wrapping_add(reg.into());
                        }
                        4 => {
                            let v = match op.name {
                                OpName::STA => self.a,
                                OpName::STX => self.x,
                                OpName::STY => self.y,
                                _ => unreachable!(),
                            };
                            bus.write(self.op_ptr.into(), v);
                            sync!();
                        }
                        5 => {}
                        _ => invalid_op_cycle!(),
                    },

                    // Read-Modify-Write instructions
                    group!(INC) => match self.op_cycle {
                        1 => self.op_a = self.read_pc(bus),
                        2 => {
                            self.op_b = self.read_pc(bus);
                            self.op_ptr = lo_hi_u16(self.op_a.wrapping_add(reg), self.op_b);
                        }
                        3 => {
                            throw_away!(bus.read(self.op_ptr.into()));
                            self.op_ptr = lo_hi_u16(self.op_a, self.op_b).wrapping_add(reg.into());
                        }
                        4 => {
                            self.op_a = bus.read(self.op_ptr.into());
                        }
                        5 => {
                            bus.write(self.op_ptr.into(), self.op_a);
                            self.op_a = self.alu_load(self.op_a.wrapping_add(1));
                            sync!();
                        }
                        6 => {
                            bus.write(self.op_ptr.into(), self.op_a);
                        }
                        _ => invalid_op_cycle!(),
                    },

                    _ => not_implemented(op),
                }
            }

            AdrMode::AbsInd => match op.name {
                OpName::JMP => match self.op_cycle {
                    1 => self.op_a = self.read_pc(bus),
                    2 => self.op_b = self.read_pc(bus),
                    3 => self.op_latch = bus.read(lo_hi_u16(self.op_a, self.op_b).into()),
                    4 => {
                        let pch = bus.read(lo_hi_u16(self.op_a.wrapping_add(1), self.op_b).into());
                        self.pc = lo_hi_u16(self.op_latch, pch);
                        sync!();
                    }
                    5 => {}
                    _ => invalid_op_cycle!(),
                },
                _ => not_implemented(op),
            },

            AdrMode::Imp => match self.op_cycle {
                1 => {
                    match op.name {
                        OpName::TAX => self.x = self.alu_load(self.a),
                        OpName::TAY => self.y = self.alu_load(self.a),
                        OpName::TXA => self.a = self.alu_load(self.x),
                        OpName::TSX => self.x = self.alu_load(self.sp),
                        OpName::TXS => self.sp = self.x,
                        OpName::TYA => self.a = self.alu_load(self.y),
                        OpName::CLC => self.set_carry_flag(false),
                        OpName::SEC => self.set_carry_flag(true),
                        OpName::CLD => self.set_decimal_mode_flag(false),
                        OpName::CLI => self.set_interrupt_disable_flag(false),
                        OpName::CLV => self.set_overflow_flag(false),
                        OpName::SED => self.set_decimal_mode_flag(true),
                        OpName::SEI => self.set_interrupt_disable_flag(true),
                        _ => {}
                    }
                    sync!();
                }
                2 => {
                    match op.name {
                        OpName::CLC => {}
                        OpName::SEC => {}
                        OpName::SED => {}
                        OpName::SEI => {}
                        OpName::CLD => {}
                        OpName::CLI => {}
                        OpName::CLV => {}
                        OpName::DEX => self.x = self.alu_load(self.x.wrapping_add(-1 as i8 as u8)),
                        OpName::DEY => self.y = self.alu_load(self.y.wrapping_add(-1 as i8 as u8)),
                        OpName::INX => self.x = self.alu_load(self.x.wrapping_add(1)),
                        OpName::INY => self.y = self.alu_load(self.y.wrapping_add(1)),
                        OpName::NOP => {}
                        OpName::TAX => {}
                        OpName::TAY => {}
                        OpName::TSX => {}
                        OpName::TXS => {}
                        OpName::TXA => {}
                        OpName::TYA => {}
                        _ => not_implemented(op),
                    };
                }
                _ => invalid_op_cycle!(),
            },

            AdrMode::Imm => match self.op_cycle {
                1 => {
                    self.op_a = self.read_pc(bus);
                    match op.name {
                        OpName::LDA => self.a = self.alu_load(self.op_a),
                        OpName::LDX => self.x = self.alu_load(self.op_a),
                        OpName::LDY => self.y = self.alu_load(self.op_a),
                        OpName::CMP => {
                            println!("Next will sync!");
                        }
                        OpName::CPX => {}
                        OpName::CPY => {}
                        OpName::ADC => {}
                        OpName::EOR => {}
                        OpName::ORA => self.a = self.alu_load(self.a | self.op_a),
                        _ => not_implemented(op),
                    }
                    sync!();
                }
                2 => {
                    match op.name {
                        OpName::LDA => {}
                        OpName::CMP => exec_cmp_cpx_cpy!(self.a, self.op_a),
                        OpName::CPX => exec_cmp_cpx_cpy!(self.x, self.op_a),
                        OpName::CPY => exec_cmp_cpx_cpy!(self.y, self.op_a),
                        OpName::ADC => exec_adc!(self.op_a),
                        OpName::EOR => self.a = self.alu_load(self.a ^ self.op_a),
                        _ => {}
                    };
                }
                _ => invalid_op_cycle!(),
            },

            AdrMode::Zp => match op.name {
                group!(LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, CPX, CPY, BIT) => {
                    match self.op_cycle {
                        1 => self.op_a = self.read_pc(bus),
                        2 => {
                            self.op_latch = bus.read(self.op_a.into());
                            match op.name {
                                OpName::LDA => self.a = self.alu_load(self.op_latch),
                                OpName::LDX => self.x = self.alu_load(self.op_latch),
                                OpName::LDY => self.y = self.alu_load(self.op_latch),
                                OpName::EOR => self.a = self.alu_load(self.a ^ self.op_latch),
                                OpName::AND => self.a = self.alu_load(self.a & self.op_latch),
                                OpName::ORA => self.a = self.alu_load(self.a | self.op_latch),
                                OpName::BIT => exec_bit!(self.op_latch),
                                _ => {}
                            };
                            sync!();
                        }
                        3 => match op.name {
                            OpName::ADC => exec_adc!(self.op_latch),
                            OpName::SBC => exec_sbc!(self.op_latch),
                            OpName::CMP => exec_cmp_cpx_cpy!(self.a, self.op_latch),
                            OpName::CPX => exec_cmp_cpx_cpy!(self.x, self.op_latch),
                            OpName::CPY => exec_cmp_cpx_cpy!(self.y, self.op_latch),
                            _ => {}
                        },
                        _ => invalid_op_cycle!(),
                    }
                }

                OpName::STA | OpName::STX | OpName::STY => match self.op_cycle {
                    1 => self.op_a = self.read_pc(bus),
                    2 => {
                        let v = match op.name {
                            OpName::STA => self.a,
                            OpName::STX => self.x,
                            OpName::STY => self.y,
                            _ => unreachable!(),
                        };
                        bus.write(self.op_a.into(), v);
                        sync!();
                    }
                    3 => {}
                    _ => invalid_op_cycle!(),
                },
                _ => not_implemented(op),
            },

            AdrMode::ZpIdxX | AdrMode::ZpIdxY => match op.name {
                group!(LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT) => match self.op_cycle {
                    1 => self.op_a = self.read_pc(bus),
                    2 => {
                        let idx = match op.adr {
                            AdrMode::ZpIdxX => self.x,
                            AdrMode::ZpIdxY => self.y,
                            _ => not_implemented(op),
                        };

                        throw_away!(bus.read(self.op_a.into()));
                        self.op_a = self.op_a.wrapping_add(idx);
                    }
                    3 => {
                        self.op_latch = bus.read(self.op_a.into());
                        println!(
                            "ZpIdxY: READ FROM {:04x} = {:02x}",
                            self.op_a, self.op_latch
                        );
                        match op.name {
                            OpName::LDA => self.a = self.alu_load(self.op_latch),
                            OpName::LDX => self.x = self.alu_load(self.op_latch),
                            OpName::LDY => self.y = self.alu_load(self.op_latch),
                            OpName::EOR => self.a = self.alu_load(self.a ^ self.op_latch),
                            OpName::AND => self.a = self.alu_load(self.a & self.op_latch),
                            OpName::ORA => self.a = self.alu_load(self.a | self.op_latch),
                            OpName::ADC => {}
                            OpName::SBC => {}
                            OpName::CMP => {}
                            OpName::BIT => exec_bit!(self.op_latch),
                            _ => not_implemented(op),
                        }
                        sync!();
                    }
                    4 => match op.name {
                        OpName::ADC => exec_adc!(self.op_latch),
                        OpName::SBC => exec_sbc!(self.op_latch),
                        OpName::CMP => exec_cmp_cpx_cpy!(self.a, self.op_latch),
                        _ => {}
                    },
                    _ => invalid_op_cycle!(),
                },

                OpName::STA | OpName::STY | OpName::STX => match self.op_cycle {
                    1 => self.op_a = self.read_pc(bus),
                    2 => throw_away!(bus.read(self.op_a.into())),
                    3 => {
                        let i = match op.adr {
                            AdrMode::ZpIdxX => self.x,
                            AdrMode::ZpIdxY => self.y,
                            _ => not_implemented(op),
                        };
                        let v = match op.name {
                            OpName::STA => self.a,
                            OpName::STX => self.x,
                            OpName::STY => self.y,
                            _ => not_implemented(op),
                        };
                        bus.write(self.op_a.wrapping_add(i).into(), v);
                        sync!();
                    }
                    4 => {}
                    _ => invalid_op_cycle!(),
                },
                _ => not_implemented(op),
            },

            AdrMode::ZpIdxInd => match op.name {
                OpName::ORA => match self.op_cycle {
                    // Fetch pointer address, increment PC:
                    1 => self.op_a = self.read_pc(bus),
                    // Read from the address, add X to it:
                    2 => self.op_a = bus.read(self.op_a.into()).wrapping_add(self.x),
                    // Fetch effective address (low)
                    3 => self.op_b = bus.read(self.op_a.into()).into(),
                    // Fetch effective address (high)
                    4 => {
                        let hi = bus.read(self.op_a.wrapping_add(1).into());
                        self.op_ptr = lo_hi_u16(self.op_b, hi);
                    }
                    // Read from the effective address
                    5 => {
                        self.a |= bus.read(self.op_ptr.into());
                        self.set_negative_flag(self.a & 0x80 != 0);
                        self.set_zero_flag(self.a == 0);
                        sync!();
                    }
                    _ => invalid_op_cycle!(),
                },
                _ => not_implemented(op),
            },

            AdrMode::ZpInd => not_implemented(op),
            AdrMode::ZpIndIdxY => not_implemented(op), CONTINUE HERE

            AdrMode::PcRel => match op.name {
                group!(BCC, BCS, BNE, BEQ, BPL, BMI, BVC, BVS) => match self.op_cycle {
                    1 => {
                        self.op_a = self.read_pc(bus);
                        self.op_branch = match op.name {
                            OpName::BNE => self.p & ZERO_MASK == 0,
                            OpName::BEQ => self.p & ZERO_MASK != 0,
                            OpName::BPL => self.p & NEG_MASK == 0,
                            OpName::BMI => self.p & NEG_MASK != 0,
                            OpName::BCC => self.p & CARRY_MASK == 0,
                            OpName::BCS => self.p & CARRY_MASK != 0,
                            OpName::BVC => self.p & OVERFLOW_MASK == 0,
                            OpName::BVS => self.p & OVERFLOW_MASK != 0,
                            _ => not_implemented(op),
                        };

                        if !self.op_branch {
                            sync!();
                        }
                    }
                    2 => {
                        // Fetch opcode of next instruction
                        let next_ir = bus.read(self.pc.into());
                        self.next_ir = Some(next_ir);

                        println!(
                            "Branch operand: signed={}, unsigned={} @{:04x}",
                            self.op_a as i8, self.op_a, self.pc,
                        );

                        if self.op_branch {
                            println!("branch taken");
                            // If branch is taken, add operand to PCL
                            self.op_fixed_pc = self.pc.wrapping_add(self.op_a as i8 as u16);
                            self.pc = (self.pc & 0xFF00) | (self.op_fixed_pc & 0xFF);
                            println!("op_fixed_pc={}, pc={}", self.op_fixed_pc, self.pc);
                            if self.op_fixed_pc == self.pc {
                                sync!();
                            }
                        }
                    }
                    3 => {
                        // Fetch opcode of next instruction
                        self.next_ir = Some(bus.read(self.pc.into()));

                        // Fix PCH
                        if self.op_fixed_pc.wrapping_add(1) == self.pc {
                            println!("pch was correct: {:04x}. end of op.", self.op_fixed_pc);
                            self.op_offs = self.pc;
                            self.op_cycle = 0;
                        } else {
                            println!(
                                "pch needs fixing. {:04x} vs {:04x}.",
                                self.op_fixed_pc, self.pc
                            );
                            self.pc = self.op_fixed_pc;
                            sync!();
                        }
                    }
                    4 => {
                        // Fetch opcode of next instruction, increment PC
                        self.next_ir = Some(bus.read(self.pc.into()));
                        println!("pch is now fixed. end of op.");
                    }
                    5 => {}
                    _ => invalid_op_cycle!(),
                },
                _ => not_implemented(op),
            },
            // _ => not_implemented(op),
        }

        self.op_cycle = next_op_cycle;
    }

    pub fn exec(&mut self, bus: &mut impl MemoryMapped) {
        self.one_cycle(bus);
        while !self.sync {
            self.one_cycle(bus);
        }
    }

    // Utility function: read byte at PC and increment PC
    fn read_pc(&mut self, bus: &mut impl MemoryMapped) -> u8 {
        let v = bus.read(self.pc.into());
        self.pc = self.pc.wrapping_add(1);
        v
    }

    // Utility function: read byte from stack, without changing the stack pointer
    fn read_stack(&self, bus: &impl MemoryMapped) -> u8 {
        bus.read(self.sp as usize + 0x100)
    }

    pub fn pcl(&self) -> u8 {
        (self.pc & 0xFF) as u8
    }

    pub fn pch(&self) -> u8 {
        ((self.pc >> 8) & 0xFF) as u8
    }

    pub fn set_pcl(&mut self, pcl: u8) {
        self.pc = (self.pc & 0xFF00) | (pcl as u16);
    }

    pub fn set_pch(&mut self, pch: u8) {
        self.pc = (self.pc & 0xFF) | ((pch as u16) << 8);
    }

    pub fn op_offset(&self) -> u16 {
        self.op_offs
    }

    pub fn get_ir(&self) -> u8 {
        match self.next_ir {
            Some(ir) if self.op_cycle == 0 => ir,
            _ => self.ir,
        }
    }
}
