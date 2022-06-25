use crate::{
    core::Core,
    cpu::{
        self,
        cpu_6510::{disassemble_one, op_len, CPU, OPS},
    },
    MemoryMapped,
};

use super::{bus::Bus, mmu::MMU};

const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;

#[derive(Copy, Clone)]
pub enum Machine {
    // The original, breadbin C64
    C64,
}

pub struct CoreC64 {
    pub mmu: MMU,
    pub machine: Machine,
    pub cpu: cpu::cpu_6510::CPU,
    pub bus: Bus,

    // Temporary hack: increments on every read of current_frame()
    // so that the screen is updated after every operation.
    // This hack is required until we have some working graphics.
    mock_frame_count: usize,
}

impl Core for CoreC64 {
    fn screen_width(&self) -> usize {
        SCREEN_WIDTH
    }

    fn screen_height(&self) -> usize {
        SCREEN_HEIGHT
    }

    fn handle_press(&self) {
        todo!()
    }

    fn handle_release(&self) {
        todo!()
    }

    fn release_all(&mut self) {
        todo!()
    }

    fn current_frame(&self) -> usize {
        self.mock_frame_count
    }

    fn log_state(&self, _: &mut std::fs::File) {
        todo!()
    }

    fn op_offset(&self) -> usize {
        self.cpu.op_offset() as usize
    }

    fn scanline(&self) -> usize {
        todo!()
    }

    fn at_source_code_breakpoint(&self) -> bool {
        todo!()
    }

    fn exec_op(&mut self) {
        self.mock_frame_count += 1;
        self.cpu.exec(&mut self.bus);
    }

    fn update_input_state(&mut self, _state: &egui::InputState) {}

    fn register_serial_output_buffer(&mut self, _p: ringbuf::Producer<u8>) {
        println!("C64 serial not implemented.");
    }

    fn set_audio_rates(&mut self, _clock_rate: f64, _sample_rate: f64) {
        println!("C64 audio not implemented.");
    }

    fn end_audio_frame(&mut self) {}

    fn push_audio_samples(&mut self, _p: &mut ringbuf::Producer<i16>) {}

    fn to_rgba8(&self, _dst: &mut Box<[u8]>, _palettee: Vec<(u8, u8, u8)>) {}

    fn op_length(&self, adr: usize) -> usize {
        let code = self.bus.read(adr);
        let op = &OPS[usize::from(code)];
        op_len(&op.adr) as usize
    }

    fn format_op(&self, adr: usize) -> (String, usize) {
        let mut next: usize = 0;
        let text = disassemble_one(&self.bus, adr, &mut next);
        (text, next)
    }

    fn read(&self, adr: usize) -> u8 {
        self.bus.read(adr)
    }

    fn write(&mut self, adr: usize, value: u8) {
        self.bus.write(adr, value);
    }

    fn reset(&mut self) {
        self.cpu.reset(&self.bus);
    }
}

impl CoreC64 {
    pub fn new(machine: Machine) -> Self {
        CoreC64 {
            mmu: MMU::new(machine),
            machine,
            mock_frame_count: 0,
            cpu: CPU::new(),
            bus: Bus::new(),
        }
    }

    pub fn init(&mut self) {
        self.cpu.reset(&self.bus);
    }
}
