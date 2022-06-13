use std::collections::HashMap;
use std::io::Write;

use super::emu::Emu;
use super::mmu::MMU;

#[derive(PartialEq)]
pub enum ExecState {
    // Continuous execution
    RUN,

    // Continue execution after a breakpoint.
    // State will change to RUN after next operation.
    CONTINUE,

    // Single-stepping
    STEP,
}

pub struct Breakpoint {
    pub enabled: bool,
}

impl Breakpoint {
    pub fn evaluate(&self, _emu: &Emu) -> bool {
        self.enabled
    }
}

pub struct Debug {
    // If true, execution will break on "software breakpoints",
    // aka "ld b, b" instructions (0x40).
    pub source_code_breakpoints: bool,
    pub debug_log: Option<std::fs::File>,
    pub state: ExecState,

    // When single-stepping, steps holds the number of steps
    // queued for execution.
    pub steps: u32,

    pub breakpoints: HashMap<u16, Vec<Breakpoint>>,

    // Execution will break when this scanline is reached.
    // Set to a value >153 to disable.
    pub break_on_scanline: usize,

    // Break on interrupt if not masked
    pub break_on_interrupt: u8,
}

impl Debug {
    pub fn new() -> Self {
        Debug {
            source_code_breakpoints: false,
            debug_log: None,
            state: ExecState::RUN,
            steps: 0,
            breakpoints: HashMap::new(),
            break_on_scanline: 0xFFFF,
            break_on_interrupt: 0x00,
        }
    }

    pub fn add_breakpoint(&mut self, adr: u16, bp: Breakpoint) {
        self.breakpoints.entry(adr).or_insert(vec![]).push(bp);
    }

    pub fn break_on_scanline(&mut self, scanline: usize) {
        self.break_on_scanline = scanline;
    }

    pub fn break_execution(&mut self) {
        println!("Breaking execution");
        self.state = ExecState::STEP;
        self.steps = 0;
    }

    pub fn continue_execution(&mut self) {
        println!("Continue execution");
        self.state = ExecState::CONTINUE;
    }

