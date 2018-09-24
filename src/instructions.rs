
use registers::{ Registers, Z_BIT, N_BIT, H_BIT, C_BIT };
use memory::Memory;

pub fn op_length(op: u8) -> u32 {
    const INSTRUCTION_LENGTH: [u32; 256] = [
        1, 3, 1, 1,  1, 1, 2, 1,  3, 1, 1, 1,  1, 1, 2, 1,
        1, 3, 1, 1,  1, 1, 2, 1,  2, 1, 1, 1,  1, 1, 2, 1,
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
        2, 1, 1, 1,  0, 1, 2, 1,  2, 1, 3, 1,  0, 0, 2, 1
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


// 16-bit push operation
// Flags: - - - -
pub fn push_op(reg: &mut Registers, mem: &mut Memory, value: u16) {
    let sp = reg.sp - 2;
    mem.write_u16(sp, value);
    reg.sp = sp;
}

// 16-bit pop operation
// Flags: - - - -
// Note that flags are still affected by POP AF
fn pop_op(reg: &mut Registers, mem: &Memory) -> u16 {
    let v = mem.read_u16(reg.sp);
    reg.sp += 2;
    v
}

// Bitwise AND operation
// Flags: Z 0 1 0
pub fn and_op(reg: &mut Registers, value: u8) {
    reg.a = reg.a & value;

    reg.zero = reg.a == 0;
    reg.half_carry = true;
    reg.neg = false;
    reg.carry = false;
}

// Bitwise OR operation
// Flags: Z 0 0 0
pub fn or_op(reg: &mut Registers, value: u8) {
    reg.a = reg.a | value;

    reg.zero = reg.a == 0;
    reg.neg = false;
    reg.half_carry = false;
    reg.carry = false;
}

// Bitwise XOR operation
// Flags: Z 0 0 0
pub fn xor_op(reg: &mut Registers, value: u8) {
    reg.a = reg.a ^ value;

    reg.zero = reg.a == 0;
    reg.neg = false;
    reg.half_carry = false;
    reg.carry = false;
}

// Bit test operation
// Flags: Z 0 1 -
pub fn bit_op(reg: &mut Registers, bit: u8, value: u8) {
    reg.zero = value & (1 << bit) == 0;
    reg.neg = false;
    reg.half_carry = true;
}

// Increment value operation
// Flags: Z 0 H -
pub fn inc_op(reg: &mut Registers, value: u8) -> u8 {
    let result = if value == 255 { 0 } else { value + 1 };
    let hc = ((value & 0xF) + (result & 0xF)) & 0x10 == 0x10;
    reg.set_znhc(result == 0, false, hc, value == 255);
    result
}

// Increment 16-bit value operation
// Flags: - - - -
pub fn inc16_op(value: u16) -> u16 {
    return if value == 0xFFFF { 0 } else { value + 1 };
}

// Add 8-bit value to accumulator
// Flags: Z 0 H C
pub fn add_op(reg: &mut Registers, value: u8) {
    let a32: u32 = reg.a as u32 + value as u32;
    let hc = ((reg.a & 0xF) + (value & 0xF)) & 0x10 == 0x10;
    reg.half_carry = hc;
    reg.carry = a32 > 0xFF;
    reg.zero = (a32 & 0xFF) == 0;
    reg.neg = false;
    reg.a = (a32 & 0xFF) as u8;
}

pub fn add_hl_op(reg: &mut Registers, value: u16) {
    // Add 16-bit value to HL
    // Flags: - 0 H C
    let sum: u32 = reg.hl() as u32 + value as u32;
    let hc = ((reg.hl() & 0x0FFF) + (value & 0xFFF)) & 0x1000 == 0x1000;
    reg.half_carry = hc;
    reg.carry = sum > 0xFFFF;
    reg.neg = false;
    reg.set_hl((sum & 0xFFFF) as u16);
}

pub fn adc_op(reg: &mut Registers, value: u8) {
    // ADC A, n: add sum of n and carry to A
    // Flags: Z 0 H C
    let carry: u32 = if reg.carry { 1 } else { 0 };
    let sum: u32 = (reg.a as u32) + (value as u32) + carry;
    let hc = ((reg.a as u32 & 0x0F) + ((value as u32 + carry) & 0x0F)) > 0xF;
    reg.half_carry = hc;
    reg.carry = sum > 0xFF;
    reg.a = (sum & 0xFF) as u8;
    reg.zero = reg.a == 0;
    reg.neg = false;
}

pub fn dec_op(reg: &mut Registers, value: u8) -> u8 {
    // Flags: Z 1 H -
    let dec_value = if value == 0 { 255 } else { value - 1 };
    reg.zero = dec_value == 0;
    // reg.set_carry(value == 0);
    reg.neg = true;
    reg.half_carry = (dec_value ^ 0x01 ^ value) & 0x10 == 0x10;
    dec_value
}

pub fn sub_op(reg: &mut Registers, value: u8) {
    // Flags: Z 1 H C
    reg.half_carry = reg.a & 0xF < value & 0xF;

    if reg.a >= value {
        reg.carry = false;
        reg.a = reg.a - value;
    } else {
        reg.carry = true;
        reg.a = (reg.a as u32 + 256 - value as u32) as u8;
    }

    reg.zero = reg.a == 0;
    reg.neg = true;
}

pub fn sbc_op(reg: &mut Registers, value: u8) {
    // SBC A, n: subtract sum of n and carry to A
    // Flags: Z 1 H C
    let carry: u32 = if reg.carry { 1 } else { 0 };

    let hc = (reg.a & 0xF) < ((value as u32 + carry) & 0xF) as u8;
    reg.half_carry = hc;

    let mut a: u32 = reg.a as u32;
    if a >= (value as u32) + carry {
        a = a - value as u32 - carry;
        reg.carry = false;
    } else {
        a = a + 256 - value as u32 - carry;
        reg.carry = true;
    }

    reg.a = (a & 0xFF) as u8;
    reg.zero = reg.a == 0;
    reg.neg = true;
}

pub fn cp_op(reg: &mut Registers, value: u8) {
    // Flags: Z 1 H C
    reg.zero = reg.a == value;
    reg.carry = reg.a < value;
    reg.half_carry = reg.a & 0xF < value & 0xF;
    reg.neg = true;
}

// Swap upper and lower 4 bits
// Flags: Z 0 0 0
pub fn swap_op(reg: &mut Registers, value: u8) -> u8 {
    let res = ((value >> 4) & 0x0F) | (value << 4);
    reg.set_znhc(res == 0, false, false, false);
    res
}

pub fn rst_op(reg: &mut Registers, mem: &mut Memory, address: u16) {
    let next = reg.pc + 1;
    push_op(reg, mem, next);
    reg.pc = address;
}

pub fn rrc_op(reg: &mut Registers, value: u8) -> u8 {
    let bit0 = value & 1;
    let rotated = (value >> 1) | (bit0 << 7);
    reg.set_znhc(rotated == 0, false, false, bit0 != 0);
    rotated
}

pub fn rlc_op(reg: &mut Registers, value: u8) -> u8 {
    let bit7 = value & 128;
    let rotated = (value << 1) | (bit7 >> 7);
    reg.set_znhc(rotated == 0, false, false, bit7 != 0);
    rotated
}

// RRA, RR A, RR B, ...:
// Rotates register to the right with the carry put in bit 7
// and bit 0 put into the carry
// Flags RRA: 0 0 0 C
// Flags RR A, RR B, ...: Z 0 0 C
pub fn rr_op(reg: &mut Registers, value: u8) -> u8 {
    let mut res;

    // Store bit 0
    let bit0 = value & 1;

    // Shift right
    res = value >> 1;

    // Copy carry to it 7
    if reg.carry {
        res = res | 128;
    }

    // Update flags (note that RRA should always clear Z)
    reg.set_znhc(res == 0, false, false, bit0 != 0);

    res
}

pub fn rl_op(reg: &mut Registers, value: u8) -> u8 {
    let mut t = (value as u32) << 1;

    if t & 0x100 != 0 {
        t |= 1;
        reg.carry = true;
    } else {
        reg.carry = false;
    }

    reg.neg = false;
    reg.half_carry = false;
    reg.zero = t & 0xFF == 0;

    return (t & 0xFF) as u8;
}


fn sla_op(reg: &mut Registers, value: u8) -> u8 {
    let result = (value << 1) & 0xFF;
    reg.set_znhc(result == 0, false, false, value & 128 != 0);
    result
}

fn sra_op(reg: &mut Registers, value: u8) -> u8 {
    let result = (value >> 1);
    reg.set_znhc(result == 0, false, false, value & 1 == 1);
    result
}

fn srl_op(reg: &mut Registers, value: u8) -> u8 {
    // Shift n right into Carry. MSB set to 0.
    let result = value >> 1;
    reg.set_znhc(result == 0, false, false, value & 1 != 0);
    result
}

fn daa_op(reg: &mut Registers) {
    let mut a: i32 = reg.a as i32;

    if !reg.neg {
        if reg.half_carry || (a & 0xF) > 9 {
            a += 0x06;
        }

        if reg.carry || a > 0x9F {
            a += 0x60;
        }
    } else {
        if reg.half_carry {
            a = a - 6;
            if !reg.carry {
                a = a & 0xFF;
            }
        }

        if reg.carry {
            a -= 0x60;
        }
    }

    reg.half_carry = false;
    reg.carry = a & 0x100 != 0;
    reg.a = (a & 0xFF) as u8;
    reg.zero = reg.a == 0;
}

pub fn step(reg: &mut Registers, mem: &mut Memory) -> u32 {
    let op: u8 = mem.read(reg.pc);
    let cycles: u32 = 4;

    // Set to false for operations that modify PC
    let mut inc_pc: bool = true;

    match op {
        // NOP: wait for 4 cycles
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0x00 => {}

        // SCF: Set Carry Flag
        // Length: 1
        // Cycles: 4
        // Flags: - 0 0 1
        0x37 => {
            reg.neg = false;
            reg.half_carry = false;
            reg.carry = true;
        }

        // DAA: ...
        0x27 => { daa_op(reg) }

        // LD rr, d16: load immediate (d16) into 16-bit register rr
        // Length: 3
        // Cycles: 12
        // Flags: - - - -
        0x01 => {
            reg.c = mem.read(reg.pc + 1);
            reg.b = mem.read(reg.pc + 2);
        }
        0x11 => {
            reg.e = mem.read(reg.pc + 1);
            reg.d = mem.read(reg.pc + 2);
        }
        0x21 => {
            reg.l = mem.read(reg.pc + 1);
            reg.h = mem.read(reg.pc + 2);
        }
        0x31 => {
            reg.sp = mem.read_u16(reg.pc + 1);
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
        0x04 => { let b = reg.b; reg.b = inc_op(reg, b); }
        0x0C => { let c = reg.c; reg.c = inc_op(reg, c); }
        0x14 => { let d = reg.d; reg.d = inc_op(reg, d); }
        0x1C => { let e = reg.e; reg.e = inc_op(reg, e); }
        0x24 => { let h = reg.h; reg.h = inc_op(reg, h); }
        0x2C => { let l = reg.l; reg.l = inc_op(reg, l); }
        0x3C => { let a = reg.a; reg.a = inc_op(reg, a); }

        // INC (HL): increment memory stored at HL
        // Length: 1
        // Cycles: 12
        // Flags: Z 0 H -
        0x34 => { let v = mem.read(reg.hl()); mem.write(reg.hl(), inc_op(reg, v)) }

        // INC nn: increments content of register pair nn by 1
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x03 => { let bc = inc16_op(reg.bc()); reg.set_bc(bc); }
        0x13 => { let de = inc16_op(reg.de()); reg.set_de(de); }
        0x23 => { let hl = inc16_op(reg.hl()); reg.set_hl(hl); }
        0x33 => { reg.sp = inc16_op(reg.sp); }

        // DEC n: decrement register n
        // Length: 1
        // Flags: Z 1 H -
        0x05 => { let b = reg.b; reg.b = dec_op(reg, b); }
        0x0D => { let c = reg.c; reg.c = dec_op(reg, c); }
        0x15 => { let d = reg.d; reg.d = dec_op(reg, d); }
        0x1D => { let e = reg.e; reg.e = dec_op(reg, e); }
        0x25 => { let h = reg.h; reg.h = dec_op(reg, h); }
        0x2D => { let l = reg.l; reg.l = dec_op(reg, l); }
        0x3D => { let a = reg.a; reg.a = dec_op(reg, a); }

        // DEC rr: decrement register pair rr
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x0B => { let bc = reg.bc(); reg.set_bc(if bc == 0 { 0xFFFF} else { bc - 1 }); }
        0x1B => { let de = reg.de(); reg.set_de(if de == 0 { 0xFFFF} else { de - 1 }); }
        0x2B => { let hl = reg.hl(); reg.set_hl(if hl == 0 { 0xFFFF} else { hl - 1 }); }
        0x3B => { reg.sp = if reg.sp == 0 { 0xFFFF } else { reg.sp - 1 }}

        // DEC (HL): decrement memory stored at HL
        // Length: 1
        // Cycles: 12
        // Flags: Z 1 H -
        0x35 => { let v = mem.read(reg.hl()); mem.write(reg.hl(), dec_op(reg, v)) }

        // ADD r, ADD (hl): add register r or value at (hl) to accumulator
        // Length: 1
        // Cycles: 4 (8 for ADD (hl))
        // Flags: Z 0 H C
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

        // ADD A, d8: add immediate value d8 to A
        // Length: 2
        // Cycles: 8
        // Flags: Z 0 H C
        0xC6 => { let d8 = mem.read(reg.pc + 1); add_op(reg, d8) }

        // ADC A, r: add register r + carry to A
        // Length: 1
        // Cycles: 4 (8)
        // Flags: Z 0 H C
        0x88 => { let b = reg.b; adc_op(reg, b) }
        0x89 => { let c = reg.c; adc_op(reg, c) }
        0x8A => { let d = reg.d; adc_op(reg, d) }
        0x8B => { let e = reg.e; adc_op(reg, e) }
        0x8C => { let h = reg.h; adc_op(reg, h) }
        0x8D => { let l = reg.l; adc_op(reg, l) }
        0x8E => { let v = mem.read(reg.hl()); adc_op(reg, v) }
        0x8F => { let a = reg.a; adc_op(reg, a) }

        // ADC A, d8: add immediate value + carry to A
        // Length: 2
        // Cycles: 8
        // Flags: Z 0 H C
        //0xCE => { let d8 = mem.read(reg.pc + 1); adc_op(reg, d8) }
        0xCE => {
            let value = mem.read(reg.pc + 1);

/*
 int n = programCounter.byteOperand(memory);
        int a = registers.readA();
        int result = n + a + (flags.isSet(Flags.Flag.CARRY) ? 1 : 0);

        boolean carry = result > 0xFF;
        result = result & 0xFF;
        boolean halfCarry = ((result ^ a ^ n) & 0x10) != 0;
        boolean zero = result == 0;

        registers.writeA(result);

        flags.set(Flags.Flag.ZERO, zero);
        flags.set(Flags.Flag.SUBTRACT, false);
        flags.set(Flags.Flag.HALF_CARRY, halfCarry);
        flags.set(Flags.Flag.CARRY, carry);

return 8;
*/
            let carry: u32 = if reg.carry { 1 } else { 0 };

            reg.half_carry = ((reg.a & 0x0F) + (value.wrapping_add(carry as u8) & 0x0F)) > 0xF;
            reg.carry = (reg.a as u32) + (value as u32) + carry > 0xFF;

            reg.a = reg.a.wrapping_add(value).wrapping_add(carry as u8);

            reg.zero = reg.a == 0;
            reg.neg = false;
        }

        // SBC A, r: subtract register r and carry from A
        // Length: 1
        // Cycles: 4 (8)
        // Flags: Z 1 H C
        0x98 => { let b = reg.b; sbc_op(reg, b) }
        0x99 => { let c = reg.c; sbc_op(reg, c) }
        0x9A => { let d = reg.d; sbc_op(reg, d) }
        0x9B => { let e = reg.e; sbc_op(reg, e) }
        0x9C => { let h = reg.h; sbc_op(reg, h) }
        0x9D => { let l = reg.l; sbc_op(reg, l) }
        0x9E => { let v = mem.read(reg.hl()); sbc_op(reg, v) }
        0x9F => { let a = reg.a; sbc_op(reg, a) }

        // SBC A, d8: subtract immediate value and carry from A
        0xDE => { let d8 = mem.read(reg.pc + 1); sbc_op(reg, d8) }

        // ADD HL, rr: adds value of register pair rr to HL and stores result in HL
        // Length: 1
        // Cycles: 8
        // Flags: - 0 H C
        0x09 => { let hl = reg.hl(); let bc = reg.bc(); add_hl_op(reg, bc) }
        0x19 => { let hl = reg.hl(); let de = reg.de(); add_hl_op(reg, de) }
        0x29 => { let hl = reg.hl(); add_hl_op(reg, hl) }
        0x39 => { let hl = reg.hl(); let sp = reg.sp; add_hl_op(reg, sp) }

        // ADD SP, d8: add immediate value d8 to SP
        // Length: 2
        // Cycles: 16
        // Flags: 0 0 H C
        // TODO: this is very similar to the add_hl_op. could they be combined?
        0xE8 => {
            let value = mem.read_i8(reg.pc + 1) as u16;
            let hc = ((reg.sp & 0x0F) + (value & 0x0F)) > 0x0F;

            reg.half_carry = hc;
            reg.carry = (reg.sp & 0xFF) + (value & 0xFF) > 0xFF;
            reg.zero = false;
            reg.neg = false;

            reg.sp = reg.sp.wrapping_add(value);
        }

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

        // SUB d8: subtract immediate value d8 from A
        // Length: 2
        // Cycles: 8
        // Flags: Z 1 H C
        0xD6 => { let v = mem.read(reg.pc + 1); sub_op(reg, v) }

        // AND r, AND (hl), AND d8: set A to "A AND r", or "A AND (hl)""
        // Length: 1 (2)
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
        // Length: 1 (2)
        // Cycles: 4 (8 for OR (hl))
        // Flags: Z 0 0 0
        0xB0 => { let b = reg.b; or_op(reg, b) }
        0xB1 => { let c = reg.c; or_op(reg, c) }
        0xB2 => { let d = reg.d; or_op(reg, d) }
        0xB3 => { let e = reg.e; or_op(reg, e) }
        0xB4 => { let h = reg.h; or_op(reg, h) }
        0xB5 => { let l = reg.l; or_op(reg, l) }
        0xB6 => { let v = mem.read(reg.hl()); or_op(reg, v); }
        0xB7 => { let a = reg.a; or_op(reg, a) }
        0xF6 => { let v = mem.read(reg.pc + 1); or_op(reg, v) }

        // RRCA: ...
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        // Note that rrc_op() sets Z flag, but RRCA should always clear Z flag
        0x0F => { let a = reg.a; reg.a = rrc_op(reg, a); reg.zero = false; }

        // RRA: ...
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        // Note that rr_op() sets Z flag, but RRA should always clear Z flag
        0x1F => { let a = reg.a; reg.a = rr_op(reg, a); reg.zero = false; }

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
            reg.pc = pop_op(reg, mem);
            inc_pc = false;
        }

        // RETI: set PC to 16-bit value popped from stack and enable IME
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        0xD9 => {
            reg.pc = pop_op(reg, mem);
            reg.ime = true;
            inc_pc = false;
        }

        // RET Z: set PC to 16-bit value popped from stack if Z-flag is set
        // RET C: set PC to 16-bit value popped from stack if C-flag is set
        // RET NZ: set PC to 16-bit value popped from stack if Z-flag is *not* set
        // RET NC: set PC to 16-bit value popped from stack if C-flag is *not* set
        // Length: 1
        // Cycles: 20/8
        // Flags: - - - -
        0xC8 => { if reg.zero { reg.pc = pop_op(reg, mem); inc_pc = false }}
        0xD8 => { if reg.carry { reg.pc = pop_op(reg, mem); inc_pc = false }}
        0xC0 => { if !reg.zero { reg.pc = pop_op(reg, mem); inc_pc = false }}
        0xD0 => { if !reg.carry { reg.pc = pop_op(reg, mem); inc_pc = false }}

        // CALL a16: push address of next instruction on stack
        //           and jump to address a16
        // Length: 3
        // Flags: - - - -
        0xCD => {
            let nexti = reg.pc + 3;
            push_op(reg, mem, nexti);
            reg.pc = mem.read_u16(reg.pc + 1);
            inc_pc = false;
        }

        // CALL NZ, a16: if Z-flag is not set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        0xC4 => {
            if !reg.zero {
                let nexti = reg.pc + 3;
                push_op(reg, mem, nexti);
                reg.pc = mem.read_u16(reg.pc + 1) ;
                inc_pc = false;
            }
        }

        // CALL NC, a16: if C-flag is not set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        0xD4 => {
            if !reg.carry {
                let nexti = reg.pc + 3;
                push_op(reg, mem, nexti);
                reg.pc = mem.read_u16(reg.pc + 1);
                inc_pc = false;
            }
        }

        // CALL Z, a16: if Z-flag is set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        0xCC => {
            if reg.zero {
                let nexti = reg.pc + 3;
                push_op(reg, mem, nexti);
                reg.pc = mem.read_u16(reg.pc + 1);
                inc_pc = false;
            }
        }

        // CALL C, a16: if C-flag is set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        0xDC => {
            if reg.carry {
                let nexti = reg.pc + 3;
                push_op(reg, mem, nexti);
                reg.pc = mem.read_u16(reg.pc + 1);
                inc_pc = false;
            }
        }

        // RST n: push PC and jump to one out of 8 possible addresses
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        0xC7 => { rst_op(reg, mem, 0x0000); inc_pc = false }
        0xCF => { rst_op(reg, mem, 0x0008); inc_pc = false }
        0xD7 => { rst_op(reg, mem, 0x0010); inc_pc = false }
        0xDF => { rst_op(reg, mem, 0x0018); inc_pc = false }
        0xE7 => { rst_op(reg, mem, 0x0020); inc_pc = false }
        0xEF => { rst_op(reg, mem, 0x0028); inc_pc = false }
        0xF7 => { rst_op(reg, mem, 0x0030); inc_pc = false }
        0xFF => { rst_op(reg, mem, 0x0038); inc_pc = false }

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
        0xC1 => { let v = pop_op(reg, mem); reg.set_bc(v); }
        0xD1 => { let v = pop_op(reg, mem); reg.set_de(v); }
        0xE1 => { let v = pop_op(reg, mem); reg.set_hl(v); }
        0xF1 => { let v = pop_op(reg, mem); reg.set_af(v); }

        0xE2 => {
            // LD ($FF00+C), A: put value of A in address 0xFF00 + C
            // Length: 1
            // Cycles: 8
            // Flags: - - - -
            let addr = 0xFF00 + reg.c as u16;
            let a = reg.a;
            mem.write(addr, a);
        }

        // LD A, ($FF00+C): store value at address 0xFF00 + C in A 
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0xF2 => {
            let addr = 0xFF00 + reg.c as u16;
            reg.a = mem.read(addr);
        }

        // JR d8: relative jump
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        0x18 => {
            let offs = mem.read_i8(reg.pc + 1);
            reg.pc += 2;
            inc_pc = false;

            if offs >= 0 {
                reg.pc = reg.pc.wrapping_add(offs as u16);
            } else {
                reg.pc = reg.pc.wrapping_sub(-offs as u16);
            }
        }

        // JR NZ, d8: jump d8 relative to PC if Z flag is not set
        // Length: 2
        // Cycles: 12/8
        // Flags: - - - -
        0x20 => {
            if !reg.zero {
                let offs = mem.read_i8(reg.pc + 1);
                reg.pc += 2;
                inc_pc = false;
                if offs >= 0 {
                    reg.pc = reg.pc.wrapping_add(offs as u16);
                } else {
                    reg.pc = reg.pc.wrapping_sub(-offs as u16);
                }
            }
        }

        // JR NC, d8: jump d8 relative to PC if C flag is not set
        // Length: 2
        // Cycles: 12/8
        // Flags: - - - -
        0x30 => {
            if !reg.carry {
                let offs = mem.read_i8(reg.pc + 1);
                reg.pc += 2;
                inc_pc = false;
                if offs >= 0 {
                    reg.pc = reg.pc.wrapping_add(offs as u16);
                } else {
                    reg.pc = reg.pc.wrapping_sub(-offs as u16);
                }
            }
        }

        // JR Z, d8: jump d8 relative to PC if Z is set
        // Length: 2
        // Cycles: 12/8
        // Flags: - - - -
        0x28 => {
            let offs = mem.read_i8(reg.pc + 1);
            if reg.zero {
                reg.pc += 2;
                inc_pc = false;
                if offs >= 0 {
                    reg.pc = reg.pc.wrapping_add(offs as u16);
                } else {
                    reg.pc = reg.pc.wrapping_sub(-offs as u16);
                }
            }
        }

        0x38 => {
            // JR C, d8: jump d8 relative to PC if C is set
            // Length: 2
            // Cycles: 12/8
            // Flags: - - - -
            if reg.carry {
                let offs = mem.read_i8(reg.pc + 1);
                reg.pc += 2;
                inc_pc = false;
                if offs >= 0 {
                    reg.pc = reg.pc.wrapping_add(offs as u16);
                } else {
                    reg.pc = reg.pc.wrapping_sub(-offs as u16);
                }
            }
        }

        // JP NZ, a16: jump to address a16 if Z is *not* set
        // JP Z, a16: jump to address a16 if Z is set
        // Length: 3
        // Cycles: 16/12
        // Flags: - - - -
        0xC2 => { if !reg.zero { reg.pc = mem.read_u16(reg.pc + 1); inc_pc = false }}
        0xCA => { if reg.zero { reg.pc = mem.read_u16(reg.pc + 1); inc_pc = false }}

        // JP NC, a16: jump to address a16 if C is *not* set
        // JP C, a16: jump to address a16 if C is set
        // Length: 3
        // Cycles: 16/12
        // Flags: - - - -
        0xD2 => { if !reg.carry { reg.pc = mem.read_u16(reg.pc + 1); inc_pc = false }}
        0xDA => { if reg.carry { reg.pc = mem.read_u16(reg.pc + 1); inc_pc = false }}

        // JP a16: jump to immediate address
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xC3 => { reg.pc = mem.read_u16(reg.pc + 1); inc_pc = false }

        // JP (HL): jump to address HL, or in other words: PC = HL
        // Note that this op does *not* set PC to the value stored in memory
        // at address (HL)!
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0xE9 => { reg.pc = reg.hl(); inc_pc = false; }

        0xF9 => {
            // LD SP, HL: set HL to value of SP
            // Length: 1
            // Cycles: 8
            // Flags: - - - -
            reg.sp = reg.hl();
        }

        // LD (HL-), A: put A into memory address HL, decrement HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x32 => {
            let hl = reg.hl();
            mem.write(hl, reg.a);
            reg.set_hl(hl - 1);
        }

        // XOR N: assign A xor N to A
        // Length: 1
        // Cycles: 4
        // Flags: Z 0 0 0
        0xA8 => { let b = reg.b; xor_op(reg, b); }
        0xA9 => { let c = reg.c; xor_op(reg, c); }
        0xAA => { let d = reg.d; xor_op(reg, d); }
        0xAB => { let e = reg.e; xor_op(reg, e); }
        0xAC => { let h = reg.h; xor_op(reg, h); }
        0xAD => { let l = reg.l; xor_op(reg, l); }
        0xAE => { let v = mem.read(reg.hl()); xor_op(reg, v); }
        0xAF => { let a = reg.a; xor_op(reg, a); }

        // XOR d8: assign A xor d8 to A
        // Length: 2
        // Cycles: 8
        // Flags: Z 0 0 0
        0xEE => { let v = mem.read(reg.pc + 1); xor_op(reg, v) }

        // RLA: Rotate the contents of register A to the left
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        0x17 => {
            let b0 = if reg.carry { 1 } else { 0 };
            let b8 = reg.a & 128 == 0;
            reg.a = reg.a << 1 | b0;
            reg.set_znhc(false, false, false, b8);
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

        // LD A, (HL-): load value from (HL) to A and decrement HL
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
            mem.write(addr, reg.a);
        }

        // LD (a16), SP: store SP at address (a16)
        // Length: 3
        // Cycles: 20
        // Flags: - - - -
        0x08 => {
            let addr = mem.read_u16(reg.pc + 1);
            mem.write_u16(addr, reg.sp);
        }

        // LD HL, SP+d8: load HL with value of SP + immediate value r8
        // Alt syntax: LDHL SP, d8
        // Length: 2
        // Cycles: 12
        // Flags: 0 0 H C
        0xF8 => {
            let value = mem.read_i8(reg.pc + 1) as u16;
            reg.zero = false;
            reg.neg = false;
            reg.half_carry = ((reg.sp & 0x0F) + (value & 0x0F)) > 0x0F;
            reg.carry = (reg.sp & 0xFF) + (value & 0xFF) > 0xFF;
            let hl = reg.sp.wrapping_add(value);
            reg.set_hl(hl);
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
        0xBF => { reg.set_znhc(true, true, false, false); }

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
            reg.ime = false;
        }

        0xFB => {
            // DI: Enable Interrupt Master Enable Flag
            // Length: 1
            // Cycles: 4
            // Flags: - - - -
            reg.ime = true;
        }

        0x07 => {
            // RLCA: rotate content of register A left, with carry
            // FIXME: don't we have multiple impl of this?
            let a = (reg.a as u32) << 1;
            if a > 0xFF {
                reg.a = (a & 0xFF) as u8 | 1;
                reg.set_znhc(false, false, false, true);
            } else {
                reg.a = (a & 0xFF) as u8;
                reg.set_znhc(false, false, false, false);
            }
        }

        // CPL: complement (bitwise not) register A
        // Length: 1
        // Cycles: 4
        // Flags: - 1 1 -
        0x2F => {
            reg.a = !reg.a;
            reg.neg = true;
            reg.half_carry = true;
        }

        // CCF: Flip carry flag
        // Length: 1
        // Cycles: 4
        // Flags: - 0 0 C
        0x3F => {
            reg.carry = !reg.carry;
            reg.half_carry = false;
            reg.neg = false;
        }

        // STOP 0
        // Length: 1 (not 2, see https://stackoverflow.com/questions/41353869)
        // Cycles: 4
        0x10 => {
            reg.stopped = true;
        }

        // Prefix 0xCB instructions
        0xCB => {
            let op2 = mem.read(reg.pc + 1);
            match op2 {
                // RLC n: rotate register n left
                0x00 => { let b = reg.b; reg.b = rlc_op(reg, b); }
                0x01 => { let c = reg.c; reg.c = rlc_op(reg, c); }
                0x02 => { let d = reg.d; reg.d = rlc_op(reg, d); }
                0x03 => { let e = reg.e; reg.e = rlc_op(reg, e); }
                0x04 => { let h = reg.h; reg.h = rlc_op(reg, h); }
                0x05 => { let l = reg.l; reg.l = rlc_op(reg, l); }
                0x06 => {
                    let v = mem.read(reg.hl());
                    let rot = rlc_op(reg, v);
                    mem.write(reg.hl(), rot);
                }
                0x07 => { let b = reg.b; reg.b = rlc_op(reg, b); }

                // RLC n: rotate register n right
                0x08 => { let b = reg.b; reg.b = rrc_op(reg, b); }
                0x09 => { let c = reg.c; reg.c = rrc_op(reg, c); }
                0x0A => { let d = reg.d; reg.d = rrc_op(reg, d); }
                0x0B => { let e = reg.e; reg.e = rrc_op(reg, e); }
                0x0C => { let h = reg.h; reg.h = rrc_op(reg, h); }
                0x0D => { let l = reg.l; reg.l = rrc_op(reg, l); }
                0x0E => {
                    let v = mem.read(reg.hl());
                    let rot = rrc_op(reg, v);
                    mem.write(reg.hl(), rot);
                }
                0x0F => { let b = reg.b; reg.b = rrc_op(reg, b); }

                // RL n: rotate register n left with carry flag
                0x10 => { let b = reg.b; reg.b = rl_op(reg, b); }
                0x11 => { let c = reg.c; reg.c = rl_op(reg, c); }
                0x12 => { let d = reg.d; reg.d = rl_op(reg, d); }
                0x13 => { let e = reg.e; reg.e = rl_op(reg, e); }
                0x14 => { let h = reg.h; reg.h = rl_op(reg, h); }
                0x15 => { let l = reg.l; reg.l = rl_op(reg, l); }
                0x16 => {
                    let v = mem.read(reg.hl());
                    let rot = rl_op(reg, v);
                    mem.write(reg.hl(), rot);
                }
                0x17 => { let a = reg.a; reg.a = rl_op(reg, a); }

                // RR n, rotate register n right with carry flag
                0x18 => { let b = reg.b; reg.b = rr_op(reg, b) }
                0x19 => { let c = reg.c; reg.c = rr_op(reg, c) }
                0x1A => { let d = reg.d; reg.d = rr_op(reg, d) }
                0x1B => { let e = reg.e; reg.e = rr_op(reg, e) }
                0x1C => { let h = reg.h; reg.h = rr_op(reg, h) }
                0x1D => { let l = reg.l; reg.l = rr_op(reg, l) }
                0x1E => {
                    let v = mem.read(reg.hl());
                    let rot = rr_op(reg, v);
                    mem.write(reg.hl(), rot);
                }
                0x1F => { let a = reg.a; reg.a = rr_op(reg, a) }

                // SLA r
                0x20 => { let b = reg.b; reg.b = sla_op(reg, b) }
                0x21 => { let c = reg.c; reg.c = sla_op(reg, c) }
                0x22 => { let d = reg.d; reg.d = sla_op(reg, d) }
                0x23 => { let e = reg.e; reg.e = sla_op(reg, e) }
                0x24 => { let h = reg.h; reg.h = sla_op(reg, h) }
                0x25 => { let l = reg.l; reg.l = sla_op(reg, l) }
                0x26 => {
                    let v = mem.read(reg.hl());
                    let result = sla_op(reg, v);
                    mem.write(reg.hl(), result);
                }
                0x27 => { let a = reg.b; reg.b = sla_op(reg, a) }

                // SRA r
                0x28 => { let b = reg.b; reg.b = sra_op(reg, b) }
                0x29 => { let c = reg.c; reg.c = sra_op(reg, c) }
                0x2A => { let d = reg.d; reg.d = sra_op(reg, d) }
                0x2B => { let e = reg.e; reg.e = sra_op(reg, e) }
                0x2C => { let h = reg.h; reg.h = sra_op(reg, h) }
                0x2D => { let l = reg.l; reg.l = sra_op(reg, l) }
                0x2E => {
                    let v = mem.read(reg.hl());
                    let result = sra_op(reg, v);
                    mem.write(reg.hl(), result);
                }
                0x2F => { let a = reg.b; reg.b = sra_op(reg, a) }

                // SWAP r
                0x30 => { let b = reg.b; reg.b = swap_op(reg, b) }
                0x31 => { let c = reg.c; reg.c = swap_op(reg, c) }
                0x32 => { let d = reg.d; reg.d = swap_op(reg, d) }
                0x33 => { let e = reg.e; reg.e = swap_op(reg, e) }
                0x34 => { let h = reg.h; reg.h = swap_op(reg, h) }
                0x35 => { let l = reg.l; reg.l = swap_op(reg, l) }
                0x36 => {
                    let v = mem.read(reg.hl());
                    let result = swap_op(reg, v);
                    mem.write(reg.hl(), result);
                }
                0x37 => { let a = reg.a; reg.a = swap_op(reg, a) }

                // SRL r
                0x38 => { let b = reg.b; reg.b = srl_op(reg, b) }
                0x39 => { let c = reg.c; reg.c = srl_op(reg, c) }
                0x3A => { let d = reg.d; reg.d = srl_op(reg, d) }
                0x3B => { let e = reg.e; reg.e = srl_op(reg, e) }
                0x3C => { let h = reg.h; reg.h = srl_op(reg, h) }
                0x3D => { let l = reg.l; reg.l = srl_op(reg, l) }
                0x3E => {
                    let v = mem.read(reg.hl());
                    let result = srl_op(reg, v);
                    mem.write(reg.hl(), result);
                }
                0x3F => { let a = reg.a; reg.a = srl_op(reg, a) }

                // BIT b, r: test if bit 'b' in register 'r' is set
                // Length: 2
                // Cycles: 8
                // Flags: Z 0 1 -
                0x40 => { let b = reg.b; bit_op(reg, 0, b); }
                0x41 => { let c = reg.c; bit_op(reg, 0, c); }
                0x42 => { let d = reg.d; bit_op(reg, 0, d); }
                0x43 => { let e = reg.e; bit_op(reg, 0, e); }
                0x44 => { let h = reg.h; bit_op(reg, 0, h); }
                0x45 => { let l = reg.l; bit_op(reg, 0, l); }
                0x46 => { let v = mem.read(reg.hl()); bit_op(reg, 0, v) }
                0x47 => { let a = reg.a; bit_op(reg, 0, a); }

                0x48 => { let b = reg.b; bit_op(reg, 1, b); }
                0x49 => { let c = reg.c; bit_op(reg, 1, c); }
                0x4A => { let d = reg.d; bit_op(reg, 1, d); }
                0x4B => { let e = reg.e; bit_op(reg, 1, e); }
                0x4C => { let h = reg.h; bit_op(reg, 1, h); }
                0x4D => { let l = reg.l; bit_op(reg, 1, l); }
                0x4E => { let v = mem.read(reg.hl()); bit_op(reg, 1, v) }
                0x4F => { let a = reg.a; bit_op(reg, 1, a); }

                0x50 => { let b = reg.b; bit_op(reg, 2, b); }
                0x51 => { let c = reg.c; bit_op(reg, 2, c); }
                0x52 => { let d = reg.d; bit_op(reg, 2, d); }
                0x53 => { let e = reg.e; bit_op(reg, 2, e); }
                0x54 => { let h = reg.h; bit_op(reg, 2, h); }
                0x55 => { let l = reg.l; bit_op(reg, 2, l); }
                0x56 => { let v = mem.read(reg.hl()); bit_op(reg, 2, v) }
                0x57 => { let a = reg.a; bit_op(reg, 2, a); }

                0x58 => { let b = reg.b; bit_op(reg, 3, b); }
                0x59 => { let c = reg.c; bit_op(reg, 3, c); }
                0x5A => { let d = reg.d; bit_op(reg, 3, d); }
                0x5B => { let e = reg.e; bit_op(reg, 3, e); }
                0x5C => { let h = reg.h; bit_op(reg, 3, h); }
                0x5D => { let l = reg.l; bit_op(reg, 3, l); }
                0x5E => { let v = mem.read(reg.hl()); bit_op(reg, 3, v) }
                0x5F => { let a = reg.a; bit_op(reg, 3, a); }

                0x60 => { let b = reg.b; bit_op(reg, 4, b); }
                0x61 => { let c = reg.c; bit_op(reg, 4, c); }
                0x62 => { let d = reg.d; bit_op(reg, 4, d); }
                0x63 => { let e = reg.e; bit_op(reg, 4, e); }
                0x64 => { let h = reg.h; bit_op(reg, 4, h); }
                0x65 => { let l = reg.l; bit_op(reg, 4, l); }
                0x66 => { let v = mem.read(reg.hl()); bit_op(reg, 4, v) }
                0x67 => { let a = reg.a; bit_op(reg, 4, a); }

                0x68 => { let b = reg.b; bit_op(reg, 5, b); }
                0x69 => { let c = reg.c; bit_op(reg, 5, c); }
                0x6A => { let d = reg.d; bit_op(reg, 5, d); }
                0x6B => { let e = reg.e; bit_op(reg, 5, e); }
                0x6C => { let h = reg.h; bit_op(reg, 5, h); }
                0x6D => { let l = reg.l; bit_op(reg, 5, l); }
                0x6E => { let v = mem.read(reg.hl()); bit_op(reg, 5, v) }
                0x6F => { let a = reg.a; bit_op(reg, 5, a); }

                0x70 => { let b = reg.b; bit_op(reg, 6, b); }
                0x71 => { let c = reg.c; bit_op(reg, 6, c); }
                0x72 => { let d = reg.d; bit_op(reg, 6, d); }
                0x73 => { let e = reg.e; bit_op(reg, 6, e); }
                0x74 => { let h = reg.h; bit_op(reg, 6, h); }
                0x75 => { let l = reg.l; bit_op(reg, 6, l); }
                0x76 => { let v = mem.read(reg.hl()); bit_op(reg, 6, v) }
                0x77 => { let a = reg.a; bit_op(reg, 6, a); }

                0x78 => { let b = reg.b; bit_op(reg, 7, b); }
                0x79 => { let c = reg.c; bit_op(reg, 7, c); }
                0x7A => { let d = reg.d; bit_op(reg, 7, d); }
                0x7B => { let e = reg.e; bit_op(reg, 7, e); }
                0x7C => { let h = reg.h; bit_op(reg, 7, h); }
                0x7D => { let l = reg.l; bit_op(reg, 7, l); }
                0x7E => { let v = mem.read(reg.hl()); bit_op(reg, 7, v) }
                0x7F => { let a = reg.a; bit_op(reg, 7, a); }

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
                0x86 => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v & !1); }
                0x87 => { reg.a &= !1; }

                0x88 => { reg.b &= !2; }
                0x89 => { reg.c &= !2; }
                0x8A => { reg.d &= !2; }
                0x8B => { reg.e &= !2; }
                0x8C => { reg.h &= !2; }
                0x8D => { reg.l &= !2; }
                0x8E => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v & !2); }
                0x8F => { reg.a &= !2; }

                0x90 => { reg.b &= !4; }
                0x91 => { reg.c &= !4; }
                0x92 => { reg.d &= !4; }
                0x93 => { reg.e &= !4; }
                0x94 => { reg.h &= !4; }
                0x95 => { reg.l &= !4; }
                0x96 => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v & !4); }
                0x97 => { reg.a &= !4; }

                0x98 => { reg.b &= !8; }
                0x99 => { reg.c &= !8; }
                0x9A => { reg.d &= !8; }
                0x9B => { reg.e &= !8; }
                0x9C => { reg.h &= !8; }
                0x9D => { reg.l &= !8; }
                0x9E => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v & !8); }
                0x9F => { reg.a &= !8; }

                0xA0 => { reg.b &= !16; }
                0xA1 => { reg.c &= !16; }
                0xA2 => { reg.d &= !16; }
                0xA3 => { reg.e &= !16; }
                0xA4 => { reg.h &= !16; }
                0xA5 => { reg.l &= !16; }
                0xA6 => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v & !16); }
                0xA7 => { reg.a &= !16; }

                0xA8 => { reg.b &= !32; }
                0xA9 => { reg.c &= !32; }
                0xAA => { reg.d &= !32; }
                0xAB => { reg.e &= !32; }
                0xAC => { reg.h &= !32; }
                0xAD => { reg.l &= !32; }
                0xAE => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v & !32); }
                0xAF => { reg.a &= !32; }

                0xB0 => { reg.b &= !64; }
                0xB1 => { reg.c &= !64; }
                0xB2 => { reg.d &= !64; }
                0xB3 => { reg.e &= !64; }
                0xB4 => { reg.h &= !64; }
                0xB5 => { reg.l &= !64; }
                0xB6 => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v & !64); }
                0xB7 => { reg.a &= !64; }

                0xB8 => { reg.b &= !128; }
                0xB9 => { reg.c &= !128; }
                0xBA => { reg.d &= !128; }
                0xBB => { reg.e &= !128; }
                0xBC => { reg.h &= !128; }
                0xBD => { reg.l &= !128; }
                0xBE => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v & !128); }
                0xBF => { reg.a &= !128; }

                // SET b, r: set bit b in register r
                // Length: 2
                // Cycles: 8
                // Flags: - - - -
                0xC0 => { reg.b |= 1; }
                0xC1 => { reg.c |= 1; }
                0xC2 => { reg.d |= 1; }
                0xC3 => { reg.e |= 1; }
                0xC4 => { reg.h |= 1; }
                0xC5 => { reg.l |= 1; }
                0xC6 => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v | 1); }
                0xC7 => { reg.a |= 1; }

                0xC8 => { reg.b |= 2; }
                0xC9 => { reg.c |= 2; }
                0xCA => { reg.d |= 2; }
                0xCB => { reg.e |= 2; }
                0xCC => { reg.h |= 2; }
                0xCD => { reg.l |= 2; }
                0xCE => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v | 2); }
                0xCF => { reg.a |= 2; }

                0xD0 => { reg.b |= 4; }
                0xD1 => { reg.c |= 4; }
                0xD2 => { reg.d |= 4; }
                0xD3 => { reg.e |= 4; }
                0xD4 => { reg.h |= 4; }
                0xD5 => { reg.l |= 4; }
                0xD6 => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v | 4); }
                0xD7 => { reg.a |= 4; }

                0xD8 => { reg.b |= 8; }
                0xD9 => { reg.c |= 8; }
                0xDA => { reg.d |= 8; }
                0xDB => { reg.e |= 8; }
                0xDC => { reg.h |= 8; }
                0xDD => { reg.l |= 8; }
                0xDE => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v | 8); }
                0xDF => { reg.a |= 8; }

                0xE0 => { reg.b |= 16; }
                0xE1 => { reg.c |= 16; }
                0xE2 => { reg.d |= 16; }
                0xE3 => { reg.e |= 16; }
                0xE4 => { reg.h |= 16; }
                0xE5 => { reg.l |= 16; }
                0xE6 => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v | 16); }
                0xE7 => { reg.a |= 16; }

                0xE8 => { reg.b |= 32; }
                0xE9 => { reg.c |= 32; }
                0xEA => { reg.d |= 32; }
                0xEB => { reg.e |= 32; }
                0xEC => { reg.h |= 32; }
                0xED => { reg.l |= 32; }
                0xEE => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v | 32); }
                0xEF => { reg.a |= 32; }

                0xF0 => { reg.b |= 64; }
                0xF1 => { reg.c |= 64; }
                0xF2 => { reg.d |= 64; }
                0xF3 => { reg.e |= 64; }
                0xF4 => { reg.h |= 64; }
                0xF5 => { reg.l |= 64; }
                0xF6 => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v | 64); }
                0xF7 => { reg.a |= 64; }

                0xF8 => { reg.b |= 128; }
                0xF9 => { reg.c |= 128; }
                0xFA => { reg.d |= 128; }
                0xFB => { reg.e |= 128; }
                0xFC => { reg.h |= 128; }
                0xFD => { reg.l |= 128; }
                0xFE => { let hl = reg.hl(); let v = mem.read(hl); mem.write(hl, v | 128); }
                0xFF => { reg.a |= 128; }

                _ => {
                    panic!("Unsupported opcode at 0x{:04X}: 0x{:02X}{:02X}", reg.pc, op, op2);
                }
            }
        }

        _ => {
            panic!("Unsupported opcode at 0x{:04X}: 0x{:02X}", reg.pc, op);
        }
    }

    if inc_pc {
        reg.pc += op_length(op) as u16
    }

    return cycles
}

