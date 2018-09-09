
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

pub fn rl_op(reg: &mut Registers, value: u8) -> u8 {
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
        0x01 => {
            // LD BC, d16: load immediate (d16) into BC
            // Length: 3
            // Cycles: 
            // Flags: - - - -
            reg.c = mem.read(reg.pc + 1);
            reg.b = mem.read(reg.pc + 2);
        }

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

        // SUB r: subtract register r from accumulator
        // Length: 1
        // Cycles: 4
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
        0x70 => {
            let hl = reg.hl();
            let b = reg.b;
            mem.write(hl, b);
        }
        0x71 => {
            let hl = reg.hl();
            let c = reg.c;
            mem.write(hl, c);
        }
        0x72 => {
            let hl = reg.hl();
            let d = reg.d;
            mem.write(hl, d);
        }
        0x73 => {
            let hl = reg.hl();
            let e = reg.e;
            mem.write(hl, e);
        }
        0x74 => {
            let hl = reg.hl();
            let h = reg.h;
            mem.write(hl, h);
        }
        0x75 => {
            let hl = reg.hl();
            let l = reg.l;
            mem.write(hl, l);
        }
        0x77 => {
            let hl = reg.hl();
            let a = reg.a;
            mem.write(hl, a);
        }

        // RET: set PC to 16-bit value popped from stack
        // Length: 1
        // Cycles: 16
        // Flags: - - - -
        0xC9 => {
            // Compensate for length of current instruction
            reg.pc = pop_op(reg, &mem) - 1;
        }


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

        // PUSH nn: push 16-bit register nn to stack
        // Length: 1
        // Flags: - - - -
        0xC5 => {
            let bc = reg.bc();
            push_op(reg, mem, bc);
        }
        0xD5 => {
            let de = reg.de();
            push_op(reg, mem, de);
        }
        0xE5 => {
            let hl = reg.hl();
            push_op(reg, mem, hl);
        }
        0xF5 => {
            let af = reg.af();
            push_op(reg, mem, af);
        }

        // POP nn: pop value from stack and store in 16-bit register nn
        // Length: 1
        // Cycles: 12
        // Flags: - - - -
        0xC1 => {
            let v = pop_op(reg, &mem);
            reg.set_bc(v);
        }

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
        0xA8 => {
            let a = reg.a;
            let b = reg.b;
            reg.a = xor_op(reg, a, b);
        }
        0xA9 => {
            let a = reg.a;
            let c = reg.c;
            reg.a = xor_op(reg, a, c);
        }
        0xAA => {
            let a = reg.a;
            let d = reg.d;
            reg.a = xor_op(reg, a, d);
        }
        0xAB => {
            let a = reg.a;
            let e = reg.e;
            reg.a = xor_op(reg, a, e);
        }
        0xAC => {
            let a = reg.a;
            let h = reg.h;
            reg.a = xor_op(reg, a, h);
        }
        0xAD => {
            let a = reg.a;
            let l = reg.l;
            reg.a = xor_op(reg, a, l);
        }
        0xAF => {
            let a = reg.a;
            reg.a = xor_op(reg, a, a);
        }

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
            let a = reg.a;
            let hl = reg.hl();
            mem.write(hl, a);
            reg.set_hl(hl + 1);
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


        // CP u8: Compare A with u8. Same as SUB but throw away result.
        // Length: 2
        // Cycles: 8
        // Flags: Z 1 H C
        0xFE => {
            let v = mem.read(reg.pc + 1);
            let a = reg.a;
            reg.set_z_flag(a == v);
            reg.set_carry(a < v);
            reg.f |= N_BIT;
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

                0x7C => {
                    // BIT 7, H: test if bit 7 in register H is set
                    // Length: 2
                    // Cycles: 8
                    // Flags: Z 0 1 -
                    let h = reg.h;
                    bit_op(reg, 7, h);
                }
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
