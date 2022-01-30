use crate::mmu::{IE_REG, IF_REG, MMU};
use crate::registers::Registers;

pub fn _op_cycles(op: u8) -> u32 {
    const OP_CYCLES: [u32; 256] = [
        1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, 0, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1,
        2, 1, 2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, 2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2,
        1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1,
        1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 2, 2, 2, 2, 2, 0, 2,
        1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1,
        2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1,
        1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2, 3, 3, 4, 3, 4, 2, 4, 2, 4, 3, 0, 3, 6, 2, 4, 2, 3,
        3, 0, 3, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4, 3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4,
        3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4,
    ];

    return OP_CYCLES[op as usize] * 4;
}

pub fn op_length(op: u8) -> Option<usize> {
    const INSTRUCTION_LENGTH: [usize; 256] = [
        1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1, 1, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1,
        2, 1, 2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, 2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1,
        1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, 1, 1,
        3, 0, 3, 1, 2, 1, 1, 1, 3, 0, 3, 0, 2, 1, 2, 1, 1, 0, 0, 1, 2, 1, 2, 1, 3, 0, 0, 0, 2, 1,
        2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 3, 1, 0, 0, 2, 1,
    ];

    if op == 0xCB {
        // All prefix 0xCB opcodes have same length
        return Some(2);
    }

    let len = INSTRUCTION_LENGTH[op as usize];

    if len == 0 {
        println!("length unknown for instruction with op code 0x{:02X}", op);
        return None;
    }

    return Some(len);
}

// 16-bit push operation
// Flags: - - - -
pub fn push_op(mmu: &mut MMU, value: u16) {
    // For correct emulation the high byte is pushed first, then the low byte
    let sp = mmu.reg.sp.wrapping_sub(1);
    mmu.write(sp, ((value >> 8) & 0xFF) as u8);
    let sp = sp.wrapping_sub(1);
    mmu.write(sp, (value & 0xFF) as u8);

    mmu.reg.sp = sp;
}