#[cfg(test)]
mod tests {
    use instructions::*;
    use debug::*;
    
    #[test]
    fn test_op_0x38_add_sp_immediate() {
        let mut reg = Registers::new();
        let mut mem = Memory::new();
        reg.pc = 0x1000;
        reg.sp = 0x2000;
        mem.mem[reg.pc as usize] = 0xE8;
        mem.mem[(reg.pc + 1) as usize] = 1 as u8;
        step(&mut reg, &mut mem);
        print_registers(&reg);
        assert!(reg.sp == 0x2001);
    }

    #[test]
    fn test_op_0xCE_add_sp_immediate() {
        let mut reg = Registers::new();
        let mut mem = Memory::new();
        reg.a = 100;
        reg.carry = true;
        reg.pc = 0x1000;
        reg.sp = 0x2000;
        mem.mem[reg.pc as usize] = 0xCE;
        mem.mem[(reg.pc + 1) as usize] = 10 as u8;
        step(&mut reg, &mut mem);
        print_registers(&reg);
        assert!(reg.a == 111);

        reg.carry = false;
        mem.mem[reg.pc as usize] = 0xCE;
        mem.mem[(reg.pc + 1) as usize] = (0 as u8).wrapping_sub(35);
        step(&mut reg, &mut mem);
        print_registers(&reg);
        assert!(reg.a == 111 - 35);
    }
}