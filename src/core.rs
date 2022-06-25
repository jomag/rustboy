use std::fs::File;

use egui::InputState;
use ringbuf::Producer;

pub trait Core: Sized {
    fn screen_width(&self) -> usize;
    fn screen_height(&self) -> usize;
    fn handle_press(&self); // key...
    fn handle_release(&self); // key...
    fn release_all(&mut self); // release all keys

    /// Current frame number. When incremented, the screen
    /// has been updated and need a refresh.
    fn current_frame(&self) -> usize;

    /// Log current state to file.
    /// This function is used by Debug to log state after each operation.
    fn log_state(&self, f: &mut File);

    /// Returns address of current operation being executed (program counter).
    /// For the first cycle of an operation execution this is typically
    /// equivalent to the program counter (PC).
    fn op_offset(&self) -> usize;

    /// Return current scanline
    fn scanline(&self) -> usize;

    /// Some architectures have semi-standardized operations that trigger
    /// breakpoints. For example, 0x40 ("LD B,B") on Gameboy.
    fn at_source_code_breakpoint(&self) -> bool;

    // Execute next operation
    fn exec_op(&mut self);

    fn update_input_state(&mut self, state: &InputState);

    fn register_serial_output_buffer(&mut self, p: Producer<u8>);
    fn set_audio_rates(&mut self, clock_rate: f64, sample_rate: f64);
    fn end_audio_frame(&mut self);
    fn push_audio_samples(&mut self, p: &mut Producer<i16>);

    fn to_rgba8(&self, dst: &mut Box<[u8]>, palette: Vec<(u8, u8, u8)>);

    // Return length of operation at given address in bytes
    fn op_length(&self, adr: usize) -> usize;

    // Return operation at given address formatted as a string,
    // plus the address of the next operation
    fn format_op(&self, adr: usize) -> (String, usize);

    fn read(&self, adr: usize) -> u8;
    fn write(&mut self, adr: usize, value: u8);
    fn reset(&mut self);
}
