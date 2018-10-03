
use registers::Registers;
use cpu::Cpu;

pub fn op_cycles(op: u8) -> u32 {
    const OP_CYCLES: [u32;256] = [
        1,3,2,2,1,1,2,1,5,2,2,2,1,1,2,1,
        0,3,2,2,1,1,2,1,3,2,2,2,1,1,2,1,
        2,3,2,2,1,1,2,1,2,2,2,2,1,1,2,1,
        2,3,2,2,3,3,3,1,2,2,2,2,1,1,2,1,
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        2,2,2,2,2,2,0,2,1,1,1,1,1,1,2,1,
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
        1,1,1,1,1,1,2,1,1,1,1,1,1,1,2,1,
	    2,3,3,4,3,4,2,4,2,4,3,0,3,6,2,4,
	    2,3,3,0,3,4,2,4,2,4,3,0,3,0,2,4,
	    3,3,2,0,0,4,2,4,4,1,4,0,0,0,2,4,
	    3,3,2,1,0,4,2,4,3,2,4,1,0,0,2,4
    ];

    return OP_CYCLES[op as usize] * 4;
}

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
pub fn push_op(cpu: &mut Cpu, value: u16) {
    let sp = cpu.reg.sp - 2;
    cpu.write_u16(sp, value);
    cpu.reg.sp = sp;
}

