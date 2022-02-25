use egui::CtxRef;

use crate::emu::Emu;

pub struct CartridgeWindow {}

impl CartridgeWindow {
    pub fn new() -> Self {
        CartridgeWindow {}
    }

    pub fn render(&mut self, ctx: &CtxRef, emu: &mut Emu) {
        let c = &emu.mmu.cartridge;
        let t = &c.cartridge_type;

        egui::Window::new("Cartridge").show(ctx, |ui| {
            ui.label(format!("Cartridge type: {}", t.to_string()));
            ui.label(format!("Type code: {}", c.rom[0x147]));
            ui.label(format!("Banking mode: {}", c.mbc1.mode));
            ui.label(format!("ROM banks: {}", c.rom_bank_count()));
            ui.label(format!("ROM size: {}", c.rom_size()));
            ui.label(format!("ROM size: {} (max)", t.max_rom_size()));
            ui.label(format!("ROM offset 1: {}", c.rom_offset_0x0000_0x3fff));
            ui.label(format!("ROM offset 2: {}", c.rom_offset_0x4000_0x7fff));

            ui.label(format!("RAM size: {}", c.ram_size()));
            ui.label(format!("RAM enabled: {}", c.mbc1.ram_enabled));
            ui.label(format!("RAM bank: {}", c.selected_ram_bank()));
        });
    }
}
