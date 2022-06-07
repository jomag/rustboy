use egui::Context;

use crate::{
    emu::Emu,
    mmu::OAM_OFFSET,
    ppu::{OAM_OBJECT_COUNT, OAM_OBJECT_SIZE},
};

pub fn render_oam_window(ctx: &Context, emu: &mut Emu, open: &mut bool) {
    egui::Window::new("OAM")
        .open(open)
        .vscroll(true)
        .show(ctx, |ui| {
            egui::Grid::new("oam_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.heading("Address");
                    ui.heading("X");
                    ui.heading("Y");
                    ui.heading("Tile");
                    ui.heading("Behind");
                    ui.heading("Flip X");
                    ui.heading("Flip Y");
                    ui.heading("Palette");
                    ui.end_row();

                    for n in 0..OAM_OBJECT_COUNT {
                        let ob = &mut emu.mmu.ppu.oam[n];
                        ui.label(format!("#{}  {:04X}", n, OAM_OFFSET + OAM_OBJECT_SIZE * n));
                        ui.label(format!("{}", ob.x));
                        ui.label(format!("{}", ob.y));
                        ui.label(format!("{}", ob.tile_index));
                        ui.checkbox(&mut ob.bg_and_window_over_obj, "");
                        ui.checkbox(&mut ob.flip_x, "");
                        ui.checkbox(&mut ob.flip_y, "");

                        // FIXME: use cgb_palette_number if CGB
                        ui.label(format!("{}", if ob.dmg_use_second_palette { 1 } else { 0 }));

                        ui.end_row();
                    }
                });
        });
}
