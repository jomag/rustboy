use egui::Ui;
use egui_wgpu_backend::RenderPass;
use wgpu::{Device, FilterMode, Queue};

use crate::{
    emu::Emu,
    ppu::{BG_AND_WINDOW_TILE_DATA_OFFSET_0, BG_AND_WINDOW_TILE_DATA_OFFSET_1, PPU},
};

use super::pixbuf::PixBuf;

const TILES_PER_ROW: usize = 16;
const TILE_COUNT: usize = 384;
const TILE_WIDTH: usize = 8;
const TILE_HEIGHT: usize = 8;

const TILE_STRIDE: usize = 2;
const TILE_SIZE: usize = TILE_STRIDE * TILE_HEIGHT;

const PIXEL_SIZE: usize = 4;

pub fn render_tile(ppu: &PPU, adr: usize, buf: &mut PixBuf, x: usize, y: usize) {
    let top_left_offs = buf.get_offset(x, y);
    let stride = buf.get_stride();

    for row in 0..TILE_HEIGHT {
        let row_offs = top_left_offs + row * stride;
        let lo = ppu.vram[adr + row * 2];
        let hi = ppu.vram[adr + row * 2 + 1];
        for col in 0..TILE_WIDTH {
            let v = ((lo >> (7 - col)) & 1) | (((hi >> (7 - col)) & 1) << 1);
            let dst = row_offs + col * PIXEL_SIZE;
            buf.buf[dst + 0] = v * 40;
            buf.buf[dst + 1] = v * 40;
            buf.buf[dst + 2] = v * 40;
            buf.buf[dst + 3] = 255;
        }
    }
}

pub struct TileDataView {
    buf: PixBuf,
}

impl TileDataView {
    pub fn new() -> Self {
        TileDataView {
            buf: PixBuf::new(TILES_PER_ROW * TILE_WIDTH, (TILE_COUNT / TILES_PER_ROW) * 8),
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

        if let Some(texture_id) = self.buf.texture_id {
            let scale: usize = 2;
            let size = egui::Vec2::new(
                (self.buf.width * scale) as f32,
                (self.buf.height * scale) as f32,
            );
            ui.image(texture_id, size);
        }
    }
}
