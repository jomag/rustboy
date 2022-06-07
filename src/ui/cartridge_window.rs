use egui::Context;

use crate::emu::Emu;

pub struct CartridgeWindow {}

impl CartridgeWindow {
    pub fn new() -> Self {
        CartridgeWindow {}
    }

    pub fn render(&mut self, ctx: &Context, emu: &mut Emu, open: &mut bool) {
        let c = &emu.mmu.cartridge;
        let t = &c.cartridge_type();

        egui::Window::new("Cartridge").open(open).show(ctx, |ui| {
            ui.label(format!("Cartridge type: {}", t.to_string()));
            ui.label(format!("Type code: {}", c.read_abs(0x147)));
            ui.label(format!("Licensee: {}", c.header().licensee()));
            ui.label(format!("ROM banks: {}", c.header().rom_bank_count));
            ui.label(format!("ROM size: {}", c.header().rom_size));
            ui.label(format!("ROM size: {} (max)", t.max_rom_size()));
            ui.label(format!("RAM size: {}", c.header().ram_size));
        });
    }
}