    pub fn next(&mut self) -> bool {
        match self.state {
            ExecState::RUN => true,
            ExecState::CONTINUE => {
                self.state = ExecState::RUN;
                true
            }
            ExecState::STEP => {
                if self.steps > 0 {
                    self.steps -= 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.steps += 1;
    }

    pub fn start_debug_log(&mut self, filename: &str) {
        self.debug_log = Some(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(filename)
                .unwrap(),
        );
    }

    #[allow(dead_code)]
    pub fn finalize(&mut self) {
        match self.debug_log {
            Some(ref mut f) => match f.sync_all() {
                Ok(_) => {}
                Err(e) => println!("Failed to sync log: {:?}", e),
            },
            None => {}
        };
    }

    // Perform debugging actions before every op.
    // Returns true if a breakpoint has been triggered.
    pub fn before_op(&mut self, emu: &Emu) -> bool {
        // FIXME: this will be executed even if next op is not executed
        // because execution is stopped.
        match self.debug_log {
            Some(ref mut f) => {
                let reg = &emu.mmu.reg;
                let pc = reg.pc as usize;
                if !emu.mmu.bootstrap_mode {
                    let m0 = emu.mmu.direct_read(pc);
                    let m1 = emu.mmu.direct_read(pc + 1);
                    let m2 = emu.mmu.direct_read(pc + 2);
                    let m3 = emu.mmu.direct_read(pc + 3);
                    println!(
                         "A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X}) {}",
                        reg.a,reg.get_f(),reg.b,reg.c,reg.d,reg.e,reg.h,reg.l,reg.sp,pc,m0,m1,m2,m3, format_mnemonic(&emu.mmu, pc),
                    );
                    let res = writeln!(
                        f, "A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X}) {}",
                        reg.a,reg.get_f(),reg.b,reg.c,reg.d,reg.e,reg.h,reg.l,reg.sp,pc,m0,m1,m2,m3, format_mnemonic(&emu.mmu, pc),
                    );
                    match res {
                        Ok(_) => {}
                        Err(_) => panic!("Failed to write log"),
                    };
                    match f.flush() {
                        Ok(_) => {}
                        Err(_) => panic!("Failed to flush log"),
                    }
                }
            }
            None => {}
        }

        // Check breakpoints, unless current state is CONTINUE
        // which means that we're continuing after a breakpoint
        // was reached.
        if self.state != ExecState::CONTINUE {
            let pc = emu.mmu.reg.pc;
            if self.breakpoints.contains_key(&pc) {
                for bp in self.breakpoints[&pc].iter() {
                    if bp.evaluate(emu) {
                        self.state = ExecState::STEP;
                    }
                }
            }

            if self.source_code_breakpoints {
                match emu.mmu.direct_read(emu.mmu.reg.pc as usize) {
                    0x40 => self.state = ExecState::STEP,
                    _ => {}
                }
            }

            if emu.mmu.ppu.ly == self.break_on_scanline {
                self.break_on_scanline = 0xFFFF;
                self.state = ExecState::STEP;
            }

            if emu.mmu.entered_interrupt_handler & self.break_on_interrupt != 0 {
                self.state = ExecState::STEP;
            }
        }

        return self.next();
    }
}

fn add_i8_to_u16(a: u16, b: i8) -> u16 {
    if b > 0 {
        return a + b as u16;
    } else {
        return a - (-b) as u16;
    }
}

const SIMPLE_MNEMONICS: [&str; 256] = [
    // 0x00
    "NOP",
    "",
    "LD   (BC), A",
    "INC  BC",
    "INC  B",
    "DEC  B",
    "",
    "RLCA",
    // 0x08
    "",
    "ADD  HL, BC",
    "LD   A, (BC)",
    "DEC  BC",
    "INC  C",
    "DEC  C",
    "",
    "RRCA",
    // 0x10
    "STOP 0",
    "",
    "LD   (DE), A",
    "INC  DE",
    "INC  D",
    "DEC  D",
    "",
    "RLA",
    // 0x18
    "",
    "ADD  HL, DE",
    "LD   A, (DE)",
    "DEC  DE",
    "INC  E",
    "DEC  E",
    "",
    "RRA",
    // 0x20
    "",
    "",
    "LD   (HL+), A",
    "INC  HL",
    "INC  H",
    "DEC  H",
    "",
    "DAA",
    // 0x28
    "",
    "ADD  HL, HL",
    "LD   A, (HL+)",
    "DEC  HL",
    "INC  L",
    "DEC  L",
    "",
    "CPL",
    // 0x30
    "",
    "",
    "LD   (HL-), A",
    "INC  SP",
    "INC  (HL)",
    "DEC  (HL)",
    "",
    "SCF",
    // 0x38
    "",
    "ADD  HL, SP",
    "LD   A, (HL-)",
    "DEC  SP",
    "INC  A",
    "DEC  A",
    "",
    "CCF",
    // 0x40
    "LD   B, B",
    "LD   B, C",
    "LD   B, D",
    "LD   B, E",
    "LD   B, H",
    "LD   B, L",
    "LD   B, (HL)",
    "LD   B, A",
    // 0x48
    "LD   C, B",
    "LD   C, C",
    "LD   C, D",
    "LD   C, E",
    "LD   C, H",
    "LD   C, L",
    "LD   C, (HL)",
    "LD   C, A",
    // 0x50
    "LD   D, B",
    "LD   D, C",
    "LD   D, D",
    "LD   D, E",
    "LD   D, H",
    "LD   D, L",
    "LD   D, (HL)",
    "LD   D, A",
    // 0x58
    "LD   E, B",
    "LD   E, C",
    "LD   E, D",
    "LD   E, E",
    "LD   E, H",
    "LD   E, L",
    "LD   E, (HL)",
    "LD   E, A",
    // 0x60
    "LD   H, B",
    "LD   H, C",
    "LD   H, D",
    "LD   H, E",
    "LD   H, H",
    "LD   H, L",
    "LD   H, (HL)",
    "LD   H, A",
    // 0x68
    "LD   L, B",
    "LD   L, C",
    "LD   L, D",
    "LD   L, E",
    "LD   L, H",
    "LD   L, L",
    "LD   L, (HL)",
    "LD   L, A",
    // 0x70
    "LD   (HL), B",
    "LD   (HL), C",
    "LD   (HL), D",
    "LD   (HL), E",
    "LD   (HL), H",
    "LD   (HL), L",
    "HALT",
    "LD   (HL), A",
    // 0x78
    "LD   A, B",
    "LD   A, C",
    "LD   A, D",
    "LD   A, E",
    "LD   A, H",
    "LD   A, L",
    "LD   A, (HL)",
    "LD   A, A",
    // 0x80
    "ADD  A, B",
    "ADD  A, C",
    "ADD  A, D",
    "ADD  A, E",
    "ADD  A, H",
    "ADD  A, L",
    "ADD  A, (HL)",
    "ADD  A, A",
    // 0x88
    "ADC  A, B",
    "ADC  A, C",
    "ADC  A, D",
    "ADC  A, E",
    "ADC  A, H",
    "ADC  A, L",
    "ADC  A, (HL)",
    "ADC  A, A",
    // 0x90
    "SUB  B",
    "SUB  C",
    "SUB  D",
    "SUB  E",
    "SUB  H",
    "SUB  L",
    "SUB  (HL)",
    "SUB  A",
    // 0x98
    "SBC  A, B",
    "SBC  A, C",
    "SBC  A, D",
    "SBC  A, E",
    "SBC  A, H",
    "SBC  A, L",
    "SBC  A, (HL)",
    "SBC  A, A",
    // 0xA0
    "AND  B",
    "AND  C",
    "AND  D",
    "AND  E",
    "AND  H",
    "AND  L",
    "AND  (HL)",
    "AND  A",
    // 0xA8
    "XOR  B",
    "XOR  C",
    "XOR  D",
    "XOR  E",
    "XOR  H",
    "XOR  L",
    "XOR  (HL)",
    "XOR  A",
    // 0xB0
    "OR   B",
    "OR   C",
    "OR   D",
    "OR   E",
    "OR   H",
    "OR   L",
    "OR   (HL)",
    "OR   A",
    // 0xB8
    "CP   B",
    "CP   C",
    "CP   D",
    "CP   E",
    "CP   H",
    "CP   L",
    "CP   (HL)",
    "CP   A",
    // 0xC0
    "RET  NZ",
    "POP  BC",
    "",
    "",
    "",
    "PUSH BC",
    "",
    "RST  00H",
    // 0xC8
    "RET  Z",
    "RET",
    "",
    "",
    "",
    "",
    "",
    "RST  08H",
    // 0xD0
    "RET  NC",
    "POP  DE",
    "",
    "",
    "",
    "PUSH DE",
    "",
    "RST  10H",
    // 0xD8
    "RET  C",
    "RETI",
    "",
    "",
    "",
    "",
    "",
    "RST  18H",
    // 0xE0
    "",
    "POP  HL",
    "LD   (C), A",
    "",
    "",
    "PUSH HL",
    "",
    "RST  20H",
    // 0xE8
    "",
    "JP   (HL)",
    "",
    "",
    "",
    "",
    "",
    "RST  28H",
    // 0xF0
    "",
    "POP  AF",
    "LD   A, (C)",
    "DI",
    "",
    "PUSH AF",
    "",
    "RST 30H",
    // 0xF8
    "",
    "LD   SP, HL",
    "",
    "EI",
    "",
    "",
    "",
    "RST  38H",
];

pub fn format_mnemonic(mmu: &MMU, addr: usize) -> String {
    let op: u8 = mmu.direct_read(addr);

    match op {
        0x01 => format!("LD   BC, ${:04X}", mmu.direct_read_u16(addr + 1)),

        // LD n, d: load immediate into register n
        0x06 => format!("LD   B, ${:02X}", mmu.direct_read(addr + 1)),
        0x08 => format!("LD   ${:02X}, SP", mmu.direct_read(addr + 1)),
        0x0E => format!("LD   C, ${:02X}", mmu.direct_read(addr + 1)),
        0x16 => format!("LD   D, ${:02X}", mmu.direct_read(addr + 1)),
        0x1E => format!("LD   E, ${:02X}", mmu.direct_read(addr + 1)),
        0x26 => format!("LD   H, ${:02X}", mmu.direct_read(addr + 1)),
        0x2E => format!("LD   L, ${:02X}", mmu.direct_read(addr + 1)),
        0x3E => format!("LD   A, ${:02X}", mmu.direct_read(addr + 1)),

        0x11 => {
            let lo = mmu.direct_read(addr + 1);
            let hi = mmu.direct_read(addr + 2);
            format!("LD   DE, ${:02X}{:02X}", hi, lo)
        }

        0x18 => {
            let rel = mmu.direct_read_i8(addr + 1);
            let abs = add_i8_to_u16(addr as u16 + 2, rel);
            format!("JR   {}  ; jump to 0x{:04X}", rel, abs)
        }

        0x1A => {
            let de = mmu.reg.de();
            let val = mmu.direct_read(de as usize);
            format!("LD   A, (DE)  ; DE=0x{:04X} (DE)=0x{:02X}", de, val)
        }

        0x20 => {
            let rel = mmu.direct_read_i8(addr + 1);
            let abs = add_i8_to_u16(addr as u16 + 2, rel);
            format!("JR   NZ, {}    ; jump to 0x{:04X}", rel, abs)
        }

        0x21 => {
            let lo = mmu.direct_read(addr + 1);
            let hi = mmu.direct_read(addr + 2);
            format!("LD   HL, ${:02X}{:02X}", hi, lo)
        }

        0x28 => {
            let rel = mmu.direct_read_i8(addr + 1);
            let abs = add_i8_to_u16(addr as u16 + 2, rel);
            format!("JR   Z, {}        ; jump to 0x{:04X}", rel, abs)
        }

        0x30 => {
            let rel = mmu.direct_read_i8(addr + 1);
            let abs = add_i8_to_u16(addr as u16 + 2, rel);
            format!("JR   NC, {}    ; jump to 0x{:04X}", rel, abs)
        }

        0x31 => {
            let lo = mmu.direct_read(addr + 1);
            let hi = mmu.direct_read(addr + 2);
            format!("LD   SP, ${:02X}{:02X}", hi, lo)
        }

        0x36 => format!("LD   (HL), 0x{:02X}", mmu.direct_read(addr + 1)),

        0x38 => {
            let rel = mmu.direct_read_i8(addr + 1);
            let abs = add_i8_to_u16(addr as u16 + 2, rel);
            format!("JR   C, {}        ; jump to 0x{:04X}", rel, abs)
        }

        0xBE => format!(
            "CP   (HL)  ; HL=0x{:04X} (HL)=0x{:02X}",
            mmu.reg.hl(),
            mmu.direct_read(mmu.reg.hl() as usize)
        ),

        0xC2 => format!("JP   NZ, 0x{:04X}", mmu.direct_read_u16(addr + 1)),
        0xC3 => format!("JP   0x{:04X}", mmu.direct_read_u16(addr + 1)),
        0xC4 => format!("CALL NZ, ${:04X}", mmu.direct_read_u16(addr + 1)),
        0xC6 => format!("ADD  A, 0x{:02X}", mmu.direct_read(addr + 1)),

        0xCA => format!("JP   Z, 0x{:04X}", mmu.direct_read_u16(addr + 1)),

        0xCB => {
            let regs: [&str; 8] = ["B", "C", "D", "E", "H", "L", "(HL)", "A"];
            let ops: [&str; 32] = [
                "RLC", "RRC", "RL", "RR", "SLA", "SRA", "SWAP", "SRL", "BIT 0,", "BIT 1,",
                "BIT 2,", "BIT 3,", "BIT 4,", "BIT 5,", "BIT 6,", "BIT 7,", "RES 0,", "RES 1,",
                "RES 2,", "RES 3,", "RES 4,", "RES 5,", "RES 6,", "RES 7,", "SET 0,", "SET 1,",
                "SET 2,", "SET 3,", "SET 4,", "SET 5,", "SET 6,", "SET 7,",
            ];
            let op2 = mmu.direct_read(addr + 1);
            let reg = regs[(op2 & 7) as usize];
            let mnemonic = ops[(op2 >> 4) as usize];
            format!("{} {}", mnemonic, reg)
        }

        0xCC => format!("CALL Z, 0x{:02X}", mmu.direct_read_u16(addr + 1)),
        0xCD => format!("CALL ${:04X}", mmu.direct_read_u16(addr + 1)),
        0xCE => format!("ADC  A, 0x{:02X}", mmu.direct_read(addr + 1)),

        0xD2 => format!("JP   NC, 0x{:04X}", mmu.direct_read_u16(addr + 1)),
        0xD4 => format!("CALL NC, 0x{:04X}", mmu.direct_read_u16(addr + 1)),
        0xD6 => format!("SUB  0x{:02X}", mmu.direct_read(addr + 1)),
        0xDA => format!("JP   C, 0x{:04X}", mmu.direct_read_u16(addr + 1)),
        0xDC => format!("CALL C, 0x{:02X}", mmu.direct_read_u16(addr + 1)),
        0xDD => format!("! Illegal op code: 0x{:02X}", op),
        0xDE => format!("SBC  A, 0x{:02X}", mmu.direct_read(addr + 1)),

        0xE0 => format!("LD   ($FF00+${:02X}), A", mmu.direct_read(addr + 1)),
        0xEA => format!("LD   (${:04X}), A", mmu.direct_read_u16(addr + 1)),
        0xE6 => format!("AND  ${:02X}", mmu.direct_read(addr + 1)),
        0xEC => format!("! Illegal op code: 0x{:02X}", op),
        0xED => format!("! Illegal op code: 0x{:02X}", op),
        0xEE => format!("XOR  0x{:02X}", mmu.direct_read(addr + 1)),

        0xF0 => format!("LD   A, ($FF00+${:02X})", mmu.direct_read(addr + 1)),
        0xF6 => format!("OR   0x{:02X}", mmu.direct_read(addr + 1)),
        0xF8 => format!("LD   HL, SP + ${:02X}", mmu.direct_read(addr + 1)),
        0xFA => format!("LD   A, (${:04X})", mmu.direct_read_u16(addr + 1)),
        0xFC => format!("! Illegal op code: 0x{:02X}", op),
        0xFE => format!("CP   ${:02X}", mmu.direct_read(addr + 1)),

        _ => {
            let easy = SIMPLE_MNEMONICS[op as usize];

            if !easy.is_empty() {
                return easy.to_string();
            }

            panic!(
                "invalid instruction op code at 0x{:04X}: 0x{:02X}",
                addr, op
            );
        }
    }
}

#[allow(dead_code)]
pub fn address_type(addr: u16) -> String {
    if addr < 0x4000 {
        return "ROM bank #0".to_string();
    }

    if addr < 0x8000 {
        return "ROM bank #1 (switchable)".to_string();
    }

    if addr < 0xA000 {
        return "Video RAM".to_string();
    }

    if addr < 0xC000 {
        return "Switchable RAM bank".to_string();
    }

    if addr < 0xE000 {
        return "Internal RAM (1)".to_string();
    }

    if addr < 0xFE00 {
        return "Echo of internal RAM".to_string();
    }

    if addr < 0xFEA0 {
        return "Sprite Attrib Memory (OAM)".to_string();
    }

    if addr < 0xFF00 {
        return "Empty memory block, unusable for I/O (1)".to_string();
    }

    if addr < 0xFF4C {
        return "I/O ports".to_string();
    }

    if addr < 0xFF80 {
        return "Empty memory block, unusable for I/O (2)".to_string();
    }

    if addr < 0xFFFF {
        return "Internal RAM (2)".to_string();
    }

    return "Interrupt Enable Register".to_string();
}
