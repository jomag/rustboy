
extern crate ctrlc;

// use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::sync::Arc;
use std::sync::atomic::{ AtomicBool, Ordering };

const Z_BIT:  u8 = 1 << 7;   // zero flag
const N_BIT:  u8 = 1 << 6;   // subtract flag
const H_BIT:  u8 = 1 << 5;   // half carry flag
const C_BIT:  u8 = 1 << 4;   // carry flag

struct VM {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
    mem: [u8; 0x10000],
    bootstrap: [u8; 0x100]
}

fn add_i8_to_u16(a: u16, b: i8) -> u16 {
    if b > 0 {
        return a + b as u16;
    } else {
        return a - (-b) as u16;
    }
}

impl VM {
    fn new() -> Self {
        VM {
            a: 0, b: 0, c: 0, d: 0,
            e: 0, f: 0, h: 0, l: 0,
            sp: 0, pc: 0,
            mem: [0; 0x10000],
            bootstrap: [0; 0x100]
        }
    }

    fn z_flag(&mut self) -> bool {
        (self.f & Z_BIT) != 0
    }

    fn xor_op(&mut self, a: u8, value: u8) -> u8 {
        // Flags: Z 0 0 0
        let res = a ^ value;
        self.f &= !(Z_BIT | N_BIT | H_BIT | C_BIT);
        if res == 0 {
            self.f |= Z_BIT;
        }
        res
    }

    fn bit_op(&mut self, bit: u8, value: u8) {
        // Test if bit in register is set
        // Flags: Z 0 1 -
        if value & (1 << bit) == 0 {
            self.f &= !N_BIT;
            self.f |= Z_BIT | H_BIT;
        } else {
            self.f &= !(Z_BIT | N_BIT);
            self.f |= H_BIT;
        }
    }

    fn inc_op(&mut self, value: u8) -> u8 {
        // Flags: Z 0 H -
        let new_value = if value == 255 { 0 } else { value + 1 };

        if new_value == 0 {
            self.f |= Z_BIT;
        } else {
            self.f &= !Z_BIT;
        }

        self.f &= !N_BIT;

        if value < 255 && ((value & 0xF) + 1) & 0x10 != 0 {
            self.f |= H_BIT;
        } else {
            self.f &= !H_BIT;
        }

        new_value
    }

    fn inc16_op(&mut self, value: u16) -> u16 {
        return if value == 0xFFFF { 0 } else { value + 1 };
    }

    fn dec_op(&mut self, value: u8) -> u8 {
        // Flags: Z 1 H -
        let new_value = if value == 0 { 255 } else { value - 1 };

        if new_value == 0 {
            self.f |= Z_BIT | N_BIT;
        } else {
            self.f &= !Z_BIT;
            self.f |= N_BIT;
        }

        // FIXME: handle half-carry flag
        new_value
    }

    fn set_carry(&mut self, en: bool) {
        if en {
            self.f |= C_BIT;
        } else {
            self.f &= !C_BIT;
        }
    }

    fn rl_op(&mut self, value: u8) -> u8 {
        // Rotate left with carry flag
        // Flags: Z 0 0 C
        let mut t = (value as u32) << 1;

        if self.f & C_BIT != 0 {
            t |= 1;
        }

        if t & 256 == 0 {
            self.f &= !C_BIT;
        } else {
            self.f |= C_BIT;
        }

        if t == 0 {
            self.f |= Z_BIT;
        } else {
            self.f &= Z_BIT;
        }

        self.f &= !(N_BIT | H_BIT);
        return (t & 0xFF) as u8;
    }

    fn update_flags(&mut self, from: u8, to: u8) -> u8 {
        if to == 0 {
            self.f |= Z_BIT;
        } else {
            self.f &= !Z_BIT;
        }
        to
    }

    fn set_z_flag(&mut self, val: bool) {
        if val {
            self.f |= Z_BIT;
        } else {
           self.f &= !Z_BIT;
        }
    }

