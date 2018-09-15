
use registers::{ Registers, Z_BIT, N_BIT, H_BIT, C_BIT };
use memory::Memory;

pub fn op_length(op: u8) -> u32 {
    const INSTRUCTION_LENGTH: [u32; 256] = [
        1, 3, 1, 1,  1, 1, 2, 1,  3, 1, 1, 1,  1, 1, 2, 1,
        2, 3, 1, 1,  1, 1, 2, 1,  2, 1, 1, 1,  1, 1, 2, 1,
        2, 3, 1, 1,  1, 1, 2, 1,  2, 1, 1, 1,  1, 1, 2, 1,
        2, 3, 1, 1,  1, 1, 2, 1,  2, 1, 1, 1,  1, 1, 2, 1,

        1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,
        1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,
        1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,
        1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,

        1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,
        1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,
        1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,
        1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,  1, 1, 1, 1,

        1, 1, 3, 3,  3, 1, 2, 1,  1, 1, 3, 1,  3, 3, 2, 1,
        1, 1, 3, 0,  3, 1, 2, 1,  1, 1, 3, 0,  3, 0, 2, 1,
        2, 1, 1, 0,  0, 1, 2, 1,  2, 1, 3, 0,  0, 0, 2, 1,
        2, 1, 2, 1,  0, 1, 2, 1,  2, 1, 3, 1,  0, 0, 2, 1
    ];

    if op == 0xCB {
        // All prefix 0xCB opcodes have same length
        return 2;
    }

    let len = INSTRUCTION_LENGTH[op as usize];

    if len == 0 {
        panic!("length unknown for instructions with op code 0x{:02X}", op);
    }

    return len;
}


fn push_op(reg: &mut Registers, mem: &mut Memory, value: u16) {
    let sp = reg.sp - 2;
    mem.write(sp + 1, (value >> 8) as u8);
    mem.write(sp, (value & 0xFF) as u8);
    reg.sp = sp;
}

fn pop_op(reg: &mut Registers, mem: &Memory) -> u16 {
    let lo = mem.read(reg.sp);
    let hi = mem.read(reg.sp + 1);
    reg.sp = reg.sp + 2;
    return (((hi as u16) << 8) | lo as u16) as u16;
}

pub fn and_op(reg: &mut Registers, value: u8) {
    reg.a = reg.a & value;
    if reg.a == 0 {
        reg.f |= Z_BIT | H_BIT;
        reg.f &= !(N_BIT | C_BIT);
    } else {
        reg.f |= H_BIT;
        reg.f &= !(Z_BIT | N_BIT | C_BIT);
    }
}

pub fn or_op(reg: &mut Registers, value: u8) {
    reg.a = reg.a | value;
    if reg.a == 0 {
        reg.f |= Z_BIT;
        reg.f &= !(N_BIT | H_BIT | C_BIT);
    } else {
        reg.f &= !(Z_BIT | N_BIT | H_BIT | C_BIT);
    }
}

pub fn xor_op(reg: &mut Registers, a: u8, value: u8) -> u8 {
    // Flags: Z 0 0 0
    let res = a ^ value;
    reg.f &= !(Z_BIT | N_BIT | H_BIT | C_BIT);
    if res == 0 {
        reg.f |= Z_BIT;
    }
    res
}

pub fn bit_op(reg: &mut Registers, bit: u8, value: u8) {
    // Test if bit in register is set
    // Flags: Z 0 1 -
    if value & (1 << bit) == 0 {
        reg.f &= !N_BIT;
        reg.f |= Z_BIT | H_BIT;
    } else {
        reg.f &= !(Z_BIT | N_BIT);
        reg.f |= H_BIT;
    }
}

pub fn inc_op(reg: &mut Registers, value: u8) -> u8 {
    // Flags: Z 0 H -
    let new_value = if value == 255 { 0 } else { value + 1 };

    if new_value == 0 {
        reg.f |= Z_BIT;
    } else {
        reg.f &= !Z_BIT;
    }

    reg.f &= !N_BIT;

    if value < 255 && ((value & 0xF) + 1) & 0x10 != 0 {
        reg.f |= H_BIT;
    } else {
        reg.f &= !H_BIT;
    }

    new_value
}

pub fn inc16_op(value: u16) -> u16 {
    return if value == 0xFFFF { 0 } else { value + 1 };
}

pub fn add_op(reg: &mut Registers, value: u8) {
    let a32: u32 = reg.a as u32 + value as u32;
    reg.f &= !N_BIT;
    reg.set_carry(a32 > 0xFF);
    let a32 = a32 & 0xFF;
    reg.a = a32 as u8;
    reg.set_z_flag(a32 == 0);
}

