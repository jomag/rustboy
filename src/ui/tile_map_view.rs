use egui::{Label, Ui};
use egui_wgpu_backend::RenderPass;
use wgpu::{Device, Queue};

use crate::gameboy::{
    emu::Emu,
    ppu::{
        get_tile_data_offset, TileAddressingMode, PPU, TILE_COLUMNS, TILE_HEIGHT, TILE_ROWS,
        TILE_WIDTH,
    },
};

use super::{
    pixbuf::PixBuf,
    utils::{render_grid, render_tile},
};

pub enum TileMapArea {
    // Use the current background tile map area
    AutoBG,

    // Use the current window tile map area
    AutoWindow,

    // Fixed memory offset
    Fixed(usize),
}

pub struct TileMapView {
    buf: PixBuf,

    // Memory offset to the tile map area
    // Typical values: 0x9800 or 0x9C00
    tile_map_area: TileMapArea,

    // Fixed tile data addressing mode. If none, the
    // current addressing mode of the PPU is used.
    tile_addressing_mode: Option<TileAddressingMode>,

    grid: bool,
}

impl TileMapView {
    pub fn new() -> Self {
        TileMapView {
            buf: PixBuf::new(TILE_COLUMNS * TILE_WIDTH, TILE_ROWS * TILE_HEIGHT),
            tile_map_area: TileMapArea::AutoBG,
            tile_addressing_mode: None,
            grid: false,
        }
    }

    pub fn init(&mut self, device: &Device, rpass: &mut RenderPass) {
        self.buf.init(device, rpass);
    }

    // Find tile data offset at given row and column.
    // Returns the tile index and tile data offset.
    fn get_tile_data_offset(&self, col: usize, row: usize, ppu: &PPU) -> (u8, usize) {
        let map_offs = match self.tile_map_area {
            TileMapArea::AutoBG => ppu.bg_tile_map_offset,
            TileMapArea::AutoWindow => ppu.window_tile_map_offset,
            TileMapArea::Fixed(o) => o,
        };

        let mode = match self.tile_addressing_mode {
            Some(m) => m,
            None => ppu.tile_addressing_mode,
        };

        let idx = ppu.vram[map_offs - 0x8000 + row * TILE_COLUMNS + col];
        let offs = get_tile_data_offset(idx, mode) - 0x8000;

        return (idx, offs);
    }

    fn render_texture(&mut self, ppu: &PPU) {
        for row in 0..TILE_ROWS {
            for col in 0..TILE_COLUMNS {
                let (_, offs) = self.get_tile_data_offset(col, row, ppu);
                render_tile(
                    ppu,
                    offs,
                    &mut self.buf,
                    col * TILE_WIDTH,
                    row * TILE_HEIGHT,
                )
            }
        }

        self.buf.dirty = true;
    }

    pub fn render(&mut self, ui: &mut Ui, emu: &mut Emu, queue: &Queue) {
        let scale: usize = 2;
        self.render_texture(&emu.mmu.ppu);
        self.buf.prepare(queue);

        if let Some(texture_id) = self.buf.texture_id {
            let size = egui::Vec2::new(
                (self.buf.width * scale) as f32,
                (self.buf.height * scale) as f32,
            );

            ui.horizontal(|ui| {
                let sel = match self.tile_addressing_mode {
                    None => 0,
                    Some(TileAddressingMode::Secondary) => 1,
                    Some(TileAddressingMode::Primary) => 2,
                };

                if ui.radio(sel == 0, "Auto").clicked() {
                    self.tile_addressing_mode = None;
                }

                if ui.radio(sel == 1, "0x8000 (u8)").clicked() {
                    self.tile_addressing_mode = Some(TileAddressingMode::Secondary)
                }

                if ui.radio(sel == 2, "0x8800 (i8)").clicked() {
                    self.tile_addressing_mode = Some(TileAddressingMode::Primary)
                }
            });

            ui.horizontal(|ui| {
                let sel = match self.tile_map_area {
                    TileMapArea::AutoBG => 0,
                    TileMapArea::AutoWindow => 1,
                    TileMapArea::Fixed(0x9800) => 2,
                    TileMapArea::Fixed(0x9C00) => 3,
                    _ => panic!("invalid tile map area"),
                };

                if ui.radio(sel == 0, "Auto (BG)").clicked() {
                    self.tile_map_area = TileMapArea::AutoBG;
                }
                if ui.radio(sel == 1, "Auto (Win)").clicked() {
                    self.tile_map_area = TileMapArea::AutoWindow;
                }
                if ui.radio(sel == 2, "0x9800..0x9BFF").clicked() {
                    self.tile_map_area = TileMapArea::Fixed(0x9800);
                }
                if ui.radio(sel == 3, "0x9C00..0x9FFF").clicked() {
                    self.tile_map_area = TileMapArea::Fixed(0x9C00);
                }
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.grid, "Show grid");
            });

            let resp = ui.image(texture_id, size);
            if self.grid {
                render_grid(ui, resp.rect, TILE_COLUMNS, TILE_ROWS, None);
            }

            match resp.hover_pos() {
                Some(p) => {
                    let col = (p[0] - resp.rect.left()) as usize / (8 * scale);
                    let row = (p[1] - resp.rect.top()) as usize / (8 * scale);
                    resp.on_hover_ui_at_pointer(|ui| {
                        let (idx, offs) = self.get_tile_data_offset(col, row, &emu.mmu.ppu);
                        ui.add(Label::new(format!(
                            "Index: {}, Data: 0x{:04x}",
                            idx,
                            offs + 0x8000
                        )));
                    });
                }
                None => {}
            }
        }
    }
}