    fn reg_af(&self) -> u16 {
        // Return 16-bit value of registers A and F
        return (self.a as u16) << 8 | self.f as u16;
    }

    fn reg_bc(&self) -> u16 {
        // Return 16-bit value of registers B and C
        return (self.b as u16) << 8 | self.c as u16;
    }

    fn set_reg_bc(&mut self, value: u16) {
        self.b = ((value >> 8) & 0xFF) as u8;
        self.c = (value & 0xFF) as u8;
    }

    fn reg_de(&self) -> u16 {
        // Return 16-bit value of registers D and E
        return (self.d as u16) << 8 | self.e as u16;
    }

    fn set_reg_de(&mut self, value: u16) {
        self.d = ((value >> 8) & 0xFF) as u8;
        self.e = (value & 0xFF) as u8;
    }

    fn reg_hl(&self) -> u16 {
        // Return 16-bit value of registers H and L
        return (self.h as u16) << 8 | self.l as u16;
    }

    fn set_reg_hl(&mut self, value: u16) {
        self.h = ((value >> 8) & 0xFF) as u8;
        self.l = (value & 0xFF) as u8;
    }

    fn read(&self, addr: u16) -> u8 {
        // Read byte (u8) from memory
        if addr < 0x100 {
            return self.bootstrap[addr as usize];
        } else {
            return self.mem[addr as usize];
        }
    }

    fn read_i8(&self, addr: u16) -> i8 {
        return u8_to_i8(self.read(addr));
    }

