
// use std::io;
use std::io::prelude::*;
use std::fs::File;

const S_BIT:  u8 = 1 << 7;   // sign flag
const Z_BIT:  u8 = 1 << 6;   // zero flag
const F5_BIT: u8 = 1 << 5;   // undocumented flag
const H_BIT:  u8 = 1 << 4;   // half carry flag
const F3_BIT: u8 = 1 << 3;   // undocumented flag
const PV_BIT: u8 = 1 << 2;   // parity/overflow flag
const N_BIT:  u8 = 1 << 1;   // subtract flag
const C_BIT:  u8 = 1 << 0;   // carry flag

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

    fn reg_bc(&self) -> u16 {
        // Return 16-bit value of registers B and C
        return (self.b as u16) << 8 | self.c as u16;
    }

    fn reg_de(&self) -> u16 {
        // Return 16-bit value of registers D and E
        return (self.e as u16) << 8 | self.e as u16;
    }

    fn reg_hl(&self) -> u16 {
        // Return 16-bit value of registers H and L
        return (self.h as u16) << 8 | self.l as u16;
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
            0x01 => {
                let hi = self.read(addr + 1);
                let lo = self.read(addr + 2);
                return format!("LD  BC, ${:02X}{:02X}", hi, lo);
            }

            0x0E => {
                return format!("LD  C, ${:02X}", self.read(addr + 1));
            }

            0x11 => {
                let hi = self.read(addr + 1);
                let lo = self.read(addr + 2);
                return format!("LD  DE, ${:02X}{:02X}", hi, lo);
            }

            0x20 => {
                return format!("JR  NZ, {}", self.read_i8(addr + 1));
            }

            0x21 => {
                let lo = self.read(addr + 1);
                let hi = self.read(addr + 2);
                return format!("LD  HL, ${:02X}{:02X}", hi, lo);
            }

            0x31 => {
                let lo = self.read(addr + 1);
                let hi = self.read(addr + 2);
                return format!("LD  SP, ${:02X}{:02X}", hi, lo);
            }

            0x32 => { return "LDD (HL), A".to_string(); }

            0x3E => {
                return format!("LD  A, ${:02X}", self.read(addr + 1));
            }

            0xE2 => {
                return format!("LD  ($FF00+C), A");
            }

            0xA8 => { return "XOR B".to_string(); }
            0xA9 => { return "XOR C".to_string(); }
            0xAA => { return "XOR D".to_string(); }
            0xAB => { return "XOR E".to_string(); }
            0xAC => { return "XOR H".to_string(); }
            0xAD => { return "XOR L".to_string(); }
            0xAF => { return "XOR A".to_string(); }

            0xCB => {
                let op2 = self.read(addr + 1);
                match op2 {
                    0x7C => { return "BIT 7, h".to_string(); }
                    _ => {
                        panic!("invalid instruction op code: 0x{:02X}{:02X}", op, op2);
                    }
                }
            }

            _ => {
                panic!("invalid instruction op code: 0x{:02X}", op);
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
            2, 1, 2, 0,  0, 1, 2, 1,  2, 1, 3, 0,  0, 0, 2, 1,
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

    fn step(&mut self) -> i32 {
        let pc = self.pc;
        let op: u8 = self.read(pc);
        let length = self.op_length(pc);
        let cycles: i32;

        match op {
            0x01 => {
                // LD BC, d16: load immediate (d16) into BC
                self.c = self.read(self.pc + 1);
                self.b = self.read(self.pc + 2);
                cycles = 0;
            }

            0x0E => {
                // LD C, d8: load immediate (d8) into C
                self.c = self.read(self.pc + 1);
                cycles = 8;
            }

            0x11 => {
                // LD DE, d16: load immediate (d16) into DE
                self.d = self.read(self.pc + 1);
                self.e = self.read(self.pc + 2);
                cycles = 12;
            }

            0x21 => {
                // LD HL, d16: load immediate (d16) into HL
                self.l = self.read(self.pc + 1);
                self.h = self.read(self.pc + 2);
                cycles = 12;
            }

            0x31 => {
                // LD SP, d16: load immediate (d16) into SP
                self.sp = self.read_u16(self.pc + 1);
                cycles = 12;
            }

            0x3E => {
                // LD A, d8: load immediate (d8) into A
                self.a = self.read(self.pc + 1);
                cycles = 8;
            }

            0xE2 => {
                // LD ($FF00+C), A: put value of A in address 0xFF00 + C
                let addr = 0xFF00 + self.c as u16;
                let value = self.a;
                self.write(addr, value);
                cycles = 8;
            }

            0x20 => {
                // JR NZ, d8: jump d8 relative to PC if Z is reset
                let offs = self.read_i8(self.pc + 1);
                self.pc += 2;
                if (self.f & Z_BIT) != 0 {
                    if offs >= 0 {
                        self.pc = self.pc.wrapping_add(offs as u16);
                    } else {
                        self.pc = self.pc.wrapping_sub(-offs as u16);
                    }
                }
                println!("JR  NZ, {}", offs);
                return 0;
            }

            0xAF => {
                self.a |= self.a ^ self.a;
                println!("XOR A");
                self.pc += 1;
                return 0;
            }
            0xA8 => {
                self.a |= self.a ^ self.b;
                println!("XOR B");
                self.pc += 1;
                return 0;
            }
            0xA9 => {
                self.a |= self.a ^ self.c;
                println!("XOR C");
                self.pc += 1;
                return 0;
            }
            0xAA => {
                self.a |= self.a ^ self.d;
                println!("XOR D");
                self.pc += 1;
                return 0;
            }
            0xAB => {
                self.a |= self.a ^ self.e;
                println!("XOR E");
                self.pc += 1;
                return 0;
            }
            0xAC => {
                self.a |= self.a ^ self.h;
                println!("XOR H");
                self.pc += 1;
                return 0;
            }
            0xAD => {
                self.a |= self.a ^ self.l;
                println!("XOR L");
                self.pc += 1;
                return 0;
            }
            0x32 => {
                println!("LDD (HL), A");
                let hl: u16 = ((self.h as u16) << 8) | (self.l as u16);
                let a = self.a;
                self.write(hl, a);
                self.pc +=1;
                let hl_dec = hl - 1;
                self.h = (hl_dec >> 8) as u8;
                self.l = (hl_dec & 0xFF) as u8;
                return 0;
            }

            0xCB => {
                let op2 = self.read(self.pc + 1);
                match op2 {
                    0x7C => {
                        self.f ^= !N_BIT;
                        self.f |= H_BIT;
                        if self.h & (1 << 7) == 0 {
                            self.f &= !Z_BIT;
                        } else {
                            self.f |= Z_BIT;
                        }
                        self.pc += 2;
                        println!("BIT 7, h");
                        return 0;
                    }
                    _ => {
                        panic!("Unsupported opcode 0x{:02X}{:02X}", op, op2);
                    }
                }
            }
            _ => {
                panic!("Unsupported opcode: 0x{:X}", op);
            }
        }

        self.pc += length as u16;
        return cycles;
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
    let mut user_input = String::new();

    println!();
    println!("Starting RustBoy (GameBoy Emulator written in Rust)");
    println!("---------------------------------------------------");
    println!();

    vm.load_bootstrap("rom/boot.gb");

    let breakpoints: Vec<u16> = Vec::new();
    let mut stepping = true;

    loop {
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
                let cmd = cmd_s.trim();

                match cmd {
                    "c" => { stepping = false; break; },
                    "s" => { break; }
                    "l" => {
                        list_offset = print_listing(&vm, list_offset, 10);
                    }
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
}
