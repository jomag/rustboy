use std::io::Write;
use std::{collections::HashMap, fs::File};

use egui::Key;
use ringbuf::Producer;

use crate::{core::Core, gameboy::instructions::format_mnemonic};

use super::buttons::ButtonType;
use super::instructions;
use super::{
    mmu::MMU,
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH},
};

#[derive(Copy, Clone)]
pub enum Machine {
    // The original Game Boy
    GameBoyDMG,

    // Game Boy Pocket
    #[allow(dead_code)]
    GameBoyMGB,

    // Super Game Boy
    #[allow(dead_code)]
    GameBoySGB,

    // Color Game Boy
    GameBoyCGB,
}

pub struct Emu {
    pub mmu: MMU,
    pub machine: Machine,
    keymap: HashMap<Key, ButtonType>,
}

impl Core for Emu {
    fn screen_width(&self) -> usize {
        SCREEN_WIDTH
    }

    fn screen_height(&self) -> usize {
        SCREEN_HEIGHT
    }

    fn log_state(&self, f: &mut File) {
        let reg = &self.mmu.reg;
        let pc = reg.pc as usize;
        if !self.mmu.bootstrap_mode {
            let m0 = self.mmu.direct_read(pc);
            let m1 = self.mmu.direct_read(pc + 1);
            let m2 = self.mmu.direct_read(pc + 2);
            let m3 = self.mmu.direct_read(pc + 3);
            println!(
                 "A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X}) {}",
                reg.a,reg.get_f(),reg.b,reg.c,reg.d,reg.e,reg.h,reg.l,reg.sp,pc,m0,m1,m2,m3, format_mnemonic(&self.mmu, pc),
            );
            let res = writeln!(
                f, "A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X}) {}",
                reg.a,reg.get_f(),reg.b,reg.c,reg.d,reg.e,reg.h,reg.l,reg.sp,pc,m0,m1,m2,m3, format_mnemonic(&self.mmu, pc),
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

    fn at_source_code_breakpoint(&self) -> bool {
        match self.mmu.direct_read(self.mmu.reg.pc as usize) {
            0x40 => true,
            _ => false,
        }
    }

    fn exec_op(&mut self) {
        self.mmu.exec_op();
    }

    fn update_input_state(&mut self, state: &egui::InputState) {
        for key in self.keymap.keys() {
            if state.key_down(*key) {
                self.mmu.buttons.handle_press(self.keymap[&key])
            }
            if state.key_released(*key) {
                self.mmu.buttons.handle_release(self.keymap[&key])
            }
        }
    }

    fn release_all(&mut self) {
        self.mmu.buttons.release_all();
    }

    fn handle_press(&self) {
        todo!()
    }

    fn handle_release(&self) {
        todo!()
    }

    fn current_frame(&self) -> usize {
        self.mmu.ppu.frame_number
    }

    fn op_offset(&self) -> usize {
        // FIXME: only true for the first cycle of an op execution
        self.mmu.reg.pc as usize
    }

    fn scanline(&self) -> usize {
        self.mmu.ppu.ly
    }

    fn register_serial_output_buffer(&mut self, p: ringbuf::Producer<u8>) {
        self.mmu.serial.output = Some(p);
    }

    fn set_audio_rates(&mut self, clock_rate: f64, sample_rate: f64) {
        self.mmu.apu.buf_left.set_rates(clock_rate, sample_rate);
        self.mmu.apu.buf_right.set_rates(clock_rate, sample_rate);
    }

    fn end_audio_frame(&mut self) {
        self.mmu.apu.buf_left.end_frame(self.mmu.apu.buf_clock);
        self.mmu.apu.buf_clock = 0;
    }

    fn push_audio_samples(&mut self, p: &mut Producer<i16>) {
        let mut b: [i16; 128] = [0; 128];

        while self.mmu.apu.buf_left.samples_avail() > 0 {
            let n = self.mmu.apu.buf_left.read_samples(&mut b, false);
            if n == 0 {
                break;
            }
            p.push_slice(&b[..n]);
        }
    }

    fn to_rgba8(&self, dst: &mut Box<[u8]>, palette: Vec<(u8, u8, u8)>) {
        let p: [(u8, u8, u8); 4] = [palette[0], palette[1], palette[2], palette[3]];
        self.mmu.ppu.to_rgba8(dst, p);
    }

    fn op_length(&self, adr: usize) -> usize {
        if let Some(l) = instructions::op_length(self.mmu.direct_read(adr)) {
            l
        } else {
            // FIXME: this is a bad workaround for ops we don't know the length of
            0
        }
    }

    fn format_op(&self, adr: usize) -> (String, usize) {
        let text = format_mnemonic(&self.mmu, adr);

        match instructions::op_length(self.mmu.direct_read(adr)) {
            Some(len) => (text, adr + len),
            None => (text, adr + 1),
        }
    }

    fn read(&self, adr: usize) -> u8 {
        self.mmu.direct_read(adr)
    }

    fn write(&mut self, adr: usize, value: u8) {
        self.mmu.direct_write(adr, value);
    }

    fn reset(&mut self) {
        self.mmu.reset();
    }
}

impl Emu {
    pub fn new(machine: Machine) -> Self {
        Emu {
            mmu: MMU::new(machine),
            machine,
            keymap: HashMap::from([
                (Key::ArrowLeft, ButtonType::Left),
                (Key::ArrowRight, ButtonType::Right),
                (Key::ArrowUp, ButtonType::Up),
                (Key::ArrowDown, ButtonType::Down),
                (Key::Z, ButtonType::A),
                (Key::X, ButtonType::B),
                (Key::Enter, ButtonType::Start),
                (Key::Space, ButtonType::Select),
            ]),
        }
    }

    pub fn init(&mut self) {
        self.mmu.init();
    }

    pub fn load_bootstrap(&mut self, path: &str) -> usize {
        self.mmu.load_bootstrap(&path)
    }

    pub fn load_cartridge(&mut self, path: &str) {
        self.mmu.load_cartridge(path);
    }
}