pub fn add_hl_op(reg: &mut Registers, value: u16) {
    // Flags: - 0 H C
    let hl32: u32 = reg.hl() as u32 + value as u32;
    reg.f &= !N_BIT;
    reg.set_carry(hl32 > 0xFFFF);
    let hl32 = hl32 & 0xFFFF;
    reg.set_hl(hl32 as u16);
}

pub fn dec_op(reg: &mut Registers, value: u8) -> u8 {
    // Flags: Z 1 H -
    let new_value = if value == 0 { 255 } else { value - 1 };

    if new_value == 0 {
        reg.f |= Z_BIT | N_BIT;
    } else {
        reg.f &= !Z_BIT;
        reg.f |= N_BIT;
    }

    // FIXME: handle half-carry flag
    new_value
}

pub fn sub_op(reg: &mut Registers, value: u8) {
    // Flags: Z 1 H C
    if reg.a >= value {
        reg.a = reg.a - value;
        reg.f &= !C_BIT;
    } else {
        reg.a = (reg.a as u32 + 256 - value as u32) as u8;
        reg.f |= C_BIT;
    }

    if reg.a == 0 {
        reg.f |= Z_BIT;
    } else {
        reg.f &= !Z_BIT;
    }

    reg.f |= N_BIT;
}

pub fn cp_op(reg: &mut Registers, value: u8) {
    // Flags: Z 1 H C
    let a = reg.a;
    reg.set_z_flag(a == value);
    reg.set_carry(a < value);
    reg.f |= N_BIT;
}

pub fn swap_op(reg: &mut Registers, value: u8) -> u8 {
    let res = ((value >> 4) & 0x0F) | (value << 4);
    reg.f &= !(Z_BIT | N_BIT | H_BIT | C_BIT);
    if res == 0 { reg.f |= Z_BIT }
    res
}

pub fn rst_op(reg: &mut Registers, mem: &mut Memory, address: u16) {
    let next = reg.pc + 1;
    push_op(reg, mem, next);

    // Jump to the address. Compensate for the length
    // of the current instruction.
    reg.pc = address - 1;
}

pub fn rr_op(reg: &mut Registers, value: u8) -> u8 {
    // RRA, RR r
    let mut res;

    if value & 1 == 0 {
        if reg.f & C_BIT == 0 {
            reg.f &= !(Z_BIT | N_BIT | H_BIT | C_BIT);
            res = value >> 1;
        } else {
            reg.f &= !(Z_BIT | N_BIT | H_BIT | C_BIT);
            res = (value >> 1) | 128;
        }
    } else {
        if reg.c & C_BIT == 0 {
            reg.f &= !(Z_BIT | N_BIT | H_BIT);
            res = value >> 1;
        } else {
            reg.f &= !(Z_BIT | N_BIT | H_BIT);
            res = value >> 1 | 128;
        }
        reg.f |= C_BIT;
    }

    res
}

pub fn rl_op(reg: &mut Registers, value: u8) -> u8 {
    let mut t = (value as u32) << 1;

    if t & 0x100 != 0 {
        t |= 1;
        reg.f |= C_BIT;
    } else {
        reg.f &= !C_BIT;
    }

    reg.f &= !(N_BIT | H_BIT);

    if t & 0xFF == 0 {
        reg.f |= Z_BIT;
    } else {
        reg.f &= !Z_BIT;
    }

    return (t & 0xFF) as u8;

    // Rotate left with carry flag
    // Flags: Z 0 0 C
    let mut t = (value as u32) << 1;

    if reg.f & C_BIT != 0 {
        t |= 1;
    }

    if t & 256 == 0 {
        reg.f &= !C_BIT;
    } else {
        reg.f |= C_BIT;
    }

    if t == 0 {
        reg.f |= Z_BIT;
    } else {
        reg.f &= Z_BIT;
    }

    reg.f &= !(N_BIT | H_BIT);

    (t & 0xFF) as u8
}