// 16-bit pop operation
// Flags: - - - -
// Cycles: 12
// Note that flags are still affected by POP AF
fn pop_op(cpu: &mut Cpu) -> u16 {
    let sp = cpu.reg.sp;
    let v = cpu.read_u16(sp);
    cpu.reg.sp += 2;
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

pub fn rst_op(cpu: &mut Cpu, address: u16) {
    let pc = cpu.reg.pc;
    push_op(cpu, pc);
    cpu.reg.pc = address;
    cpu.tick(4);
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

pub fn step(cpu: &mut Cpu) {
    let op: u8 = cpu.fetch();

    match op {
        // NOP: no operation
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0x00 => {}

        // SCF: Set Carry Flag
        // Length: 1
        // Cycles: 4
        // Flags: - 0 0 1
        0x37 => {
            cpu.reg.neg = false;
            cpu.reg.half_carry = false;
            cpu.reg.carry = true;
        }

        // DAA: ...
        // Length: 1
        // Cycles: 4
        // Flags: Z - 0 C
        0x27 => { daa_op(&mut cpu.reg) }

        // LD rr, d16: load immediate (d16) into 16-bit register rr
        // Length: 3
        // Cycles: 12
        // Flags: - - - -
        0x01 => {
            cpu.reg.c = cpu.fetch();
            cpu.reg.b = cpu.fetch();
        }
        0x11 => {
            cpu.reg.e = cpu.fetch();
            cpu.reg.d = cpu.fetch();
        }
        0x21 => {
            cpu.reg.l = cpu.fetch();
            cpu.reg.h = cpu.fetch();
        }
        0x31 => {
            cpu.reg.sp = cpu.fetch_u16();
        }

        // LD (rr), A: stores the contents of register A in the memory specified by register pair BC or DE.
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x02 => {
            let bc = cpu.reg.bc();
            let a = cpu.reg.a;
            cpu.write(bc, a);
        }
        0x12 => { 
            let de = cpu.reg.de();
            let a = cpu.reg.a;
            cpu.write(de, a);  
        }

        // LD A, (nn): loads value stored in memory at address nn (immediate)
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xFA => {
            let addr = cpu.fetch_u16();
            cpu.reg.a = cpu.read(addr);
        }

        // INC n: increment register n
        // Length: 1
        // Cycles: 4
        // Flags: Z 0 H -
        0x04 => { let b = cpu.reg.b; cpu.reg.b = inc_op(&mut cpu.reg, b); }
        0x0C => { let c = cpu.reg.c; cpu.reg.c = inc_op(&mut cpu.reg, c); }
        0x14 => { let d = cpu.reg.d; cpu.reg.d = inc_op(&mut cpu.reg, d); }
        0x1C => { let e = cpu.reg.e; cpu.reg.e = inc_op(&mut cpu.reg, e); }
        0x24 => { let h = cpu.reg.h; cpu.reg.h = inc_op(&mut cpu.reg, h); }
        0x2C => { let l = cpu.reg.l; cpu.reg.l = inc_op(&mut cpu.reg, l); }
        0x3C => { let a = cpu.reg.a; cpu.reg.a = inc_op(&mut cpu.reg, a); }

        // INC (HL): increment memory stored at HL
        // Length: 1
        // Cycles: 12
        // Flags: Z 0 H -
        0x34 => {
            let hl = cpu.reg.hl();
            let v = cpu.read(hl);
            let v = inc_op(&mut cpu.reg, v);
            cpu.write(hl, v);
        }

        // INC nn: increments content of register pair nn by 1
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        // TODO: placement of cpu.tick()?
        0x03 => {
            let bc = inc16_op(cpu.reg.bc());
            cpu.reg.set_bc(bc);
            cpu.tick(4);
        }
        0x13 => {
            let de = inc16_op(cpu.reg.de());
            cpu.reg.set_de(de);
            cpu.tick(4);
        }
        0x23 => {
            let hl = inc16_op(cpu.reg.hl());
            cpu.reg.set_hl(hl);
            cpu.tick(4);
        }
        0x33 => {
            cpu.reg.sp = inc16_op(cpu.reg.sp);
            cpu.tick(4);
        }

        // DEC n: decrement register n
        // Length: 1
        // Cycles: 4
        // Flags: Z 1 H -
        0x05 => { let b = cpu.reg.b; cpu.reg.b = dec_op(&mut cpu.reg, b); }
        0x0D => { let c = cpu.reg.c; cpu.reg.c = dec_op(&mut cpu.reg, c); }
        0x15 => { let d = cpu.reg.d; cpu.reg.d = dec_op(&mut cpu.reg, d); }
        0x1D => { let e = cpu.reg.e; cpu.reg.e = dec_op(&mut cpu.reg, e); }
        0x25 => { let h = cpu.reg.h; cpu.reg.h = dec_op(&mut cpu.reg, h); }
        0x2D => { let l = cpu.reg.l; cpu.reg.l = dec_op(&mut cpu.reg, l); }
        0x3D => { let a = cpu.reg.a; cpu.reg.a = dec_op(&mut cpu.reg, a); }

        // DEC rr: decrement register pair rr
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        // TODO: placement of cpu.tick()?
        0x0B => {
            let bc = cpu.reg.bc();
            cpu.reg.set_bc(bc.wrapping_sub(1));
            cpu.tick(4);
        }
        0x1B => {
            let de = cpu.reg.de();
            cpu.reg.set_de(de.wrapping_sub(1));
            cpu.tick(4);
        }
        0x2B => {
            let hl = cpu.reg.hl();
            cpu.reg.set_hl(hl.wrapping_sub(1));
            cpu.tick(4);
        }
        0x3B => {
            cpu.reg.sp = cpu.reg.sp.wrapping_sub(1);
            cpu.tick(4);
        }

        // DEC (HL): decrement memory stored at HL
        // Length: 1
        // Cycles: 12
        // Flags: Z 1 H -
        0x35 => {
            let hl = cpu.reg.hl();
            let v = cpu.read(hl);
            let v = dec_op(&mut cpu.reg, v);
            cpu.write(hl, v);
        }

        // ADD r, ADD (hl): add register r or value at (hl) to accumulator
        // Length: 1
        // Cycles: 4 (8 for op 0x86)
        // Flags: Z 0 H C
        0x80 => { let b = cpu.reg.b; add_op(&mut cpu.reg, b); }
        0x81 => { let c = cpu.reg.c; add_op(&mut cpu.reg, c); }
        0x82 => { let d = cpu.reg.d; add_op(&mut cpu.reg, d); }
        0x83 => { let e = cpu.reg.e; add_op(&mut cpu.reg, e); }
        0x84 => { let h = cpu.reg.h; add_op(&mut cpu.reg, h); }
        0x85 => { let l = cpu.reg.l; add_op(&mut cpu.reg, l); }
        0x86 => {
            let hl = cpu.reg.hl();
            let v = cpu.read(hl);
            add_op(&mut cpu.reg, v);
        }
        0x87 => { let a = cpu.reg.a; add_op(&mut cpu.reg, a) }

        // ADD A, d8: add immediate value d8 to A
        // Length: 2
        // Cycles: 8
        // Flags: Z 0 H C
        0xC6 => { let v = cpu.fetch(); add_op(&mut cpu.reg, v); }

        // ADC A, r: add register r + carry to A
        // Length: 1
        // Cycles: 4 (8 for op 0x8E)
        // Flags: Z 0 H C
        0x88 => { let b = cpu.reg.b; adc_op(&mut cpu.reg, b); }
        0x89 => { let c = cpu.reg.c; adc_op(&mut cpu.reg, c); }
        0x8A => { let d = cpu.reg.d; adc_op(&mut cpu.reg, d); }
        0x8B => { let e = cpu.reg.e; adc_op(&mut cpu.reg, e); }
        0x8C => { let h = cpu.reg.h; adc_op(&mut cpu.reg, h); }
        0x8D => { let l = cpu.reg.l; adc_op(&mut cpu.reg, l); }
        0x8E => { let hl = cpu.reg.hl(); let v = cpu.read(hl); adc_op(&mut cpu.reg, v); }
        0x8F => { let a = cpu.reg.a; adc_op(&mut cpu.reg, a); }

        // ADC A, d8: add immediate value + carry to A
        // Length: 2
        // Cycles: 8
        // Flags: Z 0 H C
        //0xCE => { let d8 = mem.read(reg.pc + 1); adc_op(reg, d8) }
        0xCE => { let v = cpu.fetch(); adc_op(&mut cpu.reg, v); }

        // SBC A, r: subtract register r and carry from A
        // Length: 1
        // Cycles: 4 (8)
        // Flags: Z 1 H C
        0x98 => { let b = cpu.reg.b; sbc_op(&mut cpu.reg, b) }
        0x99 => { let c = cpu.reg.c; sbc_op(&mut cpu.reg, c) }
        0x9A => { let d = cpu.reg.d; sbc_op(&mut cpu.reg, d) }
        0x9B => { let e = cpu.reg.e; sbc_op(&mut cpu.reg, e) }
        0x9C => { let h = cpu.reg.h; sbc_op(&mut cpu.reg, h) }
        0x9D => { let l = cpu.reg.l; sbc_op(&mut cpu.reg, l) }
        0x9E => { let hl = cpu.reg.hl(); let v = cpu.read(hl); sbc_op(&mut cpu.reg, v) }
        0x9F => { let a = cpu.reg.a; sbc_op(&mut cpu.reg, a) }

        // SBC A, d8: subtract immediate value and carry from A
        0xDE => { let d8 = cpu.fetch(); sbc_op(&mut cpu.reg, d8) }

        // ADD HL, rr: adds value of register pair rr to HL and stores result in HL
        // Length: 1
        // Cycles: 8
        // Flags: - 0 H C
        // TODO: placement of cpu.tick()?
        0x09 => { let bc = cpu.reg.bc(); add_hl_op(&mut cpu.reg, bc); cpu.tick(4); }
        0x19 => { let de = cpu.reg.de(); add_hl_op(&mut cpu.reg, de); cpu.tick(4); }
        0x29 => { let hl = cpu.reg.hl(); add_hl_op(&mut cpu.reg, hl); cpu.tick(4); }
        0x39 => { let sp = cpu.reg.sp; add_hl_op(&mut cpu.reg, sp); cpu.tick(4); }

        // ADD SP, d8: add immediate value d8 to SP
        // Length: 2
        // Cycles: 16
        // Flags: 0 0 H C
        // TODO: this is very similar to the add_hl_op. could they be combined?
        0xE8 => {
            // let value = mem.read_i8(reg.pc + 1) as u16;
            let value = cpu.fetch() as i8 as u16;

            let hc = ((cpu.reg.sp & 0x0F) + (value & 0x0F)) > 0x0F;

            cpu.reg.half_carry = hc;
            cpu.reg.carry = (cpu.reg.sp & 0xFF) + (value & 0xFF) > 0xFF;
            cpu.reg.zero = false;
            cpu.reg.neg = false;

            cpu.reg.sp = cpu.reg.sp.wrapping_add(value);
        }

        // SUB r, SUB (hl): subtract register r or value at (hl) from accumulator
        // Length: 1
        // Cycles: 4 (8 for op 0x96)
        // Flags: Z 1 H C
        0x90 => { let b = cpu.reg.b; sub_op(&mut cpu.reg, b) }
        0x91 => { let c = cpu.reg.c; sub_op(&mut cpu.reg, c) }
        0x92 => { let d = cpu.reg.d; sub_op(&mut cpu.reg, d) }
        0x93 => { let e = cpu.reg.e; sub_op(&mut cpu.reg, e) }
        0x94 => { let h = cpu.reg.h; sub_op(&mut cpu.reg, h) }
        0x95 => { let l = cpu.reg.l; sub_op(&mut cpu.reg, l) }
        0x96 => {
            let hl = cpu.reg.hl();
            let v = cpu.read(hl);
            sub_op(&mut cpu.reg, v);
        }
        0x97 => { let a = cpu.reg.a; sub_op(&mut cpu.reg, a) }

        // SUB d8: subtract immediate value d8 from A
        // Length: 2
        // Cycles: 8
        // Flags: Z 1 H C
        0xD6 => { let v = cpu.fetch(); sub_op(&mut cpu.reg, v); }

        // AND r, AND (hl), AND d8: set A to "A AND r", or "A AND (hl)""
        // Length: 1 (2 for op 0xE6)
        // Cycles: 4 (8 for op 0xA6 and 0xE6)
        // Flags: Z 0 1 0
        0xA0 => { let b = cpu.reg.b; and_op(&mut cpu.reg, b) }
        0xA1 => { let c = cpu.reg.c; and_op(&mut cpu.reg, c) }
        0xA2 => { let d = cpu.reg.d; and_op(&mut cpu.reg, d) }
        0xA3 => { let e = cpu.reg.e; and_op(&mut cpu.reg, e) }
        0xA4 => { let h = cpu.reg.h; and_op(&mut cpu.reg, h) }
        0xA5 => { let l = cpu.reg.l; and_op(&mut cpu.reg, l) }
        0xA6 => {
            let hl = cpu.reg.hl();
            let v = cpu.read(hl);
            and_op(&mut cpu.reg, v);
        }
        0xA7 => { let a = cpu.reg.a; and_op(&mut cpu.reg, a) }
        0xE6 => { let v = cpu.fetch(); and_op(&mut cpu.reg, v) }

        // OR r, OR (hl): set A to "A OR r", or "A OR (hl)""
        // Length: 1 (2 for 0xF6)
        // Cycles: 4 (8 for op 0xB6 and 0xF6)
        // Flags: Z 0 0 0
        0xB0 => { let b = cpu.reg.b; or_op(&mut cpu.reg, b) }
        0xB1 => { let c = cpu.reg.c; or_op(&mut cpu.reg, c) }
        0xB2 => { let d = cpu.reg.d; or_op(&mut cpu.reg, d) }
        0xB3 => { let e = cpu.reg.e; or_op(&mut cpu.reg, e) }
        0xB4 => { let h = cpu.reg.h; or_op(&mut cpu.reg, h) }
        0xB5 => { let l = cpu.reg.l; or_op(&mut cpu.reg, l) }
        0xB6 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); or_op(&mut cpu.reg, v); }
        0xB7 => { let a = cpu.reg.a; or_op(&mut cpu.reg, a) }
        0xF6 => { let v = cpu.fetch(); or_op(&mut cpu.reg, v) }

        // RRCA: ...
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        // Note that rrc_op() sets Z flag, but RRCA should always clear Z flag
        0x0F => { let a = cpu.reg.a; cpu.reg.a = rrc_op(&mut cpu.reg, a); cpu.reg.zero = false; }

        // RRA: ...
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        // Note that rr_op() sets Z flag, but RRA should always clear Z flag
        0x1F => { let a = cpu.reg.a; cpu.reg.a = rr_op(&mut cpu.reg, a); cpu.reg.zero = false; }

        // LD n, d: load immediate into register n
        // Length: 2
        // Cycles: 8
        // Flags: - - - -
        0x06 => { cpu.reg.b = cpu.fetch() }
        0x0E => { cpu.reg.c = cpu.fetch() }
        0x16 => { cpu.reg.d = cpu.fetch() }
        0x1E => { cpu.reg.e = cpu.fetch() }
        0x26 => { cpu.reg.h = cpu.fetch() }
        0x2E => { cpu.reg.l = cpu.fetch() }
        0x3E => { cpu.reg.a = cpu.fetch() }

        // LD n, m: load value of register m into register n
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0x7F => {}                 // LD A,A
        0x78 => { cpu.reg.a = cpu.reg.b }  // LD A,B
        0x79 => { cpu.reg.a = cpu.reg.c }  // LD A,C
        0x7A => { cpu.reg.a = cpu.reg.d }  // LD A,D
        0x7B => { cpu.reg.a = cpu.reg.e }  // LD A,E
        0x7C => { cpu.reg.a = cpu.reg.h }  // LD A,H
        0x7D => { cpu.reg.a = cpu.reg.l }  // LD A,L

        0x47 => { cpu.reg.b = cpu.reg.a }  // LD B,A
        0x40 => {}                 // LD B,B
        0x41 => { cpu.reg.b = cpu.reg.c }  // LD B,C
        0x42 => { cpu.reg.b = cpu.reg.d }  // LD B,D
        0x43 => { cpu.reg.b = cpu.reg.e }  // LD B,E
        0x44 => { cpu.reg.b = cpu.reg.h }  // LD B,H
        0x45 => { cpu.reg.b = cpu.reg.l }  // LD B,L

        0x4F => { cpu.reg.c = cpu.reg.a }  // LD C,A
        0x48 => { cpu.reg.c = cpu.reg.b }  // LD C,B
        0x49 => {}                 // LD C,C
        0x4A => { cpu.reg.c = cpu.reg.d }  // LD C,D
        0x4B => { cpu.reg.c = cpu.reg.e }  // LD C,E
        0x4C => { cpu.reg.c = cpu.reg.h }  // LD C,H
        0x4D => { cpu.reg.c = cpu.reg.l }  // LD C,L

        0x57 => { cpu.reg.d = cpu.reg.a }  // LD D,A
        0x50 => { cpu.reg.d = cpu.reg.b }  // LD D,B
        0x51 => { cpu.reg.d = cpu.reg.c }  // LD D,C
        0x52 => {}                 // LD D,D
        0x53 => { cpu.reg.d = cpu.reg.e }  // LD D,E
        0x54 => { cpu.reg.d = cpu.reg.h }  // LD D,H
        0x55 => { cpu.reg.d = cpu.reg.l }  // LD D,L

        0x5F => { cpu.reg.e = cpu.reg.a }  // LD E,A
        0x58 => { cpu.reg.e = cpu.reg.b }  // LD E,B
        0x59 => { cpu.reg.e = cpu.reg.c }  // LD E,C
        0x5A => { cpu.reg.e = cpu.reg.d }  // LD E,D
        0x5B => {}                 // LD E,E
        0x5C => { cpu.reg.e = cpu.reg.h }  // LD E,H
        0x5D => { cpu.reg.e = cpu.reg.l }  // LD E,L

        0x67 => { cpu.reg.h = cpu.reg.a }  // LD H,A
        0x60 => { cpu.reg.h = cpu.reg.b }  // LD H,B
        0x61 => { cpu.reg.h = cpu.reg.c }  // LD H,C
        0x62 => { cpu.reg.h = cpu.reg.d }  // LD H,D
        0x63 => { cpu.reg.h = cpu.reg.e }  // LD H,E
        0x64 => {}                 // LD H,H
        0x65 => { cpu.reg.h = cpu.reg.l }  // LD H,L

        0x6F => { cpu.reg.l = cpu.reg.a }  // LD L,A
        0x68 => { cpu.reg.l = cpu.reg.b }  // LD L,B
        0x69 => { cpu.reg.l = cpu.reg.c }  // LD L,C
        0x6A => { cpu.reg.l = cpu.reg.d }  // LD L,D
        0x6B => { cpu.reg.l = cpu.reg.e }  // LD L,E
        0x6C => { cpu.reg.l = cpu.reg.h }  // LD L,H
        0x6D => {}                 // LD L,L

        // LD n, (hl): store value at (hl) in register n
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x46 => { let hl = cpu.reg.hl(); cpu.reg.b = cpu.read(hl) }
        0x4E => { let hl = cpu.reg.hl(); cpu.reg.c = cpu.read(hl) }
        0x56 => { let hl = cpu.reg.hl(); cpu.reg.d = cpu.read(hl) }
        0x5E => { let hl = cpu.reg.hl(); cpu.reg.e = cpu.read(hl) }
        0x66 => { let hl = cpu.reg.hl(); cpu.reg.h = cpu.read(hl) }
        0x6E => { let hl = cpu.reg.hl(); cpu.reg.l = cpu.read(hl) }
        0x7E => { let hl = cpu.reg.hl(); cpu.reg.a = cpu.read(hl) }

        // LD n, (mm): load value from memory into register n
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x0A => { let bc = cpu.reg.bc(); cpu.reg.a = cpu.read(bc) }
        0x1A => { let de = cpu.reg.de(); cpu.reg.a = cpu.read(de) }

        // LD ($FF00+n), A: Put A into memory address $FF00+n
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        0xE0 => {
            let n = cpu.fetch();
            let a = cpu.reg.a;
            cpu.write(0xFF00 + n as u16, a);
        }

        // LD A, ($FF00+n): read from memory $FF00+n to register A
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        0xF0 => {
            let n = cpu.fetch();
            cpu.reg.a = cpu.read(0xFF00 + n as u16);
        }

        // LD (HL), n: store register value to memory at address HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x70 => { let hl = cpu.reg.hl(); let b = cpu.reg.b; cpu.write(hl, b) }
        0x71 => { let hl = cpu.reg.hl(); let c = cpu.reg.c; cpu.write(hl, c) }
        0x72 => { let hl = cpu.reg.hl(); let d = cpu.reg.d; cpu.write(hl, d) }
        0x73 => { let hl = cpu.reg.hl(); let e = cpu.reg.e; cpu.write(hl, e) }
        0x74 => { let hl = cpu.reg.hl(); let h = cpu.reg.h; cpu.write(hl, h) }
        0x75 => { let hl = cpu.reg.hl(); let l = cpu.reg.l; cpu.write(hl, l) }
        0x77 => { let hl = cpu.reg.hl(); let a = cpu.reg.a; cpu.write(hl, a) }

        // RET: set PC to 16-bit value popped from stack
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        // TODO: placement of cpu.tick()?
        // TODO: why is RET 16 cycles when POP BC is 12 cycles?
        0xC9 => {
            cpu.reg.pc = pop_op(cpu);
            cpu.tick(4);
        }

        // RETI: set PC to 16-bit value popped from stack and enable IME
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        // TODO: placement of cpu.tick()?
        // TODO: why is RET 16 cycles when POP BC is 12 cycles?
        0xD9 => {
            cpu.reg.pc = pop_op(cpu);
            cpu.reg.ime = true;
            cpu.tick(4);
        }

        // RET Z: set PC to 16-bit value popped from stack if Z-flag is set
        // RET C: set PC to 16-bit value popped from stack if C-flag is set
        // RET NZ: set PC to 16-bit value popped from stack if Z-flag is *not* set
        // RET NC: set PC to 16-bit value popped from stack if C-flag is *not* set
        // Length: 1
        // Cycles: 20/8
        // Flags: - - - -
        // TODO: placement of cpu.tick()?
        0xC8 => { cpu.tick(4); if cpu.reg.zero { cpu.reg.pc = pop_op(cpu); cpu.tick(4); }}
        0xD8 => { cpu.tick(4); if cpu.reg.carry { cpu.reg.pc = pop_op(cpu); cpu.tick(4); }}
        0xC0 => { cpu.tick(4); if !cpu.reg.zero { cpu.reg.pc = pop_op(cpu); cpu.tick(4); }}
        0xD0 => { cpu.tick(4); if !cpu.reg.carry { cpu.reg.pc = pop_op(cpu); cpu.tick(4); }}

        // CALL a16: push address of next instruction on stack
        //           and jump to address a16
        // Length: 3
        // Cycles: 24
        // Flags: - - - -
        // TODO: placement of cpu.tick()?
        0xCD => {
            let to = cpu.fetch_u16();
            let pc = cpu.reg.pc;
            push_op(cpu, pc);
            cpu.reg.pc = to;
            cpu.tick(4);
        }

        // CALL NZ, a16: if Z-flag is not set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        // TODO: placement of cpu.tick()?
        0xC4 => {
            let to = cpu.fetch_u16();
            if !cpu.reg.zero {
                let pc = cpu.reg.pc;
                push_op(cpu, pc);
                cpu.reg.pc = to;
                cpu.tick(4);
            }
        }

        // CALL NC, a16: if C-flag is not set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        // TODO: placement of cpu.tick()?
        0xD4 => {
            let to = cpu.fetch_u16();
            if !cpu.reg.carry {
                let pc = cpu.reg.pc;
                push_op(cpu, pc);
                cpu.reg.pc = to;
                cpu.tick(4);
            }
        }

        // CALL Z, a16: if Z-flag is set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        0xCC => {
            let to = cpu.fetch_u16();
            if cpu.reg.zero {
                let pc = cpu.reg.pc;
                push_op(cpu, pc);
                cpu.reg.pc = to;
                cpu.tick(4);
            }
        }

        // CALL C, a16: if C-flag is set, push address of next
        //               instruction on stack and jump to address a16
        // Length: 3
        // Cycles: 24/12
        // Flags: - - - -
        0xDC => {
            let to = cpu.fetch_u16();
            if cpu.reg.carry {
                let pc = cpu.reg.pc;
                push_op(cpu, pc);
                cpu.reg.pc = to;
                cpu.tick(4);
            }
        }

        // RST n: push PC and jump to one out of 8 possible addresses
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        0xC7 => { rst_op(cpu, 0x0000); }
        0xCF => { rst_op(cpu, 0x0008); }
        0xD7 => { rst_op(cpu, 0x0010); }
        0xDF => { rst_op(cpu, 0x0018); }
        0xE7 => { rst_op(cpu, 0x0020); }
        0xEF => { rst_op(cpu, 0x0028); }
        0xF7 => { rst_op(cpu, 0x0030); }
        0xFF => { rst_op(cpu, 0x0038); }

        // PUSH nn: push 16-bit register nn to stack
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        0xC5 => { let bc = cpu.reg.bc(); push_op(cpu, bc); cpu.tick(4); }
        0xD5 => { let de = cpu.reg.de(); push_op(cpu, de); cpu.tick(4); }
        0xE5 => { let hl = cpu.reg.hl(); push_op(cpu, hl); cpu.tick(4); }
        0xF5 => { let af = cpu.reg.af(); push_op(cpu, af); cpu.tick(4); }

        // POP nn: pop value from stack and store in 16-bit register nn
        // Length: 1
        // Cycles: 12
        // Flags: - - - -
        0xC1 => { let v = pop_op(cpu); cpu.reg.set_bc(v); }
        0xD1 => { let v = pop_op(cpu); cpu.reg.set_de(v); }
        0xE1 => { let v = pop_op(cpu); cpu.reg.set_hl(v); }
        0xF1 => { let v = pop_op(cpu); cpu.reg.set_af(v); }

        0xE2 => {
            // LD ($FF00+C), A: put value of A in address 0xFF00 + C
            // Length: 1
            // Cycles: 8
            // Flags: - - - -
            // Note: The opcode table at pastraiser.com specify
            // invalid length of 2. The correct length is 1.
            let addr = 0xFF00 + cpu.reg.c as u16;
            let a = cpu.reg.a;
            cpu.write(addr, a);
        }

        // LD A, ($FF00+C): store value at address 0xFF00 + C in A 
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0xF2 => {
            let addr = 0xFF00 + cpu.reg.c as u16;
            cpu.reg.a = cpu.read(addr);
        }

        // JR d8: relative jump
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        // TODO: placement of cpu.tick()?
        0x18 => {
            let offs = cpu.fetch() as i8;

            cpu.reg.pc = if offs >= 0 {
                cpu.reg.pc.wrapping_add(offs as u16)
            } else {
                cpu.reg.pc.wrapping_sub(-offs as u16)
            };

            cpu.tick(4);
        }

        // JR NZ, d8: jump d8 relative to PC if Z flag is not set
        // Length: 2
        // Cycles: 12/8
        // Flags: - - - -
        0x20 => {
            let offs = cpu.fetch() as i8;
            if !cpu.reg.zero {
                cpu.reg.pc = if offs >= 0 {
                    cpu.reg.pc.wrapping_add(offs as u16)
                } else {
                    cpu.reg.pc.wrapping_sub(-offs as u16)
                };
                cpu.tick(4);
            }
        }

        // JR NC, d8: jump d8 relative to PC if C flag is not set
        // Length: 2
        // Cycles: 12/8
        // Flags: - - - -
        0x30 => {
            let offs = cpu.fetch() as i8;
            if !cpu.reg.carry {
                cpu.reg.pc = if offs >= 0 {
                    cpu.reg.pc.wrapping_add(offs as u16)
                } else {
                    cpu.reg.pc.wrapping_sub(-offs as u16)
                };
                cpu.tick(4);
            }
        }

        // JR Z, d8: jump d8 relative to PC if Z is set
        // Length: 2
        // Cycles: 12/8
        // Flags: - - - -
        0x28 => {
            let offs = cpu.fetch() as i8;
            if cpu.reg.zero {
                cpu.reg.pc = if offs >= 0 {
                    cpu.reg.pc.wrapping_add(offs as u16)
                } else {
                    cpu.reg.pc.wrapping_sub(-offs as u16)
                };
                cpu.tick(4);
            }
        }

        0x38 => {
            // JR C, d8: jump d8 relative to PC if C is set
            // Length: 2
            // Cycles: 12/8
            // Flags: - - - -
            let offs = cpu.fetch() as i8;

            if cpu.reg.carry {
                cpu.reg.pc = if offs >= 0 {
                    cpu.reg.pc.wrapping_add(offs as u16)
                } else {
                    cpu.reg.pc.wrapping_sub(-offs as u16)
                };

                cpu.tick(4);
            }
        }

        // JP NZ, a16: jump to address a16 if Z is *not* set
        // JP Z, a16: jump to address a16 if Z is set
        // Length: 3
        // Cycles: 16/12
        // Flags: - - - -
        0xC2 => {
            let to = cpu.fetch_u16();
            if !cpu.reg.zero {
                cpu.reg.pc = to;
                cpu.tick(4);
            }
        }
        0xCA => {
            let to = cpu.fetch_u16();
            if cpu.reg.zero {
                cpu.reg.pc = to;
                cpu.tick(4);
            }
        }

        // JP NC, a16: jump to address a16 if C is *not* set
        // JP C, a16: jump to address a16 if C is set
        // Length: 3
        // Cycles: 16/12
        // Flags: - - - -
        0xD2 => {
            let to = cpu.fetch_u16();
            if !cpu.reg.carry {
                cpu.reg.pc = to;
                cpu.tick(4);
            }
        }
        0xDA => {
            let to = cpu.fetch_u16();
            if cpu.reg.carry {
                cpu.reg.pc = to;
                cpu.tick(4);
            }        
        }

        // JP a16: jump to immediate address
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xC3 => {
            cpu.reg.pc = cpu.fetch_u16();
            cpu.tick(4);
        }

        // JP (HL): jump to address HL, or in other words: PC = HL
        // Note that this op does *not* set PC to the value stored in memory
        // at address (HL)!
        // Length: 1
        // Cycles: 4
        // Flags: - - - -
        0xE9 => {
            cpu.reg.pc = cpu.reg.hl();
        }

        0xF9 => {
            // LD SP, HL: set HL to value of SP
            // Length: 1
            // Cycles: 8
            // Flags: - - - -
            cpu.reg.sp = cpu.reg.hl();
            cpu.tick(4);
        }

        // LD (HL-), A: put A into memory address HL, decrement HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x32 => {
            let hl = cpu.reg.hl();
            let a = cpu.reg.a;
            cpu.write(hl, a);
            cpu.reg.set_hl(hl.wrapping_sub(1));
        }

        // XOR N: assign A xor N to A
        // Length: 1
        // Cycles: 4 (8 for op 0xAE)
        // Flags: Z 0 0 0
        0xA8 => { let b = cpu.reg.b; xor_op(&mut cpu.reg, b); }
        0xA9 => { let c = cpu.reg.c; xor_op(&mut cpu.reg, c); }
        0xAA => { let d = cpu.reg.d; xor_op(&mut cpu.reg, d); }
        0xAB => { let e = cpu.reg.e; xor_op(&mut cpu.reg, e); }
        0xAC => { let h = cpu.reg.h; xor_op(&mut cpu.reg, h); }
        0xAD => { let l = cpu.reg.l; xor_op(&mut cpu.reg, l); }
        0xAE => { let hl = cpu.reg.hl(); let v = cpu.read(hl); xor_op(&mut cpu.reg, v); }
        0xAF => { let a = cpu.reg.a; xor_op(&mut cpu.reg, a); }

        // XOR d8: assign A xor d8 to A
        // Length: 2
        // Cycles: 8
        // Flags: Z 0 0 0
        0xEE => {
            let v = cpu.fetch();
            xor_op(&mut cpu.reg, v);
        }

        // RLA: Rotate the contents of register A to the left
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        0x17 => {
            let b0 = if cpu.reg.carry { 1 } else { 0 };
            let b8 = cpu.reg.a & 128 != 0;
            cpu.reg.set_znhc(false, false, false, b8);
            cpu.reg.a = cpu.reg.a << 1 | b0;
        }

        // LD (HL+), A: store value of A at (HL) and increment HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        // Alt mnemonic 1: LD (HLI), A
        // Alt mnemonic 2: LDI (HL), A
        0x22 => {
            let hl = cpu.reg.hl();
            let a = cpu.reg.a;
            cpu.write(hl, a);
            cpu.reg.set_hl(hl + 1);
        }

        // LD (HL), d8: store immediate value at (HL)
        // Length: 2
        // Cycles: 12
        // Flags: - - - -
        0x36 => {
            let v = cpu.fetch();
            let hl = cpu.reg.hl();
            cpu.write(hl, v);
        }

        // LD A, (HL+): load value from (HL) to A and increment HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x2A => {
            let hl = cpu.reg.hl();
            cpu.reg.a = cpu.read(hl);
            cpu.reg.set_hl(hl + 1);
        }

        // LD A, (HL-): load value from (HL) to A and decrement HL
        // Length: 1
        // Cycles: 8
        // Flags: - - - -
        0x3A => {
            let hl = cpu.reg.hl();
            cpu.reg.a = cpu.read(hl);
            cpu.reg.set_hl(hl.wrapping_sub(1));
        }

        // LD (a16), A: store value of A at address a16
        // Length: 3
        // Cycles: 16
        // Flags: - - - -
        0xEA => {
            let addr = cpu.fetch_u16();
            let a = cpu.reg.a;
            cpu.write(addr, a);
        }

        // LD (a16), SP: store SP at address (a16)
        // Length: 3
        // Cycles: 20
        // Flags: - - - -
        0x08 => {
            let addr = cpu.fetch_u16();
            let sp = cpu.reg.sp;
            cpu.write_u16(addr, sp);
        }

        // LD HL, SP+d8: load HL with value of SP + immediate value r8
        // Alt syntax: LDHL SP, d8
        // Length: 2
        // Cycles: 12
        // Flags: 0 0 H C
        // TODO: placement of cpu.tick()?
        0xF8 => {
            // let value = mem.read_i8(reg.pc + 1) as u16;
            let value = cpu.fetch() as i8 as u16;
            cpu.reg.zero = false;
            cpu.reg.neg = false;
            cpu.reg.half_carry = ((cpu.reg.sp & 0x0F) + (value & 0x0F)) > 0x0F;
            cpu.reg.carry = (cpu.reg.sp & 0xFF) + (value & 0xFF) > 0xFF;
            let hl = cpu.reg.sp.wrapping_add(value);
            cpu.reg.set_hl(hl);
            cpu.tick(4);
        }

        // CP r, CP (hl): Compare r (or value at (hl)) with A. Same as SUB but throws away the result
        // Length: 1
        // Cycles: 4 (8 for "CP (hl)")
        // Flags: Z 1 H C
        0xB8 => { let b = cpu.reg.b; cp_op(&mut cpu.reg, b); }
        0xB9 => { let c = cpu.reg.c; cp_op(&mut cpu.reg, c); }
        0xBA => { let d = cpu.reg.d; cp_op(&mut cpu.reg, d); }
        0xBB => { let e = cpu.reg.e; cp_op(&mut cpu.reg, e); }
        0xBC => { let h = cpu.reg.h; cp_op(&mut cpu.reg, h); }
        0xBD => { let l = cpu.reg.l; cp_op(&mut cpu.reg, l); }
        0xBE => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cp_op(&mut cpu.reg, v); }
        0xBF => { cpu.reg.set_znhc(true, true, false, false); }

        // CP u8: Compare A with immediate
        // Length: 2
        // Cycles: 8
        // Flags: Z 1 H C
        0xFE => { let v = cpu.fetch(); cp_op(&mut cpu.reg, v); }

        0xF3 => {
            // DI: Disable Interrupt Master Enable Flag, prohibits maskable interrupts
            // Length: 1
            // Cycles: 4
            // Flags: - - - -
            cpu.reg.ime = false;
        }

        0xFB => {
            // DI: Enable Interrupt Master Enable Flag
            // Length: 1
            // Cycles: 4
            // Flags: - - - -
            cpu.reg.ime = true;
        }

        // RLCA: rotate content of register A left, with carry
        // Length: 1
        // Cycles: 4
        // Flags: 0 0 0 C
        0x07 => {
            // FIXME: don't we have multiple impl of this?
            let a = (cpu.reg.a as u32) << 1;
            if a > 0xFF {
                cpu.reg.a = (a & 0xFF) as u8 | 1;
                cpu.reg.set_znhc(false, false, false, true);
            } else {
                cpu.reg.a = (a & 0xFF) as u8;
                cpu.reg.set_znhc(false, false, false, false);
            }
        }

        // CPL: complement (bitwise not) register A
        // Length: 1
        // Cycles: 4
        // Flags: - 1 1 -
        0x2F => {
            cpu.reg.a = !cpu.reg.a;
            cpu.reg.neg = true;
            cpu.reg.half_carry = true;
        }

        // CCF: Flip carry flag
        // Length: 1
        // Cycles: 4
        // Flags: - 0 0 C
        0x3F => {
            cpu.reg.carry = !cpu.reg.carry;
            cpu.reg.half_carry = false;
            cpu.reg.neg = false;
        }

        // STOP 0
        // Length: 1 (not 2, see https://stackoverflow.com/questions/41353869)
        // Cycles: 4
        0x10 => {
            cpu.reg.stopped = true;
        }

        // Prefix 0xCB instructions
        // All 0xCB operations have length 2
        // All 0xCB operations consume 8 cycles, except for
        // all operations with op code 0x*6 and 0x*E which
        // consume 16 cycles.
        0xCB => {
            let op2 = cpu.fetch();
            match op2 {
                // RLC n: rotate register n left
                // Length: 
                0x00 => { let b = cpu.reg.b; cpu.reg.b = rlc_op(&mut cpu.reg, b); }
                0x01 => { let c = cpu.reg.c; cpu.reg.c = rlc_op(&mut cpu.reg, c); }
                0x02 => { let d = cpu.reg.d; cpu.reg.d = rlc_op(&mut cpu.reg, d); }
                0x03 => { let e = cpu.reg.e; cpu.reg.e = rlc_op(&mut cpu.reg, e); }
                0x04 => { let h = cpu.reg.h; cpu.reg.h = rlc_op(&mut cpu.reg, h); }
                0x05 => { let l = cpu.reg.l; cpu.reg.l = rlc_op(&mut cpu.reg, l); }
                0x06 => {
                    let hl = cpu.reg.hl();
                    let v = cpu.read(hl);
                    let rot = rlc_op(&mut cpu.reg, v);
                    cpu.write(hl, rot);
                }
                0x07 => { let a = cpu.reg.a; cpu.reg.a = rlc_op(&mut cpu.reg, a); }

                // RLC n: rotate register n right
                0x08 => { let b = cpu.reg.b; cpu.reg.b = rrc_op(&mut cpu.reg, b); }
                0x09 => { let c = cpu.reg.c; cpu.reg.c = rrc_op(&mut cpu.reg, c); }
                0x0A => { let d = cpu.reg.d; cpu.reg.d = rrc_op(&mut cpu.reg, d); }
                0x0B => { let e = cpu.reg.e; cpu.reg.e = rrc_op(&mut cpu.reg, e); }
                0x0C => { let h = cpu.reg.h; cpu.reg.h = rrc_op(&mut cpu.reg, h); }
                0x0D => { let l = cpu.reg.l; cpu.reg.l = rrc_op(&mut cpu.reg, l); }
                0x0E => {
                    let hl = cpu.reg.hl();
                    let v = cpu.read(hl);
                    let rot = rrc_op(&mut cpu.reg, v);
                    cpu.write(hl, rot);
                }
                0x0F => { let a = cpu.reg.a; cpu.reg.a = rrc_op(&mut cpu.reg, a); }

                // RL n: rotate register n left with carry flag
                0x10 => { let b = cpu.reg.b; cpu.reg.b = rl_op(&mut cpu.reg, b); }
                0x11 => { let c = cpu.reg.c; cpu.reg.c = rl_op(&mut cpu.reg, c); }
                0x12 => { let d = cpu.reg.d; cpu.reg.d = rl_op(&mut cpu.reg, d); }
                0x13 => { let e = cpu.reg.e; cpu.reg.e = rl_op(&mut cpu.reg, e); }
                0x14 => { let h = cpu.reg.h; cpu.reg.h = rl_op(&mut cpu.reg, h); }
                0x15 => { let l = cpu.reg.l; cpu.reg.l = rl_op(&mut cpu.reg, l); }
                0x16 => {
                    let hl = cpu.reg.hl();
                    let v = cpu.read(hl);
                    let rot = rl_op(&mut cpu.reg, v);
                    cpu.write(hl, rot);
                }
                0x17 => { let a = cpu.reg.a; cpu.reg.a = rl_op(&mut cpu.reg, a); }

                // RR n, rotate register n right with carry flag
                0x18 => { let b = cpu.reg.b; cpu.reg.b = rr_op(&mut cpu.reg, b) }
                0x19 => { let c = cpu.reg.c; cpu.reg.c = rr_op(&mut cpu.reg, c) }
                0x1A => { let d = cpu.reg.d; cpu.reg.d = rr_op(&mut cpu.reg, d) }
                0x1B => { let e = cpu.reg.e; cpu.reg.e = rr_op(&mut cpu.reg, e) }
                0x1C => { let h = cpu.reg.h; cpu.reg.h = rr_op(&mut cpu.reg, h) }
                0x1D => { let l = cpu.reg.l; cpu.reg.l = rr_op(&mut cpu.reg, l) }
                0x1E => {
                    let hl = cpu.reg.hl();
                    let v = cpu.read(hl);
                    let rot = rr_op(&mut cpu.reg, v);
                    cpu.write(hl, rot);
                }
                0x1F => { let a = cpu.reg.a; cpu.reg.a = rr_op(&mut cpu.reg, a) }

                // SLA r
                0x20 => { let b = cpu.reg.b; cpu.reg.b = sla_op(&mut cpu.reg, b) }
                0x21 => { let c = cpu.reg.c; cpu.reg.c = sla_op(&mut cpu.reg, c) }
                0x22 => { let d = cpu.reg.d; cpu.reg.d = sla_op(&mut cpu.reg, d) }
                0x23 => { let e = cpu.reg.e; cpu.reg.e = sla_op(&mut cpu.reg, e) }
                0x24 => { let h = cpu.reg.h; cpu.reg.h = sla_op(&mut cpu.reg, h) }
                0x25 => { let l = cpu.reg.l; cpu.reg.l = sla_op(&mut cpu.reg, l) }
                0x26 => {
                    let hl = cpu.reg.hl();
                    let v = cpu.read(hl);
                    let result = sla_op(&mut cpu.reg, v);
                    cpu.write(hl, result);
                }
                0x27 => { let a = cpu.reg.a; cpu.reg.a = sla_op(&mut cpu.reg, a) }

                // SRA r
                0x28 => { let b = cpu.reg.b; cpu.reg.b = sra_op(&mut cpu.reg, b) }
                0x29 => { let c = cpu.reg.c; cpu.reg.c = sra_op(&mut cpu.reg, c) }
                0x2A => { let d = cpu.reg.d; cpu.reg.d = sra_op(&mut cpu.reg, d) }
                0x2B => { let e = cpu.reg.e; cpu.reg.e = sra_op(&mut cpu.reg, e) }
                0x2C => { let h = cpu.reg.h; cpu.reg.h = sra_op(&mut cpu.reg, h) }
                0x2D => { let l = cpu.reg.l; cpu.reg.l = sra_op(&mut cpu.reg, l) }
                0x2E => {
                    let hl = cpu.reg.hl();
                    let v = cpu.read(hl);
                    let result = sra_op(&mut cpu.reg, v);
                    cpu.write(hl, result);
                }
                0x2F => { let a = cpu.reg.a; cpu.reg.a = sra_op(&mut cpu.reg, a) }

                // SWAP r
                0x30 => { let b = cpu.reg.b; cpu.reg.b = swap_op(&mut cpu.reg, b) }
                0x31 => { let c = cpu.reg.c; cpu.reg.c = swap_op(&mut cpu.reg, c) }
                0x32 => { let d = cpu.reg.d; cpu.reg.d = swap_op(&mut cpu.reg, d) }
                0x33 => { let e = cpu.reg.e; cpu.reg.e = swap_op(&mut cpu.reg, e) }
                0x34 => { let h = cpu.reg.h; cpu.reg.h = swap_op(&mut cpu.reg, h) }
                0x35 => { let l = cpu.reg.l; cpu.reg.l = swap_op(&mut cpu.reg, l) }
                0x36 => {
                    let hl = cpu.reg.hl();
                    let v = cpu.read(hl);
                    let result = swap_op(&mut cpu.reg, v);
                    cpu.write(hl, result);
                }
                0x37 => { let a = cpu.reg.a; cpu.reg.a = swap_op(&mut cpu.reg, a) }

                // SRL r
                0x38 => { let b = cpu.reg.b; cpu.reg.b = srl_op(&mut cpu.reg, b) }
                0x39 => { let c = cpu.reg.c; cpu.reg.c = srl_op(&mut cpu.reg, c) }
                0x3A => { let d = cpu.reg.d; cpu.reg.d = srl_op(&mut cpu.reg, d) }
                0x3B => { let e = cpu.reg.e; cpu.reg.e = srl_op(&mut cpu.reg, e) }
                0x3C => { let h = cpu.reg.h; cpu.reg.h = srl_op(&mut cpu.reg, h) }
                0x3D => { let l = cpu.reg.l; cpu.reg.l = srl_op(&mut cpu.reg, l) }
                0x3E => {
                    let hl = cpu.reg.hl();
                    let v = cpu.read(hl);
                    let result = srl_op(&mut cpu.reg, v);
                    cpu.write(hl, result);
                }
                0x3F => { let a = cpu.reg.a; cpu.reg.a = srl_op(&mut cpu.reg, a) }

                // BIT b, r: test if bit 'b' in register 'r' is set
                // Flags: Z 0 1 -
                // TODO: does op 0x46, 0x4E, etc really consume 16 cycles?
                0x40 => { let b = cpu.reg.b; bit_op(&mut cpu.reg, 0, b); }
                0x41 => { let c = cpu.reg.c; bit_op(&mut cpu.reg, 0, c); }
                0x42 => { let d = cpu.reg.d; bit_op(&mut cpu.reg, 0, d); }
                0x43 => { let e = cpu.reg.e; bit_op(&mut cpu.reg, 0, e); }
                0x44 => { let h = cpu.reg.h; bit_op(&mut cpu.reg, 0, h); }
                0x45 => { let l = cpu.reg.l; bit_op(&mut cpu.reg, 0, l); }
                0x46 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); bit_op(&mut cpu.reg, 0, v) }
                0x47 => { let a = cpu.reg.a; bit_op(&mut cpu.reg, 0, a); }

                0x48 => { let b = cpu.reg.b; bit_op(&mut cpu.reg, 1, b); }
                0x49 => { let c = cpu.reg.c; bit_op(&mut cpu.reg, 1, c); }
                0x4A => { let d = cpu.reg.d; bit_op(&mut cpu.reg, 1, d); }
                0x4B => { let e = cpu.reg.e; bit_op(&mut cpu.reg, 1, e); }
                0x4C => { let h = cpu.reg.h; bit_op(&mut cpu.reg, 1, h); }
                0x4D => { let l = cpu.reg.l; bit_op(&mut cpu.reg, 1, l); }
                0x4E => { let hl = cpu.reg.hl(); let v = cpu.read(hl); bit_op(&mut cpu.reg, 1, v) }
                0x4F => { let a = cpu.reg.a; bit_op(&mut cpu.reg, 1, a); }

                0x50 => { let b = cpu.reg.b; bit_op(&mut cpu.reg, 2, b); }
                0x51 => { let c = cpu.reg.c; bit_op(&mut cpu.reg, 2, c); }
                0x52 => { let d = cpu.reg.d; bit_op(&mut cpu.reg, 2, d); }
                0x53 => { let e = cpu.reg.e; bit_op(&mut cpu.reg, 2, e); }
                0x54 => { let h = cpu.reg.h; bit_op(&mut cpu.reg, 2, h); }
                0x55 => { let l = cpu.reg.l; bit_op(&mut cpu.reg, 2, l); }
                0x56 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); bit_op(&mut cpu.reg, 2, v) }
                0x57 => { let a = cpu.reg.a; bit_op(&mut cpu.reg, 2, a); }

                0x58 => { let b = cpu.reg.b; bit_op(&mut cpu.reg, 3, b); }
                0x59 => { let c = cpu.reg.c; bit_op(&mut cpu.reg, 3, c); }
                0x5A => { let d = cpu.reg.d; bit_op(&mut cpu.reg, 3, d); }
                0x5B => { let e = cpu.reg.e; bit_op(&mut cpu.reg, 3, e); }
                0x5C => { let h = cpu.reg.h; bit_op(&mut cpu.reg, 3, h); }
                0x5D => { let l = cpu.reg.l; bit_op(&mut cpu.reg, 3, l); }
                0x5E => { let hl = cpu.reg.hl(); let v = cpu.read(hl); bit_op(&mut cpu.reg, 3, v) }
                0x5F => { let a = cpu.reg.a; bit_op(&mut cpu.reg, 3, a); }

                0x60 => { let b = cpu.reg.b; bit_op(&mut cpu.reg, 4, b); }
                0x61 => { let c = cpu.reg.c; bit_op(&mut cpu.reg, 4, c); }
                0x62 => { let d = cpu.reg.d; bit_op(&mut cpu.reg, 4, d); }
                0x63 => { let e = cpu.reg.e; bit_op(&mut cpu.reg, 4, e); }
                0x64 => { let h = cpu.reg.h; bit_op(&mut cpu.reg, 4, h); }
                0x65 => { let l = cpu.reg.l; bit_op(&mut cpu.reg, 4, l); }
                0x66 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); bit_op(&mut cpu.reg, 4, v) }
                0x67 => { let a = cpu.reg.a; bit_op(&mut cpu.reg, 4, a); }

                0x68 => { let b = cpu.reg.b; bit_op(&mut cpu.reg, 5, b); }
                0x69 => { let c = cpu.reg.c; bit_op(&mut cpu.reg, 5, c); }
                0x6A => { let d = cpu.reg.d; bit_op(&mut cpu.reg, 5, d); }
                0x6B => { let e = cpu.reg.e; bit_op(&mut cpu.reg, 5, e); }
                0x6C => { let h = cpu.reg.h; bit_op(&mut cpu.reg, 5, h); }
                0x6D => { let l = cpu.reg.l; bit_op(&mut cpu.reg, 5, l); }
                0x6E => { let hl = cpu.reg.hl(); let v = cpu.read(hl); bit_op(&mut cpu.reg, 5, v) }
                0x6F => { let a = cpu.reg.a; bit_op(&mut cpu.reg, 5, a); }

                0x70 => { let b = cpu.reg.b; bit_op(&mut cpu.reg, 6, b); }
                0x71 => { let c = cpu.reg.c; bit_op(&mut cpu.reg, 6, c); }
                0x72 => { let d = cpu.reg.d; bit_op(&mut cpu.reg, 6, d); }
                0x73 => { let e = cpu.reg.e; bit_op(&mut cpu.reg, 6, e); }
                0x74 => { let h = cpu.reg.h; bit_op(&mut cpu.reg, 6, h); }
                0x75 => { let l = cpu.reg.l; bit_op(&mut cpu.reg, 6, l); }
                0x76 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); bit_op(&mut cpu.reg, 6, v) }
                0x77 => { let a = cpu.reg.a; bit_op(&mut cpu.reg, 6, a); }

                0x78 => { let b = cpu.reg.b; bit_op(&mut cpu.reg, 7, b); }
                0x79 => { let c = cpu.reg.c; bit_op(&mut cpu.reg, 7, c); }
                0x7A => { let d = cpu.reg.d; bit_op(&mut cpu.reg, 7, d); }
                0x7B => { let e = cpu.reg.e; bit_op(&mut cpu.reg, 7, e); }
                0x7C => { let h = cpu.reg.h; bit_op(&mut cpu.reg, 7, h); }
                0x7D => { let l = cpu.reg.l; bit_op(&mut cpu.reg, 7, l); }
                0x7E => { let hl = cpu.reg.hl(); let v = cpu.read(hl); bit_op(&mut cpu.reg, 7, v) }
                0x7F => { let a = cpu.reg.a; bit_op(&mut cpu.reg, 7, a); }

                // RES b, r: reset bit b in register r
                // Length: 2
                // Cycles: 8
                // Flags: - - - -
                0x80 => { cpu.reg.b &= !1; }
                0x81 => { cpu.reg.c &= !1; }
                0x82 => { cpu.reg.d &= !1; }
                0x83 => { cpu.reg.e &= !1; }
                0x84 => { cpu.reg.h &= !1; }
                0x85 => { cpu.reg.l &= !1; }
                0x86 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v & !1); }
                0x87 => { cpu.reg.a &= !1; }

                0x88 => { cpu.reg.b &= !2; }
                0x89 => { cpu.reg.c &= !2; }
                0x8A => { cpu.reg.d &= !2; }
                0x8B => { cpu.reg.e &= !2; }
                0x8C => { cpu.reg.h &= !2; }
                0x8D => { cpu.reg.l &= !2; }
                0x8E => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v & !2); }
                0x8F => { cpu.reg.a &= !2; }

                0x90 => { cpu.reg.b &= !4; }
                0x91 => { cpu.reg.c &= !4; }
                0x92 => { cpu.reg.d &= !4; }
                0x93 => { cpu.reg.e &= !4; }
                0x94 => { cpu.reg.h &= !4; }
                0x95 => { cpu.reg.l &= !4; }
                0x96 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v & !4); }
                0x97 => { cpu.reg.a &= !4; }

                0x98 => { cpu.reg.b &= !8; }
                0x99 => { cpu.reg.c &= !8; }
                0x9A => { cpu.reg.d &= !8; }
                0x9B => { cpu.reg.e &= !8; }
                0x9C => { cpu.reg.h &= !8; }
                0x9D => { cpu.reg.l &= !8; }
                0x9E => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v & !8); }
                0x9F => { cpu.reg.a &= !8; }

                0xA0 => { cpu.reg.b &= !16; }
                0xA1 => { cpu.reg.c &= !16; }
                0xA2 => { cpu.reg.d &= !16; }
                0xA3 => { cpu.reg.e &= !16; }
                0xA4 => { cpu.reg.h &= !16; }
                0xA5 => { cpu.reg.l &= !16; }
                0xA6 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v & !16); }
                0xA7 => { cpu.reg.a &= !16; }

                0xA8 => { cpu.reg.b &= !32; }
                0xA9 => { cpu.reg.c &= !32; }
                0xAA => { cpu.reg.d &= !32; }
                0xAB => { cpu.reg.e &= !32; }
                0xAC => { cpu.reg.h &= !32; }
                0xAD => { cpu.reg.l &= !32; }
                0xAE => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v & !32); }
                0xAF => { cpu.reg.a &= !32; }

                0xB0 => { cpu.reg.b &= !64; }
                0xB1 => { cpu.reg.c &= !64; }
                0xB2 => { cpu.reg.d &= !64; }
                0xB3 => { cpu.reg.e &= !64; }
                0xB4 => { cpu.reg.h &= !64; }
                0xB5 => { cpu.reg.l &= !64; }
                0xB6 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v & !64); }
                0xB7 => { cpu.reg.a &= !64; }

                0xB8 => { cpu.reg.b &= !128; }
                0xB9 => { cpu.reg.c &= !128; }
                0xBA => { cpu.reg.d &= !128; }
                0xBB => { cpu.reg.e &= !128; }
                0xBC => { cpu.reg.h &= !128; }
                0xBD => { cpu.reg.l &= !128; }
                0xBE => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v & !128); }
                0xBF => { cpu.reg.a &= !128; }

                // SET b, r: set bit b in register r
                // Flags: - - - -
                0xC0 => { cpu.reg.b |= 1; }
                0xC1 => { cpu.reg.c |= 1; }
                0xC2 => { cpu.reg.d |= 1; }
                0xC3 => { cpu.reg.e |= 1; }
                0xC4 => { cpu.reg.h |= 1; }
                0xC5 => { cpu.reg.l |= 1; }
                0xC6 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v | 1); }
                0xC7 => { cpu.reg.a |= 1; }

                0xC8 => { cpu.reg.b |= 2; }
                0xC9 => { cpu.reg.c |= 2; }
                0xCA => { cpu.reg.d |= 2; }
                0xCB => { cpu.reg.e |= 2; }
                0xCC => { cpu.reg.h |= 2; }
                0xCD => { cpu.reg.l |= 2; }
                0xCE => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v | 2); }
                0xCF => { cpu.reg.a |= 2; }

                0xD0 => { cpu.reg.b |= 4; }
                0xD1 => { cpu.reg.c |= 4; }
                0xD2 => { cpu.reg.d |= 4; }
                0xD3 => { cpu.reg.e |= 4; }
                0xD4 => { cpu.reg.h |= 4; }
                0xD5 => { cpu.reg.l |= 4; }
                0xD6 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v | 4); }
                0xD7 => { cpu.reg.a |= 4; }

                0xD8 => { cpu.reg.b |= 8; }
                0xD9 => { cpu.reg.c |= 8; }
                0xDA => { cpu.reg.d |= 8; }
                0xDB => { cpu.reg.e |= 8; }
                0xDC => { cpu.reg.h |= 8; }
                0xDD => { cpu.reg.l |= 8; }
                0xDE => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v | 8); }
                0xDF => { cpu.reg.a |= 8; }

                0xE0 => { cpu.reg.b |= 16; }
                0xE1 => { cpu.reg.c |= 16; }
                0xE2 => { cpu.reg.d |= 16; }
                0xE3 => { cpu.reg.e |= 16; }
                0xE4 => { cpu.reg.h |= 16; }
                0xE5 => { cpu.reg.l |= 16; }
                0xE6 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v | 16); }
                0xE7 => { cpu.reg.a |= 16; }

                0xE8 => { cpu.reg.b |= 32; }
                0xE9 => { cpu.reg.c |= 32; }
                0xEA => { cpu.reg.d |= 32; }
                0xEB => { cpu.reg.e |= 32; }
                0xEC => { cpu.reg.h |= 32; }
                0xED => { cpu.reg.l |= 32; }
                0xEE => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v | 32); }
                0xEF => { cpu.reg.a |= 32; }

                0xF0 => { cpu.reg.b |= 64; }
                0xF1 => { cpu.reg.c |= 64; }
                0xF2 => { cpu.reg.d |= 64; }
                0xF3 => { cpu.reg.e |= 64; }
                0xF4 => { cpu.reg.h |= 64; }
                0xF5 => { cpu.reg.l |= 64; }
                0xF6 => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v | 64); }
                0xF7 => { cpu.reg.a |= 64; }

                0xF8 => { cpu.reg.b |= 128; }
                0xF9 => { cpu.reg.c |= 128; }
                0xFA => { cpu.reg.d |= 128; }
                0xFB => { cpu.reg.e |= 128; }
                0xFC => { cpu.reg.h |= 128; }
                0xFD => { cpu.reg.l |= 128; }
                0xFE => { let hl = cpu.reg.hl(); let v = cpu.read(hl); cpu.write(hl, v | 128); }
                0xFF => { cpu.reg.a |= 128; }

                _ => {
                    panic!("Unsupported opcode at 0x{:04X}: 0x{:02X}{:02X}", cpu.reg.pc, op, op2);
                }
            }
        }

        _ => {
            panic!("Unsupported opcode at 0x{:04X}: 0x{:02X}", cpu.reg.pc, op);
        }
    }
}

