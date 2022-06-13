use egui::{Label, Ui};
use egui_wgpu_backend::RenderPass;
use wgpu::{Device, Queue};

use crate::{emu::Emu, ppu::PPU};

use super::{
    pixbuf::PixBuf,
    utils::{render_grid, render_tile},
};

const TILES_PER_ROW: usize = 16;
const TILE_COUNT: usize = 384;
const TILE_WIDTH: usize = 8;
const TILE_HEIGHT: usize = 8;

const TILE_STRIDE: usize = 2;
const TILE_SIZE: usize = TILE_STRIDE * TILE_HEIGHT;

pub struct TileDataView {
    buf: PixBuf,
    grid: bool,
}

impl TileDataView {
    pub fn new() -> Self {
        TileDataView {
            buf: PixBuf::new(TILES_PER_ROW * TILE_WIDTH, (TILE_COUNT / TILES_PER_ROW) * 8),
            grid: false,
        }
    }

    pub fn init(&mut self, device: &Device, rpass: &mut RenderPass) {
        self.buf.init(device, rpass);
    }

    fn render_texture(&mut self, ppu: &PPU) {
        for row in 0..(TILE_COUNT / TILES_PER_ROW) {
            for col in 0..TILES_PER_ROW {
                render_tile(
                    ppu,
                    (row * TILES_PER_ROW + col) * TILE_SIZE,
                    &mut self.buf,
                    col * TILE_WIDTH,
                    row * TILE_HEIGHT,
                )
            }
        }
        self.buf.dirty = true;
    }

    pub fn render(&mut self, ui: &mut Ui, emu: &mut Emu, queue: &Queue) {
        self.render_texture(&emu.mmu.ppu);
        self.buf.prepare(queue);

        ui.horizontal(|ui| ui.checkbox(&mut self.grid, "Show grid"));

        if let Some(texture_id) = self.buf.texture_id {
            let scale: usize = 2;
            let size = egui::Vec2::new(
                (self.buf.width * scale) as f32,
                (self.buf.height * scale) as f32,
            );

            let resp = ui.image(texture_id, size);
            if self.grid {
                render_grid(
                    ui,
                    resp.rect,
                    TILES_PER_ROW,
                    TILE_COUNT / TILES_PER_ROW,
                    None,
                );
            }

            match resp.hover_pos() {
                Some(p) => {
                    let col = (p[0] - resp.rect.left()) as usize / (8 * scale);
                    let row = (p[1] - resp.rect.top()) as usize / (8 * scale);
                    resp.on_hover_ui_at_pointer(|ui| {
                        let idx = row * TILES_PER_ROW + col;
                        ui.add(Label::new(format!(
                            "Index: {}, Address: 0x{:04x}",
                            idx,
                            idx * TILE_SIZE + 0x8000
                        )));
                    });
                }
                None => {}
            }
        }
    }
}