    fn read_u16(&self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr + 1);
        return ((hi as u16) << 8) | (lo as u16);
    }

    fn write(&mut self, addr: u16, value: u8) {
        println!("WRITE MEM: 0x{:04X} = 0x{:02X}", addr, value);
        self.mem[addr as usize] = value;
    }

    fn print_state(&mut self) {
        print!("  A: 0x{:02X} B: 0x{:02X} C: 0x{:02X} D: 0x{:02X} ", self.a, self.b, self.c, self.d);
        println!("E: 0x{:02X} F: 0x{:02X} H: 0x{:02X} L: 0x{:02X}", self.e, self.f, self.h, self.l);
        println!("  SP: 0x{:04X} PC: 0x{:04X}", self.sp, self.pc);
        println!(
            "  Flags: Z={}, N={}, H={}, C={}",
            if (self.f & Z_BIT) == 0 { 0 } else { 1 },
            if (self.f & N_BIT) == 0 { 0 } else { 1 },
            if (self.f & H_BIT) == 0 { 0 } else { 1 },
            if (self.f & C_BIT) == 0 { 0 } else { 1 },
        )
    }

    fn load_bootstrap(&mut self, filename: &str) {
        // Open and read content of boot rom
        let mut f = File::open(filename)
            .expect("failed to open boot rom");
        f.read(&mut self.bootstrap)
            .expect("failed to read content of boot rom");
    }

    fn format_mnemonic(&self, addr: u16) -> String {
        let op: u8 = self.read(addr);
        match op {
            0x01 => { format!("LD  BC, ${:04X}", self.read_u16(addr + 1)) }

            // INC n: increment register n
            0x04 => { "INC  B".to_string() }
            0x0C => { "INC  C".to_string() }
            0x14 => { "INC  D".to_string() }
            0x1C => { "INC  E".to_string() }
            0x24 => { "INC  H".to_string() }
            0x2C => { "INC  L".to_string() }
            0x3C => { "INC  A".to_string() }

            // INC nn: increment 16-bit register nn
            0x13 => { "INC  DE".to_string() }
            0x23 => { "INC  HL".to_string() }

            // DEC n: decrement register n
            0x05 => { "DEC  B".to_string() }
            0x0D => { "DEC  C".to_string() }
            0x15 => { "DEC  D".to_string() }
            0x1D => { "DEC  E".to_string() }
            0x25 => { "DEC  H".to_string() }
            0x2D => { "DEC  L".to_string() }
            0x3D => { "DEC  A".to_string() }

            // LD n, d: load immediate into register n
            0x06 => { format!("LD   B, ${:02X}", self.read(addr + 1)) }
            0x0E => { format!("LD   C, ${:02X}", self.read(addr + 1)) }
            0x16 => { format!("LD   D, ${:02X}", self.read(addr + 1)) }
            0x1E => { format!("LD   E, ${:02X}", self.read(addr + 1)) }
            0x26 => { format!("LD   H, ${:02X}", self.read(addr + 1)) }
            0x2E => { format!("LD   L, ${:02X}", self.read(addr + 1)) }
            0x3E => { format!("LD   A, ${:02X}", self.read(addr + 1)) }

            0x11 => {
                let lo = self.read(addr + 1);
                let hi = self.read(addr + 2);
                format!("LD   DE, ${:02X}{:02X}", hi, lo)
            }
            0x17 => { "RLA".to_string() }
            0x18 => {
                let rel = self.read_i8(addr + 1);
                let abs = add_i8_to_u16(addr + 2, rel);
                format!("JR   {}  ; jump to 0x{:04X}", rel, abs)
            }
            0x1A => { "LD   A, (DE)".to_string() }

            0x20 => {
                let rel = self.read_i8(addr + 1);
                let abs = add_i8_to_u16(addr + 2, rel);
                format!("JR   NZ, {}    ; jump to 0x{:04X}", rel, abs)
            }
            0x21 => {
                let lo = self.read(addr + 1);
                let hi = self.read(addr + 2);
                format!("LD   HL, ${:02X}{:02X}", hi, lo)
            }
            0x22 => { "LD   (HL+), A".to_string() }
            0x28 => {
                let rel = self.read_i8(addr + 1);
                let abs = add_i8_to_u16(addr + 2, rel);
                format!("JR   Z, {}        ; jump to 0x{:04X}", rel, abs)
            }

            0x31 => {
                let lo = self.read(addr + 1);
                let hi = self.read(addr + 2);
                format!("LD   SP, ${:02X}{:02X}", hi, lo)
            }
            0x32 => { "LDD  (HL), A".to_string() }
            0x3D => { "DEC  A".to_string() }
            0x3E => { format!("LD   A, ${:02X}", self.read(addr + 1)) }

            0x4F => { "LD   C, A".to_string() }

            0x57 => { "LD   D, A".to_string() }

            0x67 => { "LD   H, A".to_string() }

            0x77 => { "LD   (HL), A".to_string() }
            0x78 => { "LD   A, B".to_string() }
            0x7B => { "LD   A, E".to_string() }
            0x7C => { "LD   A, H".to_string() }
            0x7D => { "LD   A, L".to_string() }

            0x86 => { "ADD  A, (HL)".to_string() }

            0x90 => { "SUB  B".to_string() }

            0xA8 => { "XOR  B".to_string() }
            0xA9 => { "XOR  C".to_string() }
            0xAA => { "XOR  D".to_string() }
            0xAB => { "XOR  E".to_string() }
            0xAC => { "XOR  H".to_string() }
            0xAD => { "XOR  L".to_string() }
            0xAF => { "XOR  A".to_string() }

            0xBE => { "CP   (HL)".to_string() }

            0xC1 => { "POP  BC".to_string() }
            0xC4 => { format!("CALL  NZ, ${:04X}", self.read_u16(addr + 1)) }
            0xC5 => { "PUSH BC".to_string() }
            0xC9 => { "RET".to_string() }
            0xCB => {
                let op2 = self.read(addr + 1);
                match op2 {
                    0x11 => { "RL   C".to_string() }
                    0x7C => { "BIT 7, h".to_string() }
                    _ => {
                        panic!("invalid instruction op code: 0x{:02X}{:02X}", op, op2);
                    }
                }
            }
            0xCD => { format!("CALL ${:04X}", self.read_u16(addr + 1)) }

            0xE0 => { format!("LD   ($FF00+${:02X}), A", self.read(addr + 1)) }
            0xE2 => { "LD   ($FF00+C), A".to_string() }
            0xEA => { format!("LD   (${:04X}), A", self.read_u16(addr + 1)) }

            0xF0 => { format!("LD   A, ($FF00+${:02X})", self.read(addr + 1)) }
            0xFE => { format!("CP   ${:02X}", self.read(addr + 1)) }

            _ => {
                panic!("invalid instruction op code at 0x{:04X}: 0x{:02X}", addr, op);
            }
        }
    }

    fn op_length(&self, addr: u16) -> u8 {
        const INSTRUCTION_LENGTH: [u8; 256] = [
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

        let op = self.read(addr);

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

    fn push(&mut self, value: u8) {
        let sp = self.sp - 1;
        self.write(sp, value);
        self.sp = sp;
    }

    fn push16(&mut self, value: u16) {
        self.push((value >> 8) as u8);
        self.push((value & 0xFF) as u8);
    }

    fn pop(&mut self) -> u8 {
        let v = self.read(self.sp);
        self.sp += 1;
        v
    }

    fn pop16(&mut self) -> u16{
        let lo = self.pop();
        let hi = self.pop();
        return (((hi as u16) << 8) | lo as u16) as u16;
    }

    fn step(&mut self) {
        let pc = self.pc;
        let op: u8 = self.read(pc);
        let length = self.op_length(pc);

        match op {
            0x01 => {
                // LD BC, d16: load immediate (d16) into BC
                // Length: 3
                // Flags: - - - -
                self.c = self.read(self.pc + 1);
                self.b = self.read(self.pc + 2);
            }

            // INC n: increment register n
            // Length: 1
            // Cycles: 4
            // Flags: Z 0 H -
            0x04 => {
                // INC B
                let b = self.b;
                self.b = self.inc_op(b);
            }
            0x0C => {
                // INC C
                let c = self.c;
                self.c = self.inc_op(c);
            }
            0x14 => {
                // INC D
                let d = self.d;
                self.d = self.inc_op(d);
            }
            0x1C => {
                // INC E
                let e = self.e;
                self.e = self.inc_op(e);
            }
            0x24 => {
                // INC H
                let h = self.h;
                self.h = self.inc_op(h);
            }
            0x2C => {
                // INC L
                let l = self.l;
                self.l = self.inc_op(l);
            }
            0x3C => {
                // INC A
                let a = self.a;
                self.a = self.inc_op(a);
            }

            // INC nn: increments content of register pair nn by 1
            // Length: 1
            // Cycles: 8
            // Flags: - - - -
            0x03 => {
                // INC BC
                let bc = self.reg_bc();
                let bc = self.inc16_op(bc);
                self.set_reg_bc(bc);
            }
            0x13 => {
                // INC DE
                let de = self.reg_de();
                let de = self.inc16_op(de);
                self.set_reg_de(de);
            }
            0x23 => {
                // INC HL
                let hl = self.reg_hl();
                let hl = self.inc16_op(hl);
                self.set_reg_hl(hl);
            }
            0x33 => {
                // INC SP
                let sp = self.sp;
                self.sp = self.inc16_op(sp);
            }

            // DEC n: decrement register n
            // Length: 1
            // Flags: Z 1 H -
            0x05 => {
                // DEC B
                let b = self.b;
                self.b = self.dec_op(b);
            }
            0x0D => {
                // DEC C
                let c = self.c;
                self.c = self.dec_op(c);
            }
            0x15 => {
                // DEC D
                let d = self.d;
                self.d = self.dec_op(d);
            }
            0x1D => {
                // DEC E
                let e = self.e;
                self.e = self.dec_op(e);
            }
            0x25 => {
                // DEC H
                let h = self.h;
                self.h = self.dec_op(h);
            }
            0x2D => {
                // DEC L
                let l = self.l;
                self.l = self.dec_op(l);
            }
            0x3D => {
                // DEC A
                let a = self.a;
                self.a = self.dec_op(a);
            }

            // LD n, d: load immediate into register n
            // Length: 2
            // Flags: - - - -
            0x06 => { self.b = self.read(self.pc + 1) }
            0x0E => { self.c = self.read(self.pc + 1) }
            0x16 => { self.d = self.read(self.pc + 1) }
            0x1E => { self.e = self.read(self.pc + 1) }
            0x26 => { self.h = self.read(self.pc + 1) }
            0x2E => { self.l = self.read(self.pc + 1) }
            0x3E => { self.a = self.read(self.pc + 1) }

            // LD n, m: load value of register m into register n
            // Length: 1
            // Cycles: 4
            // Flags: - - - -
            0x7F => {}                   // LD A,A
            0x78 => { self.a = self.b }  // LD A,B
            0x79 => { self.a = self.c }  // LD A,C
            0x7A => { self.a = self.d }  // LD A,D
            0x7B => { self.a = self.e }  // LD A,E
            0x7C => { self.a = self.h }  // LD A,H
            0x7D => { self.a = self.l }  // LD A,L

            0x47 => { self.b = self.a }  // LD B,A
            0x40 => {}                   // LD B,B
            0x41 => { self.b = self.c }  // LD B,C
            0x42 => { self.b = self.d }  // LD B,D
            0x43 => { self.b = self.e }  // LD B,E
            0x44 => { self.b = self.h }  // LD B,H
            0x45 => { self.b = self.l }  // LD B,L

            0x4F => { self.c = self.a }  // LD C,A
            0x48 => { self.c = self.b }  // LD C,B
            0x49 => {}                   // LD C,C
            0x4A => { self.c = self.d }  // LD C,D
            0x4B => { self.c = self.e }  // LD C,E
            0x4C => { self.c = self.h }  // LD C,H
            0x4D => { self.c = self.l }  // LD C,L

            0x57 => { self.d = self.a }  // LD D,A
            0x50 => { self.d = self.b }  // LD D,B
            0x51 => { self.d = self.c }  // LD D,C
            0x52 => {}                   // LD D,D
            0x53 => { self.d = self.e }  // LD D,E
            0x54 => { self.d = self.h }  // LD D,H
            0x55 => { self.d = self.l }  // LD D,L

            0x5F => { self.e = self.a }  // LD E,A
            0x58 => { self.e = self.b }  // LD E,B
            0x59 => { self.e = self.c }  // LD E,C
            0x5A => { self.e = self.d }  // LD E,D
            0x5B => {}                   // LD E,E
            0x5C => { self.e = self.h }  // LD E,H
            0x5D => { self.e = self.l }  // LD E,L

            0x67 => { self.h = self.a }  // LD H,A
            0x60 => { self.h = self.b }  // LD H,B
            0x61 => { self.h = self.c }  // LD H,C
            0x62 => { self.h = self.d }  // LD H,D
            0x63 => { self.h = self.e }  // LD H,E
            0x64 => {}                   // LD H,H
            0x65 => { self.h = self.l }  // LD H,L

            0x6F => { self.l = self.a }  // LD L,A
            0x68 => { self.l = self.b }  // LD L,B
            0x69 => { self.l = self.c }  // LD L,C
            0x6A => { self.l = self.d }  // LD L,D
            0x6B => { self.l = self.e }  // LD L,E
            0x6C => { self.l = self.h }  // LD L,H
            0x6D => {}                   // LD L,L

            // LD n, (mm): load value from memory into register n
            // Length: 1
            // Flags: - - - -
            0x0A => { self.a = self.read(self.reg_bc()) }
            0x1A => { self.a = self.read(self.reg_de()) }

            0xE0 => {
                // LDH (n), A: Put A into memory address $FF00+n
                // Length: 2
                // Flags: - - - -
                let n = self.read(self.pc + 1);
                let a = self.a;
                self.write(0xFF00 + n as u16, a);
            }

            // LD (HL), n: store register value to memory at address HL
            // Length: 1
            // Flags: - - - -
            0x70 => {
                let hl = self.reg_hl();
                let b = self.b;
                self.write(hl, b);
            }
            0x71 => {
                let hl = self.reg_hl();
                let c = self.c;
                self.write(hl, c);
            }
            0x72 => {
                let hl = self.reg_hl();
                let d = self.d;
                self.write(hl, d);
            }
            0x73 => {
                let hl = self.reg_hl();
                let e = self.e;
                self.write(hl, e);
            }
            0x74 => {
                let hl = self.reg_hl();
                let h = self.h;
                self.write(hl, h);
            }
            0x75 => {
                let hl = self.reg_hl();
                let l = self.l;
                self.write(hl, l);
            }
            0x77 => {
                let hl = self.reg_hl();
                let a = self.a;
                self.write(hl, a);
            }

            // RET: set PC to 16-bit value popped from stack
            // Length: 1
            // Cycles: 16
            // Flags: - - - -
            0xC9 => {
                // Compensate for length of current instruction
                self.pc = self.pop16() - 1;
            }


            // CALL a16: push address of next instruction on stack
            //           and jump to address a16
            // Length: 3
            // Flags: - - - -
            0xCD => {
                let nexti = self.pc + 3;
                self.push16(nexti);

                // Set PC to target address. Compensate
                // for the length of the current instruction.
                self.pc = self.read_u16(self.pc + 1) - 3;
            }

            // PUSH nn: push 16-bit register nn to stack
            // Length: 1
            // Flags: - - - -
            0xC5 => {
                let bc = self.reg_bc();
                self.push16(bc);
            }
            0xD5 => {
                let de = self.reg_de();
                self.push16(de);
            }
            0xE5 => {
                let hl = self.reg_hl();
                self.push16(hl);
            }
            0xF5 => {
                let af = self.reg_af();
                self.push16(af);
            }

            // POP nn: pop value from stack and store in 16-bit register nn
            // Length: 1
            // Cycles: 12
            // Flags: - - - -
            0xC1 => {
                let v = self.pop16();
                self.set_reg_bc(v);
            }



            0xE2 => {
                // LD ($FF00+C), A: put value of A in address 0xFF00 + C
                // Length: 2
                // Cycles: 8
                // Flags: - - - -
                let addr = 0xFF00 + self.c as u16;
                let a = self.a;
                self.write(addr, a);
            }

            0x11 => {
                // LD DE, d16: load immediate (d16) into DE
                // Length: 3
                // Cycles: 12
                // Flags: - - - -
                self.e = self.read(self.pc + 1);
                self.d = self.read(self.pc + 2);
            }

            0x18 => {
                // JR d8: relative jump
                // Length: 2
                // Cycles: 12
                let offs = self.read_i8(self.pc + 1);
                if offs >= 0 {
                    self.pc = self.pc.wrapping_add(offs as u16);
                } else {
                    self.pc = self.pc.wrapping_sub(-offs as u16);
                }
            }

            0x20 => {
                // JR NZ, d8: jump d8 relative to PC if Z is reset
                // Length: 2
                // Cycles: 12/8
                // Flags: - - - -
                let offs = self.read_i8(self.pc + 1);
                if !self.z_flag() {
                    if offs >= 0 {
                        self.pc = self.pc.wrapping_add(offs as u16);
                    } else {
                        self.pc = self.pc.wrapping_sub(-offs as u16);
                    }
                }
            }

            0x28 => {
                // JR Z, d8: jump d8 relative to PC if Z is set
                // Length: 2
                // Cycles: 12/8
                // Flags: - - - -
                let offs = self.read_i8(self.pc + 1);
                if self.z_flag() {
                    if offs >= 0 {
                        self.pc = self.pc.wrapping_add(offs as u16);
                    } else {
                        self.pc = self.pc.wrapping_sub(-offs as u16);
                    }
                }
            }

            0x21 => {
                // LD HL, d16: load immediate (d16) into HL
                // Length: 3
                // Cycles: 12
                // Flags: - - - -
                self.l = self.read(self.pc + 1);
                self.h = self.read(self.pc + 2);
            }

            0x31 => {
                // LD SP, d16: load immediate (d16) into SP
                // Length: 3
                // Cycles: 12
                // Flags: - - - -
                self.sp = self.read_u16(self.pc + 1);
            }

            0x32 => {
                // LD (HL-), A: put A into memory address HL, decrement HL
                // Length: 1
                // Cycles: 8
                // Flags: - - - -
                let hl: u16 = ((self.h as u16) << 8) | (self.l as u16);
                let a = self.a;
                self.write(hl, a);
                let hl = hl - 1;
                self.h = (hl >> 8) as u8;
                self.l = (hl & 0xFF) as u8;
            }

            // XOR N: assign A xor N to A
            // Length: 1
            // Cycles: 4
            // Flags: Z 0 0 0
            0xA8 => {
                let a = self.a;
                let b = self.b;
                self.a = self.xor_op(a, b);
            }
            0xA9 => {
                let a = self.a;
                let c = self.c;
                self.a = self.xor_op(a, c);
            }
            0xAA => {
                let a = self.a;
                let d = self.d;
                self.a = self.xor_op(a, d);
            }
            0xAB => {
                let a = self.a;
                let e = self.e;
                self.a = self.xor_op(a, e);
            }
            0xAC => {
                let a = self.a;
                let h = self.h;
                self.a = self.xor_op(a, h);
            }
            0xAD => {
                let a = self.a;
                let l = self.l;
                self.a = self.xor_op(a, l);
            }
            0xAF => {
                let a = self.a;
                self.a = self.xor_op(a, a);
            }

            // RLA: Rotate the contents of register A to the left
            // Length: 1
            // Cycles: 4
            // Flags: 0 0 0 C
            0x17 => {
                let b0 = if self.f & C_BIT == 0 { 0 } else { 1 };
                let b8 = self.a & 128 == 0;
                self.a = self.a << 1 | b0;
                self.set_carry(b8);
                self.f &= !(Z_BIT | H_BIT | N_BIT);
            }

            // LD (HL+), A: store value of A at (HL) and increment HL
            // Length: 1
            // Cycles: 8
            // Flags: - - - -
            // Alt mnemonic 1: LD (HLI), A
            // Alt mnemonic 2: LDI (HL), A
            0x22 => {
                let a = self.a;
                let hl = self.reg_hl();
                self.write(hl, a);
                self.set_reg_hl(hl + 1);
            }

            // LD (a16), A: store value of A at address a16
            // Length: 3
            // Cycles: 16
            // Flags: - - - -
            0xEA => {
                let addr = self.read_u16(self.pc + 1);
                let val = self.a;
                self.write(addr, val);
            }

            // 
            0xF0 => {
                
            }

            // CP u8: Compare A with u8. Same as SUB but throw away result.
            // Length: 2
            // Cycles: 8
            // Flags: Z 1 H C
            0xFE => {
                let v = self.read(self.pc + 1);
                let a = self.a;
                self.set_z_flag(a == v);
                self.set_carry(a < v);
                self.f |= N_BIT;
            }

            // Prefix 0xCB instructions
            0xCB => {
                let op2 = self.read(self.pc + 1);
                match op2 {
                    // RL n: rotate register n left with carry flag
                    0x11 => {
                        let c = self.c;
                        self.c = self.rl_op(c);
                    }

                    0x7C => {
                        // BIT 7, H: test if bit 7 in register H is set
                        // Length: 2
                        // Cycles: 8
                        // Flags: Z 0 1 -
                        let h = self.h;
                        self.bit_op(7, h);
                    }
                    _ => {
                        panic!("Unsupported opcode at 0x{:04X}: 0x{:02X}{:02X}", self.pc, op, op2);
                    }
                }
            }

            _ => {
                panic!("Unsupported opcode at 0x{:04X}: 0x{:02X}", self.pc, op);
            }
        }

        self.pc += length as u16;
    }
}

/*
fn print_hex(buf: &Vec<u8>) {
    let len = buf.len();

    for i in (0..len).step_by(16) {
        print!("{:04x}: ", i);
        for j in i..i+16 {
            print!("{:02x} ", buf[j]);
        }
        println!("");
    }
}
*/

fn u8_to_i8(v: u8) -> i8 {
    return (0 as i8).wrapping_add(v as i8);
}

fn print_listing(vm: &VM, addr: u16, line_count: i32) -> u16 {
    let mut a = addr;
    for _n in 0..line_count {
        println!("0x{:04X}: {}", a, vm.format_mnemonic(a));
        a = a + (vm.op_length(a) as u16);
    }
    return a;
}

fn main() {
    use std::io::stdin;
    use std::io::stdout;

    let mut vm = VM::new();

    println!();
    println!("Starting RustBoy (GameBoy Emulator written in Rust)");
    println!("---------------------------------------------------");
    println!();

    vm.load_bootstrap("rom/boot.gb");

    let mut breakpoints: Vec<u16> = Vec::new();
    let mut stepping = true;
    let mut last_command = "".to_string();

    breakpoints.push(0x000C);
    // breakpoints.push(0x0095);
    breakpoints.push(0x0034);
    breakpoints.push(0x0040);
    breakpoints.push(0x0051);

    let ctrlc_event = Arc::new(AtomicBool::new(false));
    let ctrlc_event_clone = ctrlc_event.clone();

    ctrlc::set_handler(move || {
        println!("Ctrl-C: breaking execution");
        ctrlc_event_clone.store(true, Ordering::SeqCst)
    }).expect("failed to setup ctrl-c handler");

    loop {
        if ctrlc_event.load(Ordering::SeqCst) {
            stepping = true;
        }

        if breakpoints.contains(&vm.pc) {
            println!("- at breakpoint (PC: 0x{:04X})", vm.pc);
            stepping = true;
        }

        if stepping {
            vm.print_state();
            let pc = vm.pc;
            let mut list_offset = pc;
            println!("0x{:04X}: {}", pc, vm.format_mnemonic(pc));

            loop {
                print!("(debug) ");
                stdout().flush().ok();
                let mut cmd_s: String = String::new();
                stdin().read_line(&mut cmd_s).expect("invalid command");

                if cmd_s == "" {
                    cmd_s = last_command.clone();
                } else {
                    last_command = cmd_s.clone();
                }

                let args: Vec<_> = cmd_s.split_whitespace().collect();

                match args[0] {
                    "c" => { stepping = false; break; },
                    "s" => { break; }
                    "n" => { break; }
                    "l" => {
                        if args.len() > 1 {
                            list_offset = args[1].parse::<u16>().unwrap();
                        }
                        list_offset = print_listing(&vm, list_offset, 10);
                    }
                    "" => {}
                    _ => { println!("invalid command!"); }
                }
            }
        }

        vm.step();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_to_i8() {
        assert_eq!(u8_to_i8(0), 0);
        assert_eq!(u8_to_i8(127), 127);
        assert_eq!(u8_to_i8(128), -0);
        assert_eq!(u8_to_i8(129), -1);
        assert_eq!(u8_to_i8(0xF0), -112);
    }

    #[test]
    fn test_op__inc_c() {
        let mut vm = VM::new();
        vm.write(0, 0x0C);

        vm.c = 100;
        vm.pc = 0;
        vm.step();
        assert_eq!(vm.c, 101);
        assert_eq!(vm.f, 0);

        vm.c = 255;
        vm.pc = 0;
        vm.step();
        assert_eq!(vm.c, 0);
        assert_eq!(vm.f, Z_BIT);
    }
}