#[cfg(test)]
mod tests {
    use instructions::*;
    use debug::*;

    fn build_cpu() -> Cpu {
        let mut cpu = Cpu::new();
        cpu.reg.pc = 0x1000;
        cpu.reg.sp = 0xFFFC;
        cpu
    }

    #[test]
    fn test_instruction_length() {
        // Test that PC increments correctly depending
        // on instruction length
        for i in 0..255 {
            // 0x76 = HALT (not implemented yet)
            if i == 0x76 { continue };
            let mut cpu = build_cpu();
            cpu.reg.pc = 0x1000;
            cpu.reg.zero = false;
            cpu.reg.carry = false;
            cpu.mem.mem[0x1000] = i as u8;
            cpu.mem.mem[0x1001] = 0x12;
            cpu.mem.mem[0x1002] = 0x34;
            cpu.mem.mem[0xFFFC] = 0x56;
            cpu.mem.mem[0xFFFD] = 0x78;
            cpu.exec_op();
            println!("Op: 0x{:02X}", i);
            match i {
                0x18 => {
                    // JR r8
                    assert_eq!(cpu.reg.pc, 0x1014)
                }
                0x20 => {
                    assert_eq!(cpu.reg.pc, 0x1014)
                }
                0x30 => {
                    assert_eq!(cpu.reg.pc, 0x1014)
                }
                0xC0 => {
                    assert_eq!(cpu.reg.pc, 0x7856)
                }
                0xC2 => {
                    assert_eq!(cpu.reg.pc, 0x3412)
                }
                0xC3 => {
                    assert_eq!(cpu.reg.pc, 0x3412)
                }
                0xC4 => {
                    assert_eq!(cpu.reg.pc, 0x3412)
                }
                0xC7 => {
                    assert_eq!(cpu.reg.pc, 0);
                    assert_eq!(cpu.pop(), 0x1001);
                }
                0xC9 => {
                    assert_eq!(cpu.reg.pc, 0x7856);
                }
                0xCD => {
                    assert_eq!(cpu.reg.pc, 0x3412)
                }
                0xCF => {
                    assert_eq!(cpu.reg.pc, 0x08);
                    assert_eq!(cpu.pop(), 0x1001);
                }
                _ => {
                    assert_eq!(cpu.reg.pc - 0x1000, op_length(i) as u16);

                    if op_cycles(i) != 0 {
                        assert_eq!(cpu.timer.cycle, op_cycles(i) as u16);
                    }
                }
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use instructions::*;
//     use debug::*;

//     fn build_cpu() -> Cpu {
//         let cpu = Cpu::new();
//         cpu.reg.pc = 0x1000;
//         cpu.reg.sp = 0x2000;
//         cpu
//     }
    
//     #[test]
//     fn test_op_0x38_add_sp_immediate() {
//         let cpu = build_cpu();
//         cpu.mem.mem[cpu.reg.pc as usize] = 0xE8;
//         cpu.mem.mem[(cpu.reg.pc + 1) as usize] = 1 as u8;
//         step(&mut cpu);
//         print_registers(&mut cpu.reg);
//         assert!(cpu.reg.sp == 0x2001);
//     }

//     #[test]
//     fn test_op_0xCE_add_sp_immediate() {
//         let cpu = build_cpu();
//         cpu.reg.a = 100;
//         cpu.reg.carry = true;
//         cpu.mem.mem[cpu.reg.pc as usize] = 0xCE;
//         cpu.mem.mem[(cpu.reg.pc + 1) as usize] = 10 as u8;
//         step(&mut cpu);
//         print_registers(&mut cpu.reg);
//         assert!(cpu.reg.a == 111);

//         cpu.reg.carry = false;
//         cpu.mem.mem[cpu.reg.pc as usize] = 0xCE;
//         cpu.mem.mem[(cpu.reg.pc + 1) as usize] = (0 as u8).wrapping_sub(35);
//         step(&mut cpu);
//         print_registers(&mut cpu.reg);
//         assert!(cpu.reg.a == 111 - 35);
//     }
// }