pub fn step(reg: &mut Registers, mem: &mut Memory) -> u32 {
    let pc = reg.pc;
    let op: u8 = mem.read(pc);
    let length = op_length(op);
    let cycles: u32 = 4;

    match op {
        // NOP: wait for 4 cycles
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0x00 => {}

        0x01 => {
            // LD BC, d16: load immediate (d16) into BC
            // Length: 3
            // Cycles: 12
            // Flags: - - - -
            reg.c = mem.read(reg.pc + 1);
            reg.b = mem.read(reg.pc + 2);
        }

        // LD (rr), A: stores the contents of register A in the memory specified by register pair BC or DE.
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x02 => { mem.write(reg.bc(), reg.a) }
        0x12 => { mem.write(reg.de(), reg.a) }

        // LD A, (nn): loads value stored in memory at address nn (immediate)
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xFA => { let addr = mem.read_u16(reg.pc + 1); reg.a = mem.read(addr) }

        // INC n: increment register n
        // Length: 1
        // Cycles: 4
        // Flags: Z 0 H -
        0x04 => {
            // INC B
            let b = reg.b;
            reg.b = inc_op(reg, b);
        }
        0x0C => {
            // INC C
            let c = reg.c;
            reg.c = inc_op(reg, c);
        }
        0x14 => {
            // INC D
            let d = reg.d;
            reg.d = inc_op(reg, d);
        }
        0x1C => {
            // INC E
            let e = reg.e;
            reg.e = inc_op(reg, e);
        }
        0x24 => {
            // INC H
            let h = reg.h;
            reg.h = inc_op(reg, h);
        }
        0x2C => {
            // INC L
            let l = reg.l;
            reg.l = inc_op(reg, l);
        }
        0x3C => {
            // INC A
            let a = reg.a;
            reg.a = inc_op(reg, a);
        }

        // INC (HL): increment memory stored at HL
        // Length: 1
        // Cycles: 12
        // Flags: Z 0 H -
        0x34 => { let v = mem.read(reg.hl()); mem.write(reg.hl(), inc_op(reg, v)) }

        // INC nn: increments content of register pair nn by 1
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x03 => {
            // INC BC
            let bc = reg.bc();
            let bc = inc16_op(bc);
            reg.set_bc(bc);
        }
        0x13 => {
            // INC DE
            let de = reg.de();
            let de = inc16_op(de);
            reg.set_de(de);
        }
        0x23 => {
            // INC HL
            let hl = reg.hl();
            let hl = inc16_op(hl);
            reg.set_hl(hl);
        }
        0x33 => {
            // INC SP
            let sp = reg.sp;
            reg.sp = inc16_op(sp);
        }

        // DEC n: decrement register n
        // Length: 1
        // Flags: Z 1 H -
        0x05 => {
            // DEC B
            let b = reg.b;
            reg.b = dec_op(reg, b);
        }
        0x0D => {
            // DEC C
            let c = reg.c;
            reg.c = dec_op(reg, c);
        }
        0x15 => {
            // DEC D
            let d = reg.d;
            reg.d = dec_op(reg, d);
        }
        0x1D => {
            // DEC E
            let e = reg.e;
            reg.e = dec_op(reg, e);
        }
        0x25 => {
            // DEC H
            let h = reg.h;
            reg.h = dec_op(reg, h);
        }
        0x2D => {
            // DEC L
            let l = reg.l;
            reg.l = dec_op(reg, l);
        }
        0x3D => {
            // DEC A
            let a = reg.a;
            reg.a = dec_op(reg, a);
        }

        // DEC rr: decrement register pair rr
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x0B => { let bc = reg.bc(); reg.set_bc(bc - 1); }
        0x1B => { let de = reg.de(); reg.set_de(de - 1); }
        0x2B => { let hl = reg.hl(); reg.set_hl(hl - 1); }
        0x3B => { reg.sp = reg.sp - 1; }

        // DEC (HL): decrement memory stored at HL
        // Length: 1
        // Cycles: 12
        // Flags: Z 1 H -
        0x35 => { let v = mem.read(reg.hl()); mem.write(reg.hl(), dec_op(reg, v)) }

        // ADD r, ADD (hl): add register r or value at (hl) to accumulator
        // Length: 1
        // Cycles: 4 (8 for ADD (hl))
        // Flags: Z 1 H C
        0x80 => { let b = reg.b; add_op(reg, b) }
        0x81 => { let c = reg.c; add_op(reg, c) }
        0x82 => { let d = reg.d; add_op(reg, d) }
        0x83 => { let e = reg.e; add_op(reg, e) }
        0x84 => { let h = reg.h; add_op(reg, h) }
        0x85 => { let l = reg.l; add_op(reg, l) }
        0x86 => {
            let hl = reg.hl();
            add_op(reg, mem.read(hl));
        }
        0x87 => { let a = reg.a; add_op(reg, a) }

        // ADD HL, rr: adds value of register pair rr to HL and stores result in HL
        // Length: 1
        // Cycles: 8
        // Flags: - 0 H C
        0x09 => { let hl = reg.hl(); let bc = reg.bc(); add_hl_op(reg, bc) }
        0x19 => { let hl = reg.hl(); let de = reg.de(); add_hl_op(reg, de) }
        0x29 => { let hl = reg.hl(); add_hl_op(reg, hl) }
        0x39 => { let hl = reg.hl(); let sp = reg.sp; add_hl_op(reg, sp) }

        // SUB r, SUB (hl): subtract register r or value at (hl) from accumulator
        // Length: 1
        // Cycles: 4 (8 for SUB (hl))
        // Flags: Z 1 H C
        0x90 => { let b = reg.b; sub_op(reg, b) }
        0x91 => { let c = reg.c; sub_op(reg, c) }
        0x92 => { let d = reg.d; sub_op(reg, d) }
        0x93 => { let e = reg.e; sub_op(reg, e) }
        0x94 => { let h = reg.h; sub_op(reg, h) }
        0x95 => { let l = reg.l; sub_op(reg, l) }
        0x96 => {
            let hl = reg.hl();
            sub_op(reg, mem.read(hl));
        }
        0x97 => { let a = reg.a; sub_op(reg, a) }

        // AND r, AND (hl), AND d8: set A to "A AND r", or "A AND (hl)""
        // Length: 1
        // Cycles: 4 (8 for (hl) and d8)
        // Flags: Z 0 1 0
        0xA0 => { let b = reg.b; and_op(reg, b) }
        0xA1 => { let c = reg.c; and_op(reg, c) }
        0xA2 => { let d = reg.d; and_op(reg, d) }
        0xA3 => { let e = reg.e; and_op(reg, e) }
        0xA4 => { let h = reg.h; and_op(reg, h) }
        0xA5 => { let l = reg.l; and_op(reg, l) }
        0xA6 => {
            let hl = reg.hl();
            and_op(reg, mem.read(hl));
        }
        0xA7 => { let a = reg.a; and_op(reg, a) }
        0xE6 => { let v = mem.read(reg.pc + 1); and_op(reg, v) }

        // OR r, OR (hl): set A to "A OR r", or "A OR (hl)""
        // Length: 1
        // Cycles: 4 (8 for OR (hl))
        // Flags: Z 0 0 0
        0xB0 => { let b = reg.b; or_op(reg, b) }
        0xB1 => { let c = reg.c; or_op(reg, c) }
        0xB2 => { let d = reg.d; or_op(reg, d) }
        0xB3 => { let e = reg.e; or_op(reg, e) }
        0xB4 => { let h = reg.h; or_op(reg, h) }
        0xB5 => { let l = reg.l; or_op(reg, l) }
        0xB6 => {
            let hl = reg.hl();
            or_op(reg, mem.read(hl));
        }
        0xB7 => { let a = reg.a; or_op(reg, a) }
        0xF6 => { let v = mem.read(reg.pc + 1); or_op(reg, v) }

        // RRA: ...
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        0x2F => { let a = reg.a; reg.a = rr_op(reg, a); }

        // LD n, d: load immediate into register n
        // Length: 2
        // Flags: - - - -
        0x06 => { reg.b = mem.read(reg.pc + 1) }
        0x0E => { reg.c = mem.read(reg.pc + 1) }
        0x16 => { reg.d = mem.read(reg.pc + 1) }
        0x1E => { reg.e = mem.read(reg.pc + 1) }
        0x26 => { reg.h = mem.read(reg.pc + 1) }
        0x2E => { reg.l = mem.read(reg.pc + 1) }
        0x3E => { reg.a = mem.read(reg.pc + 1) }

        // LD n, m: load value of register m into register n
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0x7F => {}                 // LD A,A
        0x78 => { reg.a = reg.b }  // LD A,B
        0x79 => { reg.a = reg.c }  // LD A,C
        0x7A => { reg.a = reg.d }  // LD A,D
        0x7B => { reg.a = reg.e }  // LD A,E
        0x7C => { reg.a = reg.h }  // LD A,H
        0x7D => { reg.a = reg.l }  // LD A,L

        0x47 => { reg.b = reg.a }  // LD B,A
        0x40 => {}                 // LD B,B
        0x41 => { reg.b = reg.c }  // LD B,C
        0x42 => { reg.b = reg.d }  // LD B,D
        0x43 => { reg.b = reg.e }  // LD B,E
        0x44 => { reg.b = reg.h }  // LD B,H
        0x45 => { reg.b = reg.l }  // LD B,L

        0x4F => { reg.c = reg.a }  // LD C,A
        0x48 => { reg.c = reg.b }  // LD C,B
        0x49 => {}                 // LD C,C
        0x4A => { reg.c = reg.d }  // LD C,D
        0x4B => { reg.c = reg.e }  // LD C,E
        0x4C => { reg.c = reg.h }  // LD C,H
        0x4D => { reg.c = reg.l }  // LD C,L

        0x57 => { reg.d = reg.a }  // LD D,A
        0x50 => { reg.d = reg.b }  // LD D,B
        0x51 => { reg.d = reg.c }  // LD D,C
        0x52 => {}                 // LD D,D
        0x53 => { reg.d = reg.e }  // LD D,E
        0x54 => { reg.d = reg.h }  // LD D,H
        0x55 => { reg.d = reg.l }  // LD D,L

        0x5F => { reg.e = reg.a }  // LD E,A
        0x58 => { reg.e = reg.b }  // LD E,B
        0x59 => { reg.e = reg.c }  // LD E,C
        0x5A => { reg.e = reg.d }  // LD E,D
        0x5B => {}                 // LD E,E
        0x5C => { reg.e = reg.h }  // LD E,H
        0x5D => { reg.e = reg.l }  // LD E,L

        0x67 => { reg.h = reg.a }  // LD H,A
        0x60 => { reg.h = reg.b }  // LD H,B
        0x61 => { reg.h = reg.c }  // LD H,C
        0x62 => { reg.h = reg.d }  // LD H,D
        0x63 => { reg.h = reg.e }  // LD H,E
        0x64 => {}                 // LD H,H
        0x65 => { reg.h = reg.l }  // LD H,L

        0x6F => { reg.l = reg.a }  // LD L,A
        0x68 => { reg.l = reg.b }  // LD L,B
        0x69 => { reg.l = reg.c }  // LD L,C
        0x6A => { reg.l = reg.d }  // LD L,D
        0x6B => { reg.l = reg.e }  // LD L,E
        0x6C => { reg.l = reg.h }  // LD L,H
        0x6D => {}                 // LD L,L

        // LD n, (hl): store value at (hl) in register n
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x46 => { reg.b = mem.read(reg.hl()) }
        0x4E => { reg.c = mem.read(reg.hl()) }
        0x56 => { reg.d = mem.read(reg.hl()) }
        0x5E => { reg.e = mem.read(reg.hl()) }
        0x66 => { reg.h = mem.read(reg.hl()) }
        0x6E => { reg.l = mem.read(reg.hl()) }
        0x7E => { reg.a = mem.read(reg.hl()) }

        // LD n, (mm): load value from memory into register n
        // Length: 1
        // Flags: - - - -
        0x0A => { reg.a = mem.read(reg.bc()) }
        0x1A => { reg.a = mem.read(reg.de()) }

        // LD ($FF00+n), A: Put A into memory address $FF00+n
        // Length: 2
        // Flags: - - - -
        0xE0 => {
            let n = mem.read(reg.pc + 1);
            let a = reg.a;
            mem.write(0xFF00 + n as u16, a);
        }

        // LD A, ($FF00+n): read from memory $FF00+n to register A
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        0xF0 => {
            let n = mem.read(reg.pc + 1);
            reg.a = mem.read(0xFF00 + n as u16);
        }

        // LD (HL), n: store register value to memory at address HL
        // Length: 1
        // Flags: - - - -
        0x70 => { mem.write(reg.hl(), reg.b) }
        0x71 => { mem.write(reg.hl(), reg.c) }
        0x72 => { mem.write(reg.hl(), reg.d) }
        0x73 => { mem.write(reg.hl(), reg.e) }
        0x74 => { mem.write(reg.hl(), reg.h) }
        0x75 => { mem.write(reg.hl(), reg.l) }
        0x77 => { mem.write(reg.hl(), reg.a) }

        // RET: set PC to 16-bit value popped from stack
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        0xC9 => {
            // Compensate for length of current instruction
            reg.pc = pop_op(reg, &mem) - 1;
        }

        // RET Z: set PC to 16-bit value popped from stack if Z-flag is set
        // Length: 1
        // Cycles: 20/8
        // Flags: - - - -
        0xC8 => { if reg.z_flag() { reg.pc = pop_op(reg, &mem) - 1 }}
        0xD8 => { if reg.c_flag() { reg.pc = pop_op(reg, &mem) - 1 }}

        // CALL a16: push address of next instruction on stack
        //           and jump to address a16
        // Length: 3
        // Flags: - - - -
        0xCD => {
            let nexti = reg.pc + 3;
            push_op(reg, mem, nexti);

            // Set PC to target address. Compensate
            // for the length of the current instruction.
            reg.pc = mem.read_u16(reg.pc + 1) - 3;
        }

        // RST n: push PC and jump to one out of 8 possible addresses
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        0xC7 => { rst_op(reg, mem, 0x0000) }
        0xCF => { rst_op(reg, mem, 0x0008) }
        0xD7 => { rst_op(reg, mem, 0x0010) }
        0xDF => { rst_op(reg, mem, 0x0018) }
        0xE7 => { rst_op(reg, mem, 0x0020) }
        0xEF => { rst_op(reg, mem, 0x0028) }
        0xF7 => { rst_op(reg, mem, 0x0030) }
        0xFF => { rst_op(reg, mem, 0x0038) }

        // PUSH nn: push 16-bit register nn to stack
        // Length: 1
        // Flags: - - - -
        0xC5 => { let bc = reg.bc(); push_op(reg, mem, bc); }
        0xD5 => { let de = reg.de(); push_op(reg, mem, de); }
        0xE5 => { let hl = reg.hl(); push_op(reg, mem, hl); }
        0xF5 => { let af = reg.af(); push_op(reg, mem, af); }

        // POP nn: pop value from stack and store in 16-bit register nn
        // Length: 1
        // Cycles: 12
        // Flags: - - - -
        0xC1 => { let v = pop_op(reg, &mem); reg.set_bc(v); }
        0xD1 => { let v = pop_op(reg, &mem); reg.set_de(v); }
        0xE1 => { let v = pop_op(reg, &mem); reg.set_hl(v); }
        0xF1 => { let v = pop_op(reg, &mem); reg.set_af(v); }

        0xE2 => {
            // LD ($FF00+C), A: put value of A in address 0xFF00 + C
            // Length: 2
            // Cycles: 8
            // Flags: - - - -
            let addr = 0xFF00 + reg.c as u16;
            let a = reg.a;
            mem.write(addr, a);
        }

        0x11 => {
            // LD DE, d16: load immediate (d16) into DE
            // Length: 3
            // Cycles: 12
            // Flags: - - - -
            reg.e = mem.read(reg.pc + 1);
            reg.d = mem.read(reg.pc + 2);
        }

        0x18 => {
            // JR d8: relative jump
            // Length: 2
            // Cycles: 12
            let offs = mem.read_i8(reg.pc + 1);
            if offs >= 0 {
                reg.pc = reg.pc.wrapping_add(offs as u16);
            } else {
                reg.pc = reg.pc.wrapping_sub(-offs as u16);
            }
        }

        0x20 => {
            // JR NZ, d8: jump d8 relative to PC if Z is reset
            // Length: 2
            // Cycles: 12/8
            // Flags: - - - -
            let offs = mem.read_i8(reg.pc + 1);
            if !reg.z_flag() {
                if offs >= 0 {
                    reg.pc = reg.pc.wrapping_add(offs as u16);
                } else {
                    reg.pc = reg.pc.wrapping_sub(-offs as u16);
                }
            }
        }

        0x28 => {
            // JR Z, d8: jump d8 relative to PC if Z is set
            // Length: 2
            // Cycles: 12/8
            // Flags: - - - -
            let offs = mem.read_i8(reg.pc + 1);
            if reg.z_flag() {
                if offs >= 0 {
                    reg.pc = reg.pc.wrapping_add(offs as u16);
                } else {
                    reg.pc = reg.pc.wrapping_sub(-offs as u16);
                }
            }
        }

        // JP Z, a16: jump to address a16 if Z is set
        // Length: 3
        // Cycles: 16/12
        // Flags: - - - -
        0xCA => { if reg.z_flag() { reg.pc = mem.read_u16(reg.pc + 1) - 3 }}

        // JP Z, a16: jump to address a16 if C is set
        // Length: 3
        // Cycles: 16/12
        // Flags: - - - -
        0xDA => { if reg.c_flag() { reg.pc = mem.read_u16(reg.pc + 1) - 3 }}

        // JP a16: jump to immediate address
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xC3 => { reg.pc = mem.read_u16(reg.pc + 1) - 3; }

        // LD (HL): jump to address HL, or in other words: PC = HL
        // Note that this op does *not* set PC to the value stored in memory
        // at address (HL)!
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0xE9 => {
            // Set PC to HL and compensate for length of current instruction
            reg.pc = reg.hl() - 1
        }

        0x21 => {
            // LD HL, d16: load immediate (d16) into HL
            // Length: 3
            // Cycles: 12
            // Flags: - - - -
            reg.l = mem.read(reg.pc + 1);
            reg.h = mem.read(reg.pc + 2);
        }

        0x31 => {
            // LD SP, d16: load immediate (d16) into SP
            // Length: 3
            // Cycles: 12
            // Flags: - - - -
            reg.sp = mem.read_u16(reg.pc + 1);
        }

        0x32 => {
            // LD (HL-), A: put A into memory address HL, decrement HL
            // Length: 1
            // Cycles: 8
            // Flags: - - - -
            let hl: u16 = ((reg.h as u16) << 8) | (reg.l as u16);
            let a = reg.a;
            mem.write(hl, a);
            let hl = hl - 1;
            reg.h = (hl >> 8) as u8;
            reg.l = (hl & 0xFF) as u8;
        }

        // XOR N: assign A xor N to A
        // Length: 1
        // Cycles: 4
        // Flags: Z 0 0 0
        0xA8 => { let a = reg.a; let b = reg.b; reg.a = xor_op(reg, a, b); }
        0xA9 => { let a = reg.a; let c = reg.c; reg.a = xor_op(reg, a, c); }
        0xAA => { let a = reg.a; let d = reg.d; reg.a = xor_op(reg, a, d); }
        0xAB => { let a = reg.a; let e = reg.e; reg.a = xor_op(reg, a, e); }
        0xAC => { let a = reg.a; let h = reg.h; reg.a = xor_op(reg, a, h); }
        0xAD => { let a = reg.a; let l = reg.l; reg.a = xor_op(reg, a, l); }
        0xAF => { let a = reg.a; reg.a = xor_op(reg, a, a); }

        // RLA: Rotate the contents of register A to the left
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        0x17 => {
            let b0 = if reg.f & C_BIT == 0 { 0 } else { 1 };
            let b8 = reg.a & 128 == 0;
            reg.a = reg.a << 1 | b0;
            reg.set_carry(b8);
            reg.f &= !(Z_BIT | H_BIT | N_BIT);
        }

        // LD (HL+), A: store value of A at (HL) and increment HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        // Alt mnemonic 1: LD (HLI), A
        // Alt mnemonic 2: LDI (HL), A
        0x22 => {
            let hl = reg.hl();
            mem.write(hl, reg.a);
            reg.set_hl(hl + 1);
        }

        // LD (HL), d8: store immediate value at (HL)
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        0x36 => {
            let v = mem.read(reg.pc + 1);
            mem.write(reg.hl(), v);
        }

        // LD A, (HL+): load value from (HL) to A and increment HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x2A => {
            let hl = reg.hl();
            reg.a = mem.read(hl);
            reg.set_hl(hl + 1);
        }

        // LD A, (HL+): load value from (HL) to A and decrement HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x3A => {
            let hl = reg.hl();
            reg.a = mem.read(hl);
            reg.set_hl(hl - 1);
        }

        // LD (a16), A: store value of A at address a16
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xEA => {
            let addr = mem.read_u16(reg.pc + 1);
            let val = reg.a;
            mem.write(addr, val);
        }

        // CP r, CP (hl): Compare r (or value at (hl)) with A. Same as SUB but throws away the result
        // Length: 1
        // Cycles: 4 (8 for "CP (hl)")
        // Flags: Z 1 H C
        0xB8 => { let b = reg.b; cp_op(reg, b); }
        0xB9 => { let c = reg.c; cp_op(reg, c); }
        0xBA => { let d = reg.d; cp_op(reg, d); }
        0xBB => { let e = reg.e; cp_op(reg, e); }
        0xBC => { let h = reg.h; cp_op(reg, h); }
        0xBD => { let l = reg.l; cp_op(reg, l); }
        0xBE => { let v = mem.read(reg.hl()); cp_op(reg, v); }
        0xBF => { reg.set_z_flag(true); reg.set_carry(false); reg.f |= N_BIT; }

        // CP u8: Compare A with immediate
        // Length: 2
        // Cycles: 8
        // Flags: Z 1 H C
        0xFE => { let v = mem.read(reg.pc + 1); cp_op(reg, v); }

        0xF3 => {
            // DI: Disable Interrupt Master Enable Flag, prohibits maskable interrupts
            // Length: 1
            // Cycles: 4
            // Flags: - - - -
            println!("Note: DI not implemented!");
        }

        0xFB => {
            // DI: Enable Interrupt Master Enable Flag
            // Length: 1
            // Cycles: 4
            // Flags: - - - -
            println!("Note: EI not implemented!");
        }

        // Prefix 0xCB instructions
        0xCB => {
            let op2 = mem.read(reg.pc + 1);
            match op2 {
                // RL n: rotate register n left with carry flag
                0x11 => {
                    let c = reg.c;
                    reg.c = rl_op(reg, c);
                }

                // SWAP r
                0x30 => { let b = reg.b; reg.b = swap_op(reg, b) }
                0x31 => { let c = reg.c; reg.c = swap_op(reg, c) }
                0x32 => { let d = reg.d; reg.d = swap_op(reg, d) }
                0x33 => { let e = reg.e; reg.e = swap_op(reg, e) }
                0x34 => { let h = reg.h; reg.h = swap_op(reg, h) }
                0x35 => { let l = reg.l; reg.l = swap_op(reg, l) }
                0x36 => { let v = mem.read(reg.hl()); mem.write(reg.hl(), swap_op(reg, v)) }
                0x37 => { let a = reg.a; reg.a = swap_op(reg, a) }

                0x7C => {
                    // BIT 7, H: test if bit 7 in register H is set
                    // Length: 2
                    // Cycles: 8
                    // Flags: Z 0 1 -
                    let h = reg.h;
                    bit_op(reg, 7, h);
                }

                // RES b, r: reset bit b in register r
                // Length: 2
                // Cycles: 8
                // Flags: - - - -
                0x80 => { reg.b &= !1; }
                0x81 => { reg.c &= !1; }
                0x82 => { reg.d &= !1; }
                0x83 => { reg.e &= !1; }
                0x84 => { reg.h &= !1; }
                0x85 => { reg.l &= !1; }
                // 0x86 => { /* FIXME */ }
                0x87 => { reg.a &= !1; }

                0x88 => { reg.b &= !2; }
                0x89 => { reg.c &= !2; }
                0x8A => { reg.d &= !2; }
                0x8B => { reg.e &= !2; }
                0x8C => { reg.h &= !2; }
                0x8D => { reg.l &= !2; }
                // 0x8E => { /* FIXME */ }
                0x8F => { reg.a &= !2; }

                0x90 => { reg.b &= !4; }
                0x91 => { reg.c &= !4; }
                0x92 => { reg.d &= !4; }
                0x93 => { reg.e &= !4; }
                0x94 => { reg.h &= !4; }
                0x95 => { reg.l &= !4; }
                // 0x96 => { /* FIXME */ }
                0x97 => { reg.a &= !4; }

                0x98 => { reg.b &= !8; }
                0x99 => { reg.c &= !8; }
                0x9A => { reg.d &= !8; }
                0x9B => { reg.e &= !8; }
                0x9C => { reg.h &= !8; }
                0x9D => { reg.l &= !8; }
                // 0x9E => { /* FIXME */ }
                0x9F => { reg.a &= !8; }

                0xA0 => { reg.b &= !16; }
                0xA1 => { reg.c &= !16; }
                0xA2 => { reg.d &= !16; }
                0xA3 => { reg.e &= !16; }
                0xA4 => { reg.h &= !16; }
                0xA5 => { reg.l &= !16; }
                // 0xA6 => { /* FIXME */ }
                0xA7 => { reg.a &= !16; }

                0xA8 => { reg.b &= !32; }
                0xA9 => { reg.c &= !32; }
                0xAA => { reg.d &= !32; }
                0xAB => { reg.e &= !32; }
                0xAC => { reg.h &= !32; }
                0xAD => { reg.l &= !32; }
                // 0xAE => { /* FIXME */ }
                0xAF => { reg.a &= !32; }

                0xB0 => { reg.b &= !64; }
                0xB1 => { reg.c &= !64; }
                0xB2 => { reg.d &= !64; }
                0xB3 => { reg.e &= !64; }
                0xB4 => { reg.h &= !64; }
                0xB5 => { reg.l &= !64; }
                // 0xB6 => { /* FIXME */ }
                0xB7 => { reg.a &= !64; }

                0xB8 => { reg.b &= !128; }
                0xB9 => { reg.c &= !128; }
                0xBA => { reg.d &= !128; }
                0xBB => { reg.e &= !128; }
                0xBC => { reg.h &= !128; }
                0xBD => { reg.l &= !128; }
                // 0xBE => { /* FIXME */ }
                0xBF => { reg.a &= !128; }

                _ => {
                    panic!("Unsupported opcode at 0x{:04X}: 0x{:02X}{:02X}", reg.pc, op, op2);
                }
            }
        }

        _ => {
            panic!("Unsupported opcode at 0x{:04X}: 0x{:02X}", reg.pc, op);
        }
    }

    reg.pc += length as u16;
    return cycles
}