// 16-bit pop operation
// Flags: - - - -
// Cycles: 12
// Note that flags are still affected by POP AF
fn pop_op(mmu: &mut MMU) -> u16 {
    let sp = mmu.reg.sp;
    let lo = mmu.read(sp);
    let sp = sp.wrapping_add(1);
    let hi = mmu.read(sp);
    mmu.reg.sp = sp.wrapping_add(1);
    ((hi as u16) << 8) | (lo as u16)
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
    let result = value.wrapping_add(1);
    reg.zero = result == 0;
    reg.neg = false;
    reg.half_carry = (value & 0xF) == 0xF;
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
    let carry: u8 = if reg.carry { 1 } else { 0 };
    let result = reg.a.wrapping_add(value).wrapping_add(carry);
    reg.zero = result == 0;
    reg.neg = false;
    reg.half_carry = (reg.a & 0x0F) + (value & 0x0F) + carry > 0xF;
    reg.carry = (reg.a as u16 + value as u16 + carry as u16) > 0xFF;
    reg.a = result
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
    let carry: u8 = if reg.carry { 1 } else { 0 };
    let result = reg.a.wrapping_sub(value).wrapping_sub(carry);
    reg.zero = result == 0;
    reg.neg = true;
    reg.half_carry = reg.a & 0xF < (value & 0xF) + carry;
    reg.carry = (reg.a as u16) < (value as u16 + carry as u16);
    reg.a = result;
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

pub fn rst_op(mmu: &mut MMU, address: u16) {
    let pc = mmu.reg.pc;
    mmu.tick(4);
    push_op(mmu, pc);
    mmu.reg.pc = address;
}

pub fn rrc_op(reg: &mut Registers, value: u8) -> u8 {
    let bit0 = value & 1;
    let rotated = (value >> 1) | (bit0 << 7);
    reg.set_znhc(rotated == 0, false, false, bit0 != 0);
    rotated
}

pub fn rlc_op(reg: &mut Registers, value: u8) -> u8 {
    let bit7 = value & 128;
    let rotated = value.rotate_left(1);
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
    let carry_bit: u8 = if reg.carry { 1 } else { 0 };
    let rotated = value << 1 | carry_bit;
    reg.set_znhc(rotated == 0, false, false, value & 128 != 0);
    rotated
}

fn sla_op(reg: &mut Registers, value: u8) -> u8 {
    let result = (value << 1) & 0xFF;
    reg.set_znhc(result == 0, false, false, value & 128 != 0);
    result
}

fn sra_op(reg: &mut Registers, value: u8) -> u8 {
    let result = value >> 1 | (value & 128);
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
    // This implementation is heavily inspired by `mooneye-gb`
    // https://github.com/Gekkio/mooneye-gb/blob/master/core/src/cpu/mod.rs
    let mut carry = false;

    if !reg.neg {
        if reg.carry || reg.a > 0x99 {
            reg.a = reg.a.wrapping_add(0x60);
            carry = true;
        }

        if reg.half_carry || reg.a & 0xF > 0x9 {
            reg.a = reg.a.wrapping_add(0x6);
        }
    } else {
        if reg.carry {
            carry = true;

            if reg.half_carry {
                reg.a = reg.a.wrapping_add(0x9A);
            } else {
                reg.a = reg.a.wrapping_add(0xA0);
            }
        } else {
            if reg.half_carry {
                reg.a = reg.a.wrapping_add(0xFA);
            }
        }
    }

    reg.zero = reg.a == 0;
    reg.carry = carry;
    reg.half_carry = false;
}

pub fn step(mmu: &mut MMU) {
    let op: u8 = mmu.fetch();

    match op {
        // NOP: no operation
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0x00 => {}

        // HALT: ...
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0x76 => {
            if mmu.reg.ime != 0 {
                mmu.reg.halted = true;
            } else {
                let if_reg = mmu.direct_read(IF_REG);
                let ie_reg = mmu.direct_read(IE_REG);
                if if_reg & ie_reg & 0x1F == 0 {
                    mmu.reg.halted = true;
                } else {
                    // FIXME: Emulate HALT bug: next op is executed twice
                    // if a single byte op. If a multi byte op, it's even worse.
                    println!("Ooops! HALT bug is NOT emulated!")
                }
            }
        }

        // SCF: Set Carry Flag
        // Length: 1
        // Cycles: 4
        // Flags: - 0 0 1
        0x37 => {
            mmu.reg.neg = false;
            mmu.reg.half_carry = false;
            mmu.reg.carry = true;
        }

        // DAA: ...
        // Length: 1
        // Cycles: 4
        // Flags: Z - 0 C
        0x27 => daa_op(&mut mmu.reg),

        // LD rr, d16: load immediate (d16) into 16-bit register rr
        // Length: 3
        // Cycles: 12
        // Flags: - - - -
        0x01 => {
            mmu.reg.c = mmu.fetch();
            mmu.reg.b = mmu.fetch();
        }
        0x11 => {
            mmu.reg.e = mmu.fetch();
            mmu.reg.d = mmu.fetch();
        }
        0x21 => {
            mmu.reg.l = mmu.fetch();
            mmu.reg.h = mmu.fetch();
        }
        0x31 => {
            mmu.reg.sp = mmu.fetch_u16();
        }

        // LD (rr), A: stores the contents of register A in the memory specified by register pair BC or DE.
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x02 => {
            let bc = mmu.reg.bc();
            let a = mmu.reg.a;
            mmu.write(bc, a);
        }
        0x12 => {
            let de = mmu.reg.de();
            let a = mmu.reg.a;
            mmu.write(de, a);
        }

        // LD A, (nn): loads value stored in memory at address nn (immediate)
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xFA => {
            let addr = mmu.fetch_u16();
            mmu.reg.a = mmu.read(addr);
        }

        // INC n: increment register n
        // Length: 1
        // Cycles: 4
        // Flags: Z 0 H -
        0x04 => {
            let b = mmu.reg.b;
            mmu.reg.b = inc_op(&mut mmu.reg, b);
        }
        0x0C => {
            let c = mmu.reg.c;
            mmu.reg.c = inc_op(&mut mmu.reg, c);
        }
        0x14 => {
            let d = mmu.reg.d;
            mmu.reg.d = inc_op(&mut mmu.reg, d);
        }
        0x1C => {
            let e = mmu.reg.e;
            mmu.reg.e = inc_op(&mut mmu.reg, e);
        }
        0x24 => {
            let h = mmu.reg.h;
            mmu.reg.h = inc_op(&mut mmu.reg, h);
        }
        0x2C => {
            let l = mmu.reg.l;
            mmu.reg.l = inc_op(&mut mmu.reg, l);
        }
        0x3C => {
            let a = mmu.reg.a;
            mmu.reg.a = inc_op(&mut mmu.reg, a);
        }

        // INC (HL): increment memory stored at HL
        // Length: 1
        // Cycles: 12
        // Flags: Z 0 H -
        0x34 => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            let v = inc_op(&mut mmu.reg, v);
            mmu.write(hl, v);
        }

        // INC nn: increments content of register pair nn by 1
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        // TODO: placement of mmu.tick()?
        0x03 => {
            let bc = inc16_op(mmu.reg.bc());
            mmu.reg.set_bc(bc);
            mmu.tick(4);
        }
        0x13 => {
            let de = inc16_op(mmu.reg.de());
            mmu.reg.set_de(de);
            mmu.tick(4);
        }
        0x23 => {
            let hl = inc16_op(mmu.reg.hl());
            mmu.reg.set_hl(hl);
            mmu.tick(4);
        }
        0x33 => {
            mmu.reg.sp = inc16_op(mmu.reg.sp);
            mmu.tick(4);
        }

        // DEC n: decrement register n
        // Length: 1
        // Cycles: 4
        // Flags: Z 1 H -
        0x05 => {
            let b = mmu.reg.b;
            mmu.reg.b = dec_op(&mut mmu.reg, b);
        }
        0x0D => {
            let c = mmu.reg.c;
            mmu.reg.c = dec_op(&mut mmu.reg, c);
        }
        0x15 => {
            let d = mmu.reg.d;
            mmu.reg.d = dec_op(&mut mmu.reg, d);
        }
        0x1D => {
            let e = mmu.reg.e;
            mmu.reg.e = dec_op(&mut mmu.reg, e);
        }
        0x25 => {
            let h = mmu.reg.h;
            mmu.reg.h = dec_op(&mut mmu.reg, h);
        }
        0x2D => {
            let l = mmu.reg.l;
            mmu.reg.l = dec_op(&mut mmu.reg, l);
        }
        0x3D => {
            let a = mmu.reg.a;
            mmu.reg.a = dec_op(&mut mmu.reg, a);
        }

        // DEC rr: decrement register pair rr
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        // TODO: placement of mmu.tick()?
        0x0B => {
            let bc = mmu.reg.bc();
            mmu.reg.set_bc(bc.wrapping_sub(1));
            mmu.tick(4);
        }
        0x1B => {
            let de = mmu.reg.de();
            mmu.reg.set_de(de.wrapping_sub(1));
            mmu.tick(4);
        }
        0x2B => {
            let hl = mmu.reg.hl();
            mmu.reg.set_hl(hl.wrapping_sub(1));
            mmu.tick(4);
        }
        0x3B => {
            mmu.reg.sp = mmu.reg.sp.wrapping_sub(1);
            mmu.tick(4);
        }

        // DEC (HL): decrement memory stored at HL
        // Length: 1
        // Cycles: 12
        // Flags: Z 1 H -
        0x35 => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            let v = dec_op(&mut mmu.reg, v);
            mmu.write(hl, v);
        }

        // ADD r, ADD (hl): add register r or value at (hl) to accumulator
        // Length: 1
        // Cycles: 4 (8 for op 0x86)
        // Flags: Z 0 H C
        0x80 => {
            let b = mmu.reg.b;
            add_op(&mut mmu.reg, b);
        }
        0x81 => {
            let c = mmu.reg.c;
            add_op(&mut mmu.reg, c);
        }
        0x82 => {
            let d = mmu.reg.d;
            add_op(&mut mmu.reg, d);
        }
        0x83 => {
            let e = mmu.reg.e;
            add_op(&mut mmu.reg, e);
        }
        0x84 => {
            let h = mmu.reg.h;
            add_op(&mut mmu.reg, h);
        }
        0x85 => {
            let l = mmu.reg.l;
            add_op(&mut mmu.reg, l);
        }
        0x86 => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            add_op(&mut mmu.reg, v);
        }
        0x87 => {
            let a = mmu.reg.a;
            add_op(&mut mmu.reg, a)
        }

        // ADD A, d8: add immediate value d8 to A
        // Length: 2
        // Cycles: 8
        // Flags: Z 0 H C
        0xC6 => {
            let v = mmu.fetch();
            add_op(&mut mmu.reg, v);
        }

        // ADC A, r: add register r + carry to A
        // Length: 1
        // Cycles: 4 (8 for op 0x8E)
        // Flags: Z 0 H C
        0x88 => {
            let b = mmu.reg.b;
            adc_op(&mut mmu.reg, b);
        }
        0x89 => {
            let c = mmu.reg.c;
            adc_op(&mut mmu.reg, c);
        }
        0x8A => {
            let d = mmu.reg.d;
            adc_op(&mut mmu.reg, d);
        }
        0x8B => {
            let e = mmu.reg.e;
            adc_op(&mut mmu.reg, e);
        }
        0x8C => {
            let h = mmu.reg.h;
            adc_op(&mut mmu.reg, h);
        }
        0x8D => {
            let l = mmu.reg.l;
            adc_op(&mut mmu.reg, l);
        }
        0x8E => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            adc_op(&mut mmu.reg, v);
        }
        0x8F => {
            let a = mmu.reg.a;
            adc_op(&mut mmu.reg, a);
        }

        // ADC A, d8: add immediate value + carry to A
        // Length: 2
        // Cycles: 8
        // Flags: Z 0 H C
        //0xCE => { let d8 = mem.read(reg.pc + 1); adc_op(reg, d8) }
        0xCE => {
            let v = mmu.fetch();
            adc_op(&mut mmu.reg, v);
        }

        // SBC A, r: subtract register r and carry from A
        // Length: 1
        // Cycles: 4 (8)
        // Flags: Z 1 H C
        0x98 => {
            let b = mmu.reg.b;
            sbc_op(&mut mmu.reg, b)
        }
        0x99 => {
            let c = mmu.reg.c;
            sbc_op(&mut mmu.reg, c)
        }
        0x9A => {
            let d = mmu.reg.d;
            sbc_op(&mut mmu.reg, d)
        }
        0x9B => {
            let e = mmu.reg.e;
            sbc_op(&mut mmu.reg, e)
        }
        0x9C => {
            let h = mmu.reg.h;
            sbc_op(&mut mmu.reg, h)
        }
        0x9D => {
            let l = mmu.reg.l;
            sbc_op(&mut mmu.reg, l)
        }
        0x9E => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            sbc_op(&mut mmu.reg, v)
        }
        0x9F => {
            let a = mmu.reg.a;
            sbc_op(&mut mmu.reg, a)
        }

        // SBC A, d8: subtract immediate value and carry from A
        0xDE => {
            let d8 = mmu.fetch();
            sbc_op(&mut mmu.reg, d8)
        }

        // ADD HL, rr: adds value of register pair rr to HL and stores result in HL
        // Length: 1
        // Cycles: 8
        // Flags: - 0 H C
        // TODO: placement of mmu.tick()?
        0x09 => {
            let bc = mmu.reg.bc();
            add_hl_op(&mut mmu.reg, bc);
            mmu.tick(4);
        }
        0x19 => {
            let de = mmu.reg.de();
            add_hl_op(&mut mmu.reg, de);
            mmu.tick(4);
        }
        0x29 => {
            let hl = mmu.reg.hl();
            add_hl_op(&mut mmu.reg, hl);
            mmu.tick(4);
        }
        0x39 => {
            let sp = mmu.reg.sp;
            add_hl_op(&mut mmu.reg, sp);
            mmu.tick(4);
        }

        // ADD SP, d8: add immediate value d8 to SP
        // Length: 2
        // Cycles: 16
        // Flags: 0 0 H C
        // TODO: this is very similar to the add_hl_op. could they be combined?
        0xE8 => {
            // let value = mem.read_i8(reg.pc + 1) as u16;
            let value = mmu.fetch() as i8 as u16;

            let hc = ((mmu.reg.sp & 0x0F) + (value & 0x0F)) > 0x0F;

            mmu.reg.half_carry = hc;
            mmu.reg.carry = (mmu.reg.sp & 0xFF) + (value & 0xFF) > 0xFF;
            mmu.reg.zero = false;
            mmu.reg.neg = false;

            mmu.reg.sp = mmu.reg.sp.wrapping_add(value);
            mmu.tick(8);
        }

        // SUB r, SUB (hl): subtract register r or value at (hl) from accumulator
        // Length: 1
        // Cycles: 4 (8 for op 0x96)
        // Flags: Z 1 H C
        0x90 => {
            let b = mmu.reg.b;
            sub_op(&mut mmu.reg, b)
        }
        0x91 => {
            let c = mmu.reg.c;
            sub_op(&mut mmu.reg, c)
        }
        0x92 => {
            let d = mmu.reg.d;
            sub_op(&mut mmu.reg, d)
        }
        0x93 => {
            let e = mmu.reg.e;
            sub_op(&mut mmu.reg, e)
        }
        0x94 => {
            let h = mmu.reg.h;
            sub_op(&mut mmu.reg, h)
        }
        0x95 => {
            let l = mmu.reg.l;
            sub_op(&mut mmu.reg, l)
        }
        0x96 => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            sub_op(&mut mmu.reg, v);
        }
        0x97 => {
            let a = mmu.reg.a;
            sub_op(&mut mmu.reg, a)
        }

        // SUB d8: subtract immediate value d8 from A
        // Length: 2
        // Cycles: 8
        // Flags: Z 1 H C
        0xD6 => {
            let v = mmu.fetch();
            sub_op(&mut mmu.reg, v);
        }

        // AND r, AND (hl), AND d8: set A to "A AND r", or "A AND (hl)""
        // Length: 1 (2 for op 0xE6)
        // Cycles: 4 (8 for op 0xA6 and 0xE6)
        // Flags: Z 0 1 0
        0xA0 => {
            let b = mmu.reg.b;
            and_op(&mut mmu.reg, b)
        }
        0xA1 => {
            let c = mmu.reg.c;
            and_op(&mut mmu.reg, c)
        }
        0xA2 => {
            let d = mmu.reg.d;
            and_op(&mut mmu.reg, d)
        }
        0xA3 => {
            let e = mmu.reg.e;
            and_op(&mut mmu.reg, e)
        }
        0xA4 => {
            let h = mmu.reg.h;
            and_op(&mut mmu.reg, h)
        }
        0xA5 => {
            let l = mmu.reg.l;
            and_op(&mut mmu.reg, l)
        }
        0xA6 => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            and_op(&mut mmu.reg, v);
        }
        0xA7 => {
            let a = mmu.reg.a;
            and_op(&mut mmu.reg, a)
        }
        0xE6 => {
            let v = mmu.fetch();
            and_op(&mut mmu.reg, v)
        }

        // OR r, OR (hl): set A to "A OR r", or "A OR (hl)""
        // Length: 1 (2 for 0xF6)
        // Cycles: 4 (8 for op 0xB6 and 0xF6)
        // Flags: Z 0 0 0
        0xB0 => {
            let b = mmu.reg.b;
            or_op(&mut mmu.reg, b)
        }
        0xB1 => {
            let c = mmu.reg.c;
            or_op(&mut mmu.reg, c)
        }
        0xB2 => {
            let d = mmu.reg.d;
            or_op(&mut mmu.reg, d)
        }
        0xB3 => {
            let e = mmu.reg.e;
            or_op(&mut mmu.reg, e)
        }
        0xB4 => {
            let h = mmu.reg.h;
            or_op(&mut mmu.reg, h)
        }
        0xB5 => {
            let l = mmu.reg.l;
            or_op(&mut mmu.reg, l)
        }
        0xB6 => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            or_op(&mut mmu.reg, v);
        }
        0xB7 => {
            let a = mmu.reg.a;
            or_op(&mut mmu.reg, a)
        }
        0xF6 => {
            let v = mmu.fetch();
            or_op(&mut mmu.reg, v)
        }

        // RRCA: ...
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        // Note that rrc_op() sets Z flag, but RRCA should always clear Z flag
        0x0F => {
            let a = mmu.reg.a;
            mmu.reg.a = rrc_op(&mut mmu.reg, a);
            mmu.reg.zero = false;
        }

        // RRA: ...
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        // Note that rr_op() sets Z flag, but RRA should always clear Z flag
        0x1F => {
            let a = mmu.reg.a;
            mmu.reg.a = rr_op(&mut mmu.reg, a);
            mmu.reg.zero = false;
        }

        // LD n, d: load immediate into register n
        // Length: 2
        // Cycles: 8
        // Flags: - - - -
        0x06 => mmu.reg.b = mmu.fetch(),
        0x0E => mmu.reg.c = mmu.fetch(),
        0x16 => mmu.reg.d = mmu.fetch(),
        0x1E => mmu.reg.e = mmu.fetch(),
        0x26 => mmu.reg.h = mmu.fetch(),
        0x2E => mmu.reg.l = mmu.fetch(),
        0x3E => mmu.reg.a = mmu.fetch(),

        // LD n, m: load value of register m into register n
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0x7F => {}                     // LD A,A
        0x78 => mmu.reg.a = mmu.reg.b, // LD A,B
        0x79 => mmu.reg.a = mmu.reg.c, // LD A,C
        0x7A => mmu.reg.a = mmu.reg.d, // LD A,D
        0x7B => mmu.reg.a = mmu.reg.e, // LD A,E
        0x7C => mmu.reg.a = mmu.reg.h, // LD A,H
        0x7D => mmu.reg.a = mmu.reg.l, // LD A,L

        0x47 => mmu.reg.b = mmu.reg.a, // LD B,A
        0x40 => {}                     // LD B,B
        0x41 => mmu.reg.b = mmu.reg.c, // LD B,C
        0x42 => mmu.reg.b = mmu.reg.d, // LD B,D
        0x43 => mmu.reg.b = mmu.reg.e, // LD B,E
        0x44 => mmu.reg.b = mmu.reg.h, // LD B,H
        0x45 => mmu.reg.b = mmu.reg.l, // LD B,L

        0x4F => mmu.reg.c = mmu.reg.a, // LD C,A
        0x48 => mmu.reg.c = mmu.reg.b, // LD C,B
        0x49 => {}                     // LD C,C
        0x4A => mmu.reg.c = mmu.reg.d, // LD C,D
        0x4B => mmu.reg.c = mmu.reg.e, // LD C,E
        0x4C => mmu.reg.c = mmu.reg.h, // LD C,H
        0x4D => mmu.reg.c = mmu.reg.l, // LD C,L

        0x57 => mmu.reg.d = mmu.reg.a, // LD D,A
        0x50 => mmu.reg.d = mmu.reg.b, // LD D,B
        0x51 => mmu.reg.d = mmu.reg.c, // LD D,C
        0x52 => {}                     // LD D,D
        0x53 => mmu.reg.d = mmu.reg.e, // LD D,E
        0x54 => mmu.reg.d = mmu.reg.h, // LD D,H
        0x55 => mmu.reg.d = mmu.reg.l, // LD D,L

        0x5F => mmu.reg.e = mmu.reg.a, // LD E,A
        0x58 => mmu.reg.e = mmu.reg.b, // LD E,B
        0x59 => mmu.reg.e = mmu.reg.c, // LD E,C
        0x5A => mmu.reg.e = mmu.reg.d, // LD E,D
        0x5B => {}                     // LD E,E
        0x5C => mmu.reg.e = mmu.reg.h, // LD E,H
        0x5D => mmu.reg.e = mmu.reg.l, // LD E,L

        0x67 => mmu.reg.h = mmu.reg.a, // LD H,A
        0x60 => mmu.reg.h = mmu.reg.b, // LD H,B
        0x61 => mmu.reg.h = mmu.reg.c, // LD H,C
        0x62 => mmu.reg.h = mmu.reg.d, // LD H,D
        0x63 => mmu.reg.h = mmu.reg.e, // LD H,E
        0x64 => {}                     // LD H,H
        0x65 => mmu.reg.h = mmu.reg.l, // LD H,L

        0x6F => mmu.reg.l = mmu.reg.a, // LD L,A
        0x68 => mmu.reg.l = mmu.reg.b, // LD L,B
        0x69 => mmu.reg.l = mmu.reg.c, // LD L,C
        0x6A => mmu.reg.l = mmu.reg.d, // LD L,D
        0x6B => mmu.reg.l = mmu.reg.e, // LD L,E
        0x6C => mmu.reg.l = mmu.reg.h, // LD L,H
        0x6D => {}                     // LD L,L

        // LD n, (hl): store value at (hl) in register n
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x46 => {
            let hl = mmu.reg.hl();
            mmu.reg.b = mmu.read(hl)
        }
        0x4E => {
            let hl = mmu.reg.hl();
            mmu.reg.c = mmu.read(hl)
        }
        0x56 => {
            let hl = mmu.reg.hl();
            mmu.reg.d = mmu.read(hl)
        }
        0x5E => {
            let hl = mmu.reg.hl();
            mmu.reg.e = mmu.read(hl)
        }
        0x66 => {
            let hl = mmu.reg.hl();
            mmu.reg.h = mmu.read(hl)
        }
        0x6E => {
            let hl = mmu.reg.hl();
            mmu.reg.l = mmu.read(hl)
        }
        0x7E => {
            let hl = mmu.reg.hl();
            mmu.reg.a = mmu.read(hl)
        }

        // LD n, (mm): load value from memory into register n
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x0A => {
            let bc = mmu.reg.bc();
            mmu.reg.a = mmu.read(bc)
        }
        0x1A => {
            let de = mmu.reg.de();
            mmu.reg.a = mmu.read(de)
        }

        // LD ($FF00+n), A: Put A into memory address $FF00+n
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        0xE0 => {
            let n = mmu.fetch();
            let a = mmu.reg.a;
            mmu.write(0xFF00 + n as u16, a);
        }

        // LD A, ($FF00+n): read from memory $FF00+n to register A
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        0xF0 => {
            let n = mmu.fetch();
            mmu.reg.a = mmu.read(0xFF00 + n as u16);
        }

        // LD (HL), n: store register value to memory at address HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x70 => {
            let hl = mmu.reg.hl();
            let b = mmu.reg.b;
            mmu.write(hl, b)
        }
        0x71 => {
            let hl = mmu.reg.hl();
            let c = mmu.reg.c;
            mmu.write(hl, c)
        }
        0x72 => {
            let hl = mmu.reg.hl();
            let d = mmu.reg.d;
            mmu.write(hl, d)
        }
        0x73 => {
            let hl = mmu.reg.hl();
            let e = mmu.reg.e;
            mmu.write(hl, e)
        }
        0x74 => {
            let hl = mmu.reg.hl();
            let h = mmu.reg.h;
            mmu.write(hl, h)
        }
        0x75 => {
            let hl = mmu.reg.hl();
            let l = mmu.reg.l;
            mmu.write(hl, l)
        }
        0x77 => {
            let hl = mmu.reg.hl();
            let a = mmu.reg.a;
            mmu.write(hl, a)
        }

        // RET: set PC to 16-bit value popped from stack
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        // TODO: placement of mmu.tick()?
        // TODO: why is RET 16 cycles when POP BC is 12 cycles?
        0xC9 => {
            mmu.reg.pc = pop_op(mmu);
            mmu.tick(4);
        }

        // RETI: set PC to 16-bit value popped from stack and enable IME
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        // This function is really EI followed by RET
        0xD9 => {
            mmu.reg.ime = 1;
            mmu.reg.pc = pop_op(mmu);
            mmu.tick(4);
            mmu.reg.ime = 2;
        }

        // RET Z: set PC to 16-bit value popped from stack if Z-flag is set
        // RET C: set PC to 16-bit value popped from stack if C-flag is set
        // RET NZ: set PC to 16-bit value popped from stack if Z-flag is *not* set
        // RET NC: set PC to 16-bit value popped from stack if C-flag is *not* set
        // Length: 1
        // Cycles: 20/8
        // Flags: - - - -
        // TODO: placement of mmu.tick()?
        0xC8 => {
            mmu.tick(4);
            if mmu.reg.zero {
                mmu.reg.pc = pop_op(mmu);
                mmu.tick(4);
            }
        }
        0xD8 => {
            mmu.tick(4);
            if mmu.reg.carry {
                mmu.reg.pc = pop_op(mmu);
                mmu.tick(4);
            }
        }
        0xC0 => {
            mmu.tick(4);
            if !mmu.reg.zero {
                mmu.reg.pc = pop_op(mmu);
                mmu.tick(4);
            }
        }
        0xD0 => {
            mmu.tick(4);
            if !mmu.reg.carry {
                mmu.reg.pc = pop_op(mmu);
                mmu.tick(4);
            }
        }

        // CALL a16: push address of next instruction on stack
        //           and jump to address a16
        // Length: 3
        // Cycles: 24
        // Flags: - - - -
        // TODO: placement of mmu.tick()?
        0xCD => {
            let to = mmu.fetch_u16();
            let pc = mmu.reg.pc;
            mmu.tick(4);
            push_op(mmu, pc);
            mmu.reg.pc = to;
        }

        // CALL NZ, a16: if Z-flag is not set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        // TODO: placement of mmu.tick()?
        0xC4 => {
            let to = mmu.fetch_u16();
            if !mmu.reg.zero {
                let pc = mmu.reg.pc;
                mmu.tick(4);
                push_op(mmu, pc);
                mmu.reg.pc = to;
            }
        }

        // CALL NC, a16: if C-flag is not set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        // TODO: placement of mmu.tick()?
        0xD4 => {
            let to = mmu.fetch_u16();
            if !mmu.reg.carry {
                let pc = mmu.reg.pc;
                mmu.tick(4);
                push_op(mmu, pc);
                mmu.reg.pc = to;
            }
        }

        // CALL Z, a16: if Z-flag is set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        0xCC => {
            let to = mmu.fetch_u16();
            if mmu.reg.zero {
                let pc = mmu.reg.pc;
                mmu.tick(4);
                push_op(mmu, pc);
                mmu.reg.pc = to;
            }
        }

        // CALL C, a16: if C-flag is set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        0xDC => {
            let to = mmu.fetch_u16();
            if mmu.reg.carry {
                let pc = mmu.reg.pc;
                mmu.tick(4);
                push_op(mmu, pc);
                mmu.reg.pc = to;
            }
        }

        // RST n: push PC and jump to one out of 8 possible addresses
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        0xC7 => {
            rst_op(mmu, 0x0000);
        }
        0xCF => {
            rst_op(mmu, 0x0008);
        }
        0xD7 => {
            rst_op(mmu, 0x0010);
        }
        0xDF => {
            rst_op(mmu, 0x0018);
        }
        0xE7 => {
            rst_op(mmu, 0x0020);
        }
        0xEF => {
            rst_op(mmu, 0x0028);
        }
        0xF7 => {
            rst_op(mmu, 0x0030);
        }
        0xFF => {
            rst_op(mmu, 0x0038);
        }

        // PUSH nn: push 16-bit register nn to stack
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        0xC5 => {
            let bc = mmu.reg.bc();
            mmu.tick(4);
            push_op(mmu, bc);
        }
        0xD5 => {
            let de = mmu.reg.de();
            mmu.tick(4);
            push_op(mmu, de);
        }
        0xE5 => {
            let hl = mmu.reg.hl();
            mmu.tick(4);
            push_op(mmu, hl);
        }
        0xF5 => {
            let af = mmu.reg.af();
            mmu.tick(4);
            push_op(mmu, af);
        }

        // POP nn: pop value from stack and store in 16-bit register nn
        // Length: 1
        // Cycles: 12
        // Flags: - - - -
        0xC1 => {
            let v = pop_op(mmu);
            mmu.reg.set_bc(v);
        }
        0xD1 => {
            let v = pop_op(mmu);
            mmu.reg.set_de(v);
        }
        0xE1 => {
            let v = pop_op(mmu);
            mmu.reg.set_hl(v);
        }
        0xF1 => {
            let v = pop_op(mmu);
            mmu.reg.set_af(v);
        }

        0xE2 => {
            // LD ($FF00+C), A: put value of A in address 0xFF00 + C
            // Length: 1
            // Cycles: 8
            // Flags: - - - -
            // Note: The opcode table at pastraiser.com specify
            // invalid length of 2. The correct length is 1.
            let addr = 0xFF00 + mmu.reg.c as u16;
            let a = mmu.reg.a;
            mmu.write(addr, a);
        }

        // LD A, ($FF00+C): store value at address 0xFF00 + C in A
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0xF2 => {
            let addr = 0xFF00 + mmu.reg.c as u16;
            mmu.reg.a = mmu.read(addr);
        }

        // JR d8: relative jump
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        // TODO: placement of mmu.tick()?
        0x18 => {
            let offs = mmu.fetch() as i8;

            mmu.reg.pc = if offs >= 0 {
                mmu.reg.pc.wrapping_add(offs as u16)
            } else {
                mmu.reg.pc.wrapping_sub(-offs as u16)
            };

            mmu.tick(4);
        }

        // JR NZ, d8: jump d8 relative to PC if Z flag is not set
        // Length: 2
        // Cycles: 12/8
        // Flags: - - - -
        0x20 => {
            let offs = mmu.fetch() as i8;
            if !mmu.reg.zero {
                mmu.reg.pc = if offs >= 0 {
                    mmu.reg.pc.wrapping_add(offs as u16)
                } else {
                    mmu.reg.pc.wrapping_sub(-offs as u16)
                };
                mmu.tick(4);
            }
        }

        // JR NC, d8: jump d8 relative to PC if C flag is not set
        // Length: 2
        // Cycles: 12/8
        // Flags: - - - -
        0x30 => {
            let offs = mmu.fetch() as i8;
            if !mmu.reg.carry {
                mmu.reg.pc = if offs >= 0 {
                    mmu.reg.pc.wrapping_add(offs as u16)
                } else {
                    mmu.reg.pc.wrapping_sub(-offs as u16)
                };
                mmu.tick(4);
            }
        }

        // JR Z, d8: jump d8 relative to PC if Z is set
        // Length: 2
        // Cycles: 12/8
        // Flags: - - - -
        0x28 => {
            let offs = mmu.fetch() as i8;
            if mmu.reg.zero {
                mmu.reg.pc = if offs >= 0 {
                    mmu.reg.pc.wrapping_add(offs as u16)
                } else {
                    mmu.reg.pc.wrapping_sub(-offs as u16)
                };
                mmu.tick(4);
            }
        }

        0x38 => {
            // JR C, d8: jump d8 relative to PC if C is set
            // Length: 2
            // Cycles: 12/8
            // Flags: - - - -
            let offs = mmu.fetch() as i8;

            if mmu.reg.carry {
                mmu.reg.pc = if offs >= 0 {
                    mmu.reg.pc.wrapping_add(offs as u16)
                } else {
                    mmu.reg.pc.wrapping_sub(-offs as u16)
                };

                mmu.tick(4);
            }
        }

        // JP NZ, a16: jump to address a16 if Z is *not* set
        // JP Z, a16: jump to address a16 if Z is set
        // Length: 3
        // Cycles: 16/12
        // Flags: - - - -
        0xC2 => {
            let to = mmu.fetch_u16();
            if !mmu.reg.zero {
                mmu.reg.pc = to;
                mmu.tick(4);
            }
        }
        0xCA => {
            let to = mmu.fetch_u16();
            if mmu.reg.zero {
                mmu.reg.pc = to;
                mmu.tick(4);
            }
        }

        // JP NC, a16: jump to address a16 if C is *not* set
        // JP C, a16: jump to address a16 if C is set
        // Length: 3
        // Cycles: 16/12
        // Flags: - - - -
        0xD2 => {
            let to = mmu.fetch_u16();
            if !mmu.reg.carry {
                mmu.reg.pc = to;
                mmu.tick(4);
            }
        }
        0xDA => {
            let to = mmu.fetch_u16();
            if mmu.reg.carry {
                mmu.reg.pc = to;
                mmu.tick(4);
            }
        }

        // JP a16: jump to immediate address
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xC3 => {
            mmu.reg.pc = mmu.fetch_u16();
            mmu.tick(4);
        }

        // JP (HL): jump to address HL, or in other words: PC = HL
        // Note that this op does *not* set PC to the value stored in memory
        // at address (HL)!
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0xE9 => {
            mmu.reg.pc = mmu.reg.hl();
        }

        0xF9 => {
            // LD SP, HL: set HL to value of SP
            // Length: 1
            // Cycles: 8
            // Flags: - - - -
            mmu.reg.sp = mmu.reg.hl();
            mmu.tick(4);
        }

        // LD (HL-), A: put A into memory address HL, decrement HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x32 => {
            let hl = mmu.reg.hl();
            let a = mmu.reg.a;
            mmu.write(hl, a);
            mmu.reg.set_hl(hl.wrapping_sub(1));
        }

        // XOR N: assign A xor N to A
        // Length: 1
        // Cycles: 4 (8 for op 0xAE)
        // Flags: Z 0 0 0
        0xA8 => {
            let b = mmu.reg.b;
            xor_op(&mut mmu.reg, b);
        }
        0xA9 => {
            let c = mmu.reg.c;
            xor_op(&mut mmu.reg, c);
        }
        0xAA => {
            let d = mmu.reg.d;
            xor_op(&mut mmu.reg, d);
        }
        0xAB => {
            let e = mmu.reg.e;
            xor_op(&mut mmu.reg, e);
        }
        0xAC => {
            let h = mmu.reg.h;
            xor_op(&mut mmu.reg, h);
        }
        0xAD => {
            let l = mmu.reg.l;
            xor_op(&mut mmu.reg, l);
        }
        0xAE => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            xor_op(&mut mmu.reg, v);
        }
        0xAF => {
            let a = mmu.reg.a;
            xor_op(&mut mmu.reg, a);
        }

        // XOR d8: assign A xor d8 to A
        // Length: 2
        // Cycles: 8
        // Flags: Z 0 0 0
        0xEE => {
            let v = mmu.fetch();
            xor_op(&mut mmu.reg, v);
        }

        // RLA: Rotate the contents of register A to the left
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        0x17 => {
            let b0 = if mmu.reg.carry { 1 } else { 0 };
            let b8 = mmu.reg.a & 128 != 0;
            mmu.reg.set_znhc(false, false, false, b8);
            mmu.reg.a = mmu.reg.a << 1 | b0;
        }

        // LD (HL+), A: store value of A at (HL) and increment HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        // Alt mnemonic 1: LD (HLI), A
        // Alt mnemonic 2: LDI (HL), A
        0x22 => {
            let hl = mmu.reg.hl();
            let a = mmu.reg.a;
            mmu.write(hl, a);
            mmu.reg.set_hl(hl + 1);
        }

        // LD (HL), d8: store immediate value at (HL)
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        0x36 => {
            let v = mmu.fetch();
            let hl = mmu.reg.hl();
            mmu.write(hl, v);
        }

        // LD A, (HL+): load value from (HL) to A and increment HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x2A => {
            let hl = mmu.reg.hl();
            mmu.reg.a = mmu.read(hl);
            mmu.reg.set_hl(hl + 1);
        }

        // LD A, (HL-): load value from (HL) to A and decrement HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x3A => {
            let hl = mmu.reg.hl();
            mmu.reg.a = mmu.read(hl);
            mmu.reg.set_hl(hl.wrapping_sub(1));
        }

        // LD (a16), A: store value of A at address a16
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xEA => {
            let addr = mmu.fetch_u16();
            let a = mmu.reg.a;
            mmu.write(addr, a);
        }

        // LD (a16), SP: store SP at address (a16)
        // Length: 3
        // Cycles: 20
        // Flags: - - - -
        0x08 => {
            let addr = mmu.fetch_u16();
            let sp = mmu.reg.sp;
            mmu.write_u16(addr, sp);
        }

        // LD HL, SP+d8: load HL with value of SP + immediate value r8
        // Alt syntax: LDHL SP, d8
        // Length: 2
        // Cycles: 12
        // Flags: 0 0 H C
        // TODO: placement of mmu.tick()?
        0xF8 => {
            // let value = mem.read_i8(reg.pc + 1) as u16;
            let value = mmu.fetch() as i8 as u16;
            mmu.reg.zero = false;
            mmu.reg.neg = false;
            mmu.reg.half_carry = ((mmu.reg.sp & 0x0F) + (value & 0x0F)) > 0x0F;
            mmu.reg.carry = (mmu.reg.sp & 0xFF) + (value & 0xFF) > 0xFF;
            let hl = mmu.reg.sp.wrapping_add(value);
            mmu.reg.set_hl(hl);
            mmu.tick(4);
        }

        // CP r, CP (hl): Compare r (or value at (hl)) with A. Same as SUB but throws away the result
        // Length: 1
        // Cycles: 4 (8 for "CP (hl)")
        // Flags: Z 1 H C
        0xB8 => {
            let b = mmu.reg.b;
            cp_op(&mut mmu.reg, b);
        }
        0xB9 => {
            let c = mmu.reg.c;
            cp_op(&mut mmu.reg, c);
        }
        0xBA => {
            let d = mmu.reg.d;
            cp_op(&mut mmu.reg, d);
        }
        0xBB => {
            let e = mmu.reg.e;
            cp_op(&mut mmu.reg, e);
        }
        0xBC => {
            let h = mmu.reg.h;
            cp_op(&mut mmu.reg, h);
        }
        0xBD => {
            let l = mmu.reg.l;
            cp_op(&mut mmu.reg, l);
        }
        0xBE => {
            let hl = mmu.reg.hl();
            let v = mmu.read(hl);
            cp_op(&mut mmu.reg, v);
        }
        0xBF => {
            mmu.reg.set_znhc(true, true, false, false);
        }

        // CP u8: Compare A with immediate
        // Length: 2
        // Cycles: 8
        // Flags: Z 1 H C
        0xFE => {
            let v = mmu.fetch();
            cp_op(&mut mmu.reg, v);
        }

        0xF3 => {
            // DI: Disable Interrupt Master Enable Flag, prohibits maskable interrupts
            // Length: 1
            // Cycles: 4
            // Flags: - - - -
            mmu.reg.ime = 0;
        }

        0xFB => {
            // EI: Enable Interrupt Master Enable Flag
            // Length: 1
            // Cycles: 4
            // Flags: - - - -
            if mmu.reg.ime == 0 {
                mmu.reg.ime = 1;
            }
        }

        // RLCA: rotate content of register A left, with carry
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        0x07 => {
            // FIXME: don't we have multiple impl of this?
            let a = (mmu.reg.a as u32) << 1;
            if a > 0xFF {
                mmu.reg.a = (a & 0xFF) as u8 | 1;
                mmu.reg.set_znhc(false, false, false, true);
            } else {
                mmu.reg.a = (a & 0xFF) as u8;
                mmu.reg.set_znhc(false, false, false, false);
            }
        }

        // CPL: complement (bitwise not) register A
        // Length: 1
        // Cycles: 4
        // Flags: - 1 1 -
        0x2F => {
            mmu.reg.a = !mmu.reg.a;
            mmu.reg.neg = true;
            mmu.reg.half_carry = true;
        }

        // CCF: Flip carry flag
        // Length: 1
        // Cycles: 4
        // Flags: - 0 0 C
        0x3F => {
            mmu.reg.carry = !mmu.reg.carry;
            mmu.reg.half_carry = false;
            mmu.reg.neg = false;
        }

        // STOP 0
        // Length: 1 (not 2, see https://stackoverflow.com/questions/41353869)
        // Cycles: 4
        0x10 => {
            mmu.reg.stopped = true;
        }

        // Prefix 0xCB instructions
        // All 0xCB operations have length 2
        // All 0xCB operations consume 8 cycles, except for
        // all operations with op code 0x*6 and 0x*E which
        // consume 16 cycles.
        0xCB => {
            let op2 = mmu.fetch();
            match op2 {
                // RLC n: rotate register n left
                // Length:
                0x00 => {
                    let b = mmu.reg.b;
                    mmu.reg.b = rlc_op(&mut mmu.reg, b);
                }
                0x01 => {
                    let c = mmu.reg.c;
                    mmu.reg.c = rlc_op(&mut mmu.reg, c);
                }
                0x02 => {
                    let d = mmu.reg.d;
                    mmu.reg.d = rlc_op(&mut mmu.reg, d);
                }
                0x03 => {
                    let e = mmu.reg.e;
                    mmu.reg.e = rlc_op(&mut mmu.reg, e);
                }
                0x04 => {
                    let h = mmu.reg.h;
                    mmu.reg.h = rlc_op(&mut mmu.reg, h);
                }
                0x05 => {
                    let l = mmu.reg.l;
                    mmu.reg.l = rlc_op(&mut mmu.reg, l);
                }
                0x06 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    let rot = rlc_op(&mut mmu.reg, v);
                    mmu.write(hl, rot);
                }
                0x07 => {
                    let a = mmu.reg.a;
                    mmu.reg.a = rlc_op(&mut mmu.reg, a);
                }

                // RLC n: rotate register n right
                0x08 => {
                    let b = mmu.reg.b;
                    mmu.reg.b = rrc_op(&mut mmu.reg, b);
                }
                0x09 => {
                    let c = mmu.reg.c;
                    mmu.reg.c = rrc_op(&mut mmu.reg, c);
                }
                0x0A => {
                    let d = mmu.reg.d;
                    mmu.reg.d = rrc_op(&mut mmu.reg, d);
                }
                0x0B => {
                    let e = mmu.reg.e;
                    mmu.reg.e = rrc_op(&mut mmu.reg, e);
                }
                0x0C => {
                    let h = mmu.reg.h;
                    mmu.reg.h = rrc_op(&mut mmu.reg, h);
                }
                0x0D => {
                    let l = mmu.reg.l;
                    mmu.reg.l = rrc_op(&mut mmu.reg, l);
                }
                0x0E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    let rot = rrc_op(&mut mmu.reg, v);
                    mmu.write(hl, rot);
                }
                0x0F => {
                    let a = mmu.reg.a;
                    mmu.reg.a = rrc_op(&mut mmu.reg, a);
                }

                // RL n: rotate register n left with carry flag
                0x10 => {
                    let b = mmu.reg.b;
                    mmu.reg.b = rl_op(&mut mmu.reg, b);
                }
                0x11 => {
                    let c = mmu.reg.c;
                    mmu.reg.c = rl_op(&mut mmu.reg, c);
                }
                0x12 => {
                    let d = mmu.reg.d;
                    mmu.reg.d = rl_op(&mut mmu.reg, d);
                }
                0x13 => {
                    let e = mmu.reg.e;
                    mmu.reg.e = rl_op(&mut mmu.reg, e);
                }
                0x14 => {
                    let h = mmu.reg.h;
                    mmu.reg.h = rl_op(&mut mmu.reg, h);
                }
                0x15 => {
                    let l = mmu.reg.l;
                    mmu.reg.l = rl_op(&mut mmu.reg, l);
                }
                0x16 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    let rot = rl_op(&mut mmu.reg, v);
                    mmu.write(hl, rot);
                }
                0x17 => {
                    let a = mmu.reg.a;
                    mmu.reg.a = rl_op(&mut mmu.reg, a);
                }

                // RR n, rotate register n right with carry flag
                0x18 => {
                    let b = mmu.reg.b;
                    mmu.reg.b = rr_op(&mut mmu.reg, b)
                }
                0x19 => {
                    let c = mmu.reg.c;
                    mmu.reg.c = rr_op(&mut mmu.reg, c)
                }
                0x1A => {
                    let d = mmu.reg.d;
                    mmu.reg.d = rr_op(&mut mmu.reg, d)
                }
                0x1B => {
                    let e = mmu.reg.e;
                    mmu.reg.e = rr_op(&mut mmu.reg, e)
                }
                0x1C => {
                    let h = mmu.reg.h;
                    mmu.reg.h = rr_op(&mut mmu.reg, h)
                }
                0x1D => {
                    let l = mmu.reg.l;
                    mmu.reg.l = rr_op(&mut mmu.reg, l)
                }
                0x1E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    let rot = rr_op(&mut mmu.reg, v);
                    mmu.write(hl, rot);
                }
                0x1F => {
                    let a = mmu.reg.a;
                    mmu.reg.a = rr_op(&mut mmu.reg, a)
                }

                // SLA r
                0x20 => {
                    let b = mmu.reg.b;
                    mmu.reg.b = sla_op(&mut mmu.reg, b)
                }
                0x21 => {
                    let c = mmu.reg.c;
                    mmu.reg.c = sla_op(&mut mmu.reg, c)
                }
                0x22 => {
                    let d = mmu.reg.d;
                    mmu.reg.d = sla_op(&mut mmu.reg, d)
                }
                0x23 => {
                    let e = mmu.reg.e;
                    mmu.reg.e = sla_op(&mut mmu.reg, e)
                }
                0x24 => {
                    let h = mmu.reg.h;
                    mmu.reg.h = sla_op(&mut mmu.reg, h)
                }
                0x25 => {
                    let l = mmu.reg.l;
                    mmu.reg.l = sla_op(&mut mmu.reg, l)
                }
                0x26 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    let result = sla_op(&mut mmu.reg, v);
                    mmu.write(hl, result);
                }
                0x27 => {
                    let a = mmu.reg.a;
                    mmu.reg.a = sla_op(&mut mmu.reg, a)
                }

                // SRA r
                0x28 => {
                    let b = mmu.reg.b;
                    mmu.reg.b = sra_op(&mut mmu.reg, b)
                }
                0x29 => {
                    let c = mmu.reg.c;
                    mmu.reg.c = sra_op(&mut mmu.reg, c)
                }
                0x2A => {
                    let d = mmu.reg.d;
                    mmu.reg.d = sra_op(&mut mmu.reg, d)
                }
                0x2B => {
                    let e = mmu.reg.e;
                    mmu.reg.e = sra_op(&mut mmu.reg, e)
                }
                0x2C => {
                    let h = mmu.reg.h;
                    mmu.reg.h = sra_op(&mut mmu.reg, h)
                }
                0x2D => {
                    let l = mmu.reg.l;
                    mmu.reg.l = sra_op(&mut mmu.reg, l)
                }
                0x2E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    let result = sra_op(&mut mmu.reg, v);
                    mmu.write(hl, result);
                }
                0x2F => {
                    let a = mmu.reg.a;
                    mmu.reg.a = sra_op(&mut mmu.reg, a)
                }

                // SWAP r
                0x30 => {
                    let b = mmu.reg.b;
                    mmu.reg.b = swap_op(&mut mmu.reg, b)
                }
                0x31 => {
                    let c = mmu.reg.c;
                    mmu.reg.c = swap_op(&mut mmu.reg, c)
                }
                0x32 => {
                    let d = mmu.reg.d;
                    mmu.reg.d = swap_op(&mut mmu.reg, d)
                }
                0x33 => {
                    let e = mmu.reg.e;
                    mmu.reg.e = swap_op(&mut mmu.reg, e)
                }
                0x34 => {
                    let h = mmu.reg.h;
                    mmu.reg.h = swap_op(&mut mmu.reg, h)
                }
                0x35 => {
                    let l = mmu.reg.l;
                    mmu.reg.l = swap_op(&mut mmu.reg, l)
                }
                0x36 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    let result = swap_op(&mut mmu.reg, v);
                    mmu.write(hl, result);
                }
                0x37 => {
                    let a = mmu.reg.a;
                    mmu.reg.a = swap_op(&mut mmu.reg, a)
                }

                // SRL r
                0x38 => {
                    let b = mmu.reg.b;
                    mmu.reg.b = srl_op(&mut mmu.reg, b)
                }
                0x39 => {
                    let c = mmu.reg.c;
                    mmu.reg.c = srl_op(&mut mmu.reg, c)
                }
                0x3A => {
                    let d = mmu.reg.d;
                    mmu.reg.d = srl_op(&mut mmu.reg, d)
                }
                0x3B => {
                    let e = mmu.reg.e;
                    mmu.reg.e = srl_op(&mut mmu.reg, e)
                }
                0x3C => {
                    let h = mmu.reg.h;
                    mmu.reg.h = srl_op(&mut mmu.reg, h)
                }
                0x3D => {
                    let l = mmu.reg.l;
                    mmu.reg.l = srl_op(&mut mmu.reg, l)
                }
                0x3E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    let result = srl_op(&mut mmu.reg, v);
                    mmu.write(hl, result);
                }
                0x3F => {
                    let a = mmu.reg.a;
                    mmu.reg.a = srl_op(&mut mmu.reg, a)
                }

                // BIT b, r: test if bit 'b' in register 'r' is set
                // Flags: Z 0 1 -
                // TODO: does op 0x46, 0x4E, etc really consume 16 cycles?
                0x40 => {
                    let b = mmu.reg.b;
                    bit_op(&mut mmu.reg, 0, b);
                }
                0x41 => {
                    let c = mmu.reg.c;
                    bit_op(&mut mmu.reg, 0, c);
                }
                0x42 => {
                    let d = mmu.reg.d;
                    bit_op(&mut mmu.reg, 0, d);
                }
                0x43 => {
                    let e = mmu.reg.e;
                    bit_op(&mut mmu.reg, 0, e);
                }
                0x44 => {
                    let h = mmu.reg.h;
                    bit_op(&mut mmu.reg, 0, h);
                }
                0x45 => {
                    let l = mmu.reg.l;
                    bit_op(&mut mmu.reg, 0, l);
                }
                0x46 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    bit_op(&mut mmu.reg, 0, v)
                }
                0x47 => {
                    let a = mmu.reg.a;
                    bit_op(&mut mmu.reg, 0, a);
                }

                0x48 => {
                    let b = mmu.reg.b;
                    bit_op(&mut mmu.reg, 1, b);
                }
                0x49 => {
                    let c = mmu.reg.c;
                    bit_op(&mut mmu.reg, 1, c);
                }
                0x4A => {
                    let d = mmu.reg.d;
                    bit_op(&mut mmu.reg, 1, d);
                }
                0x4B => {
                    let e = mmu.reg.e;
                    bit_op(&mut mmu.reg, 1, e);
                }
                0x4C => {
                    let h = mmu.reg.h;
                    bit_op(&mut mmu.reg, 1, h);
                }
                0x4D => {
                    let l = mmu.reg.l;
                    bit_op(&mut mmu.reg, 1, l);
                }
                0x4E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    bit_op(&mut mmu.reg, 1, v)
                }
                0x4F => {
                    let a = mmu.reg.a;
                    bit_op(&mut mmu.reg, 1, a);
                }

                0x50 => {
                    let b = mmu.reg.b;
                    bit_op(&mut mmu.reg, 2, b);
                }
                0x51 => {
                    let c = mmu.reg.c;
                    bit_op(&mut mmu.reg, 2, c);
                }
                0x52 => {
                    let d = mmu.reg.d;
                    bit_op(&mut mmu.reg, 2, d);
                }
                0x53 => {
                    let e = mmu.reg.e;
                    bit_op(&mut mmu.reg, 2, e);
                }
                0x54 => {
                    let h = mmu.reg.h;
                    bit_op(&mut mmu.reg, 2, h);
                }
                0x55 => {
                    let l = mmu.reg.l;
                    bit_op(&mut mmu.reg, 2, l);
                }
                0x56 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    bit_op(&mut mmu.reg, 2, v)
                }
                0x57 => {
                    let a = mmu.reg.a;
                    bit_op(&mut mmu.reg, 2, a);
                }

                0x58 => {
                    let b = mmu.reg.b;
                    bit_op(&mut mmu.reg, 3, b);
                }
                0x59 => {
                    let c = mmu.reg.c;
                    bit_op(&mut mmu.reg, 3, c);
                }
                0x5A => {
                    let d = mmu.reg.d;
                    bit_op(&mut mmu.reg, 3, d);
                }
                0x5B => {
                    let e = mmu.reg.e;
                    bit_op(&mut mmu.reg, 3, e);
                }
                0x5C => {
                    let h = mmu.reg.h;
                    bit_op(&mut mmu.reg, 3, h);
                }
                0x5D => {
                    let l = mmu.reg.l;
                    bit_op(&mut mmu.reg, 3, l);
                }
                0x5E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    bit_op(&mut mmu.reg, 3, v)
                }
                0x5F => {
                    let a = mmu.reg.a;
                    bit_op(&mut mmu.reg, 3, a);
                }

                0x60 => {
                    let b = mmu.reg.b;
                    bit_op(&mut mmu.reg, 4, b);
                }
                0x61 => {
                    let c = mmu.reg.c;
                    bit_op(&mut mmu.reg, 4, c);
                }
                0x62 => {
                    let d = mmu.reg.d;
                    bit_op(&mut mmu.reg, 4, d);
                }
                0x63 => {
                    let e = mmu.reg.e;
                    bit_op(&mut mmu.reg, 4, e);
                }
                0x64 => {
                    let h = mmu.reg.h;
                    bit_op(&mut mmu.reg, 4, h);
                }
                0x65 => {
                    let l = mmu.reg.l;
                    bit_op(&mut mmu.reg, 4, l);
                }
                0x66 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    bit_op(&mut mmu.reg, 4, v)
                }
                0x67 => {
                    let a = mmu.reg.a;
                    bit_op(&mut mmu.reg, 4, a);
                }

                0x68 => {
                    let b = mmu.reg.b;
                    bit_op(&mut mmu.reg, 5, b);
                }
                0x69 => {
                    let c = mmu.reg.c;
                    bit_op(&mut mmu.reg, 5, c);
                }
                0x6A => {
                    let d = mmu.reg.d;
                    bit_op(&mut mmu.reg, 5, d);
                }
                0x6B => {
                    let e = mmu.reg.e;
                    bit_op(&mut mmu.reg, 5, e);
                }
                0x6C => {
                    let h = mmu.reg.h;
                    bit_op(&mut mmu.reg, 5, h);
                }
                0x6D => {
                    let l = mmu.reg.l;
                    bit_op(&mut mmu.reg, 5, l);
                }
                0x6E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    bit_op(&mut mmu.reg, 5, v)
                }
                0x6F => {
                    let a = mmu.reg.a;
                    bit_op(&mut mmu.reg, 5, a);
                }

                0x70 => {
                    let b = mmu.reg.b;
                    bit_op(&mut mmu.reg, 6, b);
                }
                0x71 => {
                    let c = mmu.reg.c;
                    bit_op(&mut mmu.reg, 6, c);
                }
                0x72 => {
                    let d = mmu.reg.d;
                    bit_op(&mut mmu.reg, 6, d);
                }
                0x73 => {
                    let e = mmu.reg.e;
                    bit_op(&mut mmu.reg, 6, e);
                }
                0x74 => {
                    let h = mmu.reg.h;
                    bit_op(&mut mmu.reg, 6, h);
                }
                0x75 => {
                    let l = mmu.reg.l;
                    bit_op(&mut mmu.reg, 6, l);
                }
                0x76 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    bit_op(&mut mmu.reg, 6, v)
                }
                0x77 => {
                    let a = mmu.reg.a;
                    bit_op(&mut mmu.reg, 6, a);
                }

                0x78 => {
                    let b = mmu.reg.b;
                    bit_op(&mut mmu.reg, 7, b);
                }
                0x79 => {
                    let c = mmu.reg.c;
                    bit_op(&mut mmu.reg, 7, c);
                }
                0x7A => {
                    let d = mmu.reg.d;
                    bit_op(&mut mmu.reg, 7, d);
                }
                0x7B => {
                    let e = mmu.reg.e;
                    bit_op(&mut mmu.reg, 7, e);
                }
                0x7C => {
                    let h = mmu.reg.h;
                    bit_op(&mut mmu.reg, 7, h);
                }
                0x7D => {
                    let l = mmu.reg.l;
                    bit_op(&mut mmu.reg, 7, l);
                }
                0x7E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    bit_op(&mut mmu.reg, 7, v)
                }
                0x7F => {
                    let a = mmu.reg.a;
                    bit_op(&mut mmu.reg, 7, a);
                }

                // RES b, r: reset bit b in register r
                // Length: 2
                // Cycles: 8
                // Flags: - - - -
                0x80 => {
                    mmu.reg.b &= !1;
                }
                0x81 => {
                    mmu.reg.c &= !1;
                }
                0x82 => {
                    mmu.reg.d &= !1;
                }
                0x83 => {
                    mmu.reg.e &= !1;
                }
                0x84 => {
                    mmu.reg.h &= !1;
                }
                0x85 => {
                    mmu.reg.l &= !1;
                }
                0x86 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v & !1);
                }
                0x87 => {
                    mmu.reg.a &= !1;
                }

                0x88 => {
                    mmu.reg.b &= !2;
                }
                0x89 => {
                    mmu.reg.c &= !2;
                }
                0x8A => {
                    mmu.reg.d &= !2;
                }
                0x8B => {
                    mmu.reg.e &= !2;
                }
                0x8C => {
                    mmu.reg.h &= !2;
                }
                0x8D => {
                    mmu.reg.l &= !2;
                }
                0x8E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v & !2);
                }
                0x8F => {
                    mmu.reg.a &= !2;
                }

                0x90 => {
                    mmu.reg.b &= !4;
                }
                0x91 => {
                    mmu.reg.c &= !4;
                }
                0x92 => {
                    mmu.reg.d &= !4;
                }
                0x93 => {
                    mmu.reg.e &= !4;
                }
                0x94 => {
                    mmu.reg.h &= !4;
                }
                0x95 => {
                    mmu.reg.l &= !4;
                }
                0x96 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v & !4);
                }
                0x97 => {
                    mmu.reg.a &= !4;
                }

                0x98 => {
                    mmu.reg.b &= !8;
                }
                0x99 => {
                    mmu.reg.c &= !8;
                }
                0x9A => {
                    mmu.reg.d &= !8;
                }
                0x9B => {
                    mmu.reg.e &= !8;
                }
                0x9C => {
                    mmu.reg.h &= !8;
                }
                0x9D => {
                    mmu.reg.l &= !8;
                }
                0x9E => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v & !8);
                }
                0x9F => {
                    mmu.reg.a &= !8;
                }

                0xA0 => {
                    mmu.reg.b &= !16;
                }
                0xA1 => {
                    mmu.reg.c &= !16;
                }
                0xA2 => {
                    mmu.reg.d &= !16;
                }
                0xA3 => {
                    mmu.reg.e &= !16;
                }
                0xA4 => {
                    mmu.reg.h &= !16;
                }
                0xA5 => {
                    mmu.reg.l &= !16;
                }
                0xA6 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v & !16);
                }
                0xA7 => {
                    mmu.reg.a &= !16;
                }

                0xA8 => {
                    mmu.reg.b &= !32;
                }
                0xA9 => {
                    mmu.reg.c &= !32;
                }
                0xAA => {
                    mmu.reg.d &= !32;
                }
                0xAB => {
                    mmu.reg.e &= !32;
                }
                0xAC => {
                    mmu.reg.h &= !32;
                }
                0xAD => {
                    mmu.reg.l &= !32;
                }
                0xAE => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v & !32);
                }
                0xAF => {
                    mmu.reg.a &= !32;
                }

                0xB0 => {
                    mmu.reg.b &= !64;
                }
                0xB1 => {
                    mmu.reg.c &= !64;
                }
                0xB2 => {
                    mmu.reg.d &= !64;
                }
                0xB3 => {
                    mmu.reg.e &= !64;
                }
                0xB4 => {
                    mmu.reg.h &= !64;
                }
                0xB5 => {
                    mmu.reg.l &= !64;
                }
                0xB6 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v & !64);
                }
                0xB7 => {
                    mmu.reg.a &= !64;
                }

                0xB8 => {
                    mmu.reg.b &= !128;
                }
                0xB9 => {
                    mmu.reg.c &= !128;
                }
                0xBA => {
                    mmu.reg.d &= !128;
                }
                0xBB => {
                    mmu.reg.e &= !128;
                }
                0xBC => {
                    mmu.reg.h &= !128;
                }
                0xBD => {
                    mmu.reg.l &= !128;
                }
                0xBE => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v & !128);
                }
                0xBF => {
                    mmu.reg.a &= !128;
                }

                // SET b, r: set bit b in register r
                // Flags: - - - -
                0xC0 => {
                    mmu.reg.b |= 1;
                }
                0xC1 => {
                    mmu.reg.c |= 1;
                }
                0xC2 => {
                    mmu.reg.d |= 1;
                }
                0xC3 => {
                    mmu.reg.e |= 1;
                }
                0xC4 => {
                    mmu.reg.h |= 1;
                }
                0xC5 => {
                    mmu.reg.l |= 1;
                }
                0xC6 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v | 1);
                }
                0xC7 => {
                    mmu.reg.a |= 1;
                }

                0xC8 => {
                    mmu.reg.b |= 2;
                }
                0xC9 => {
                    mmu.reg.c |= 2;
                }
                0xCA => {
                    mmu.reg.d |= 2;
                }
                0xCB => {
                    mmu.reg.e |= 2;
                }
                0xCC => {
                    mmu.reg.h |= 2;
                }
                0xCD => {
                    mmu.reg.l |= 2;
                }
                0xCE => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v | 2);
                }
                0xCF => {
                    mmu.reg.a |= 2;
                }

                0xD0 => {
                    mmu.reg.b |= 4;
                }
                0xD1 => {
                    mmu.reg.c |= 4;
                }
                0xD2 => {
                    mmu.reg.d |= 4;
                }
                0xD3 => {
                    mmu.reg.e |= 4;
                }
                0xD4 => {
                    mmu.reg.h |= 4;
                }
                0xD5 => {
                    mmu.reg.l |= 4;
                }
                0xD6 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v | 4);
                }
                0xD7 => {
                    mmu.reg.a |= 4;
                }

                0xD8 => {
                    mmu.reg.b |= 8;
                }
                0xD9 => {
                    mmu.reg.c |= 8;
                }
                0xDA => {
                    mmu.reg.d |= 8;
                }
                0xDB => {
                    mmu.reg.e |= 8;
                }
                0xDC => {
                    mmu.reg.h |= 8;
                }
                0xDD => {
                    mmu.reg.l |= 8;
                }
                0xDE => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v | 8);
                }
                0xDF => {
                    mmu.reg.a |= 8;
                }

                0xE0 => {
                    mmu.reg.b |= 16;
                }
                0xE1 => {
                    mmu.reg.c |= 16;
                }
                0xE2 => {
                    mmu.reg.d |= 16;
                }
                0xE3 => {
                    mmu.reg.e |= 16;
                }
                0xE4 => {
                    mmu.reg.h |= 16;
                }
                0xE5 => {
                    mmu.reg.l |= 16;
                }
                0xE6 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v | 16);
                }
                0xE7 => {
                    mmu.reg.a |= 16;
                }

                0xE8 => {
                    mmu.reg.b |= 32;
                }
                0xE9 => {
                    mmu.reg.c |= 32;
                }
                0xEA => {
                    mmu.reg.d |= 32;
                }
                0xEB => {
                    mmu.reg.e |= 32;
                }
                0xEC => {
                    mmu.reg.h |= 32;
                }
                0xED => {
                    mmu.reg.l |= 32;
                }
                0xEE => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v | 32);
                }
                0xEF => {
                    mmu.reg.a |= 32;
                }

                0xF0 => {
                    mmu.reg.b |= 64;
                }
                0xF1 => {
                    mmu.reg.c |= 64;
                }
                0xF2 => {
                    mmu.reg.d |= 64;
                }
                0xF3 => {
                    mmu.reg.e |= 64;
                }
                0xF4 => {
                    mmu.reg.h |= 64;
                }
                0xF5 => {
                    mmu.reg.l |= 64;
                }
                0xF6 => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v | 64);
                }
                0xF7 => {
                    mmu.reg.a |= 64;
                }

                0xF8 => {
                    mmu.reg.b |= 128;
                }
                0xF9 => {
                    mmu.reg.c |= 128;
                }
                0xFA => {
                    mmu.reg.d |= 128;
                }
                0xFB => {
                    mmu.reg.e |= 128;
                }
                0xFC => {
                    mmu.reg.h |= 128;
                }
                0xFD => {
                    mmu.reg.l |= 128;
                }
                0xFE => {
                    let hl = mmu.reg.hl();
                    let v = mmu.read(hl);
                    mmu.write(hl, v | 128);
                }
                0xFF => {
                    mmu.reg.a |= 128;
                }
            }
        }

        _ => {
            panic!("Unsupported opcode at 0x{:04X}: 0x{:02X}", mmu.reg.pc, op);
        }
    }
}
