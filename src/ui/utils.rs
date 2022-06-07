use egui::{Color32, Pos2, Rect, Stroke, Ui};

use crate::ppu::{PPU, TILE_HEIGHT, TILE_WIDTH};

use super::{full::PIXEL_SIZE, pixbuf::PixBuf};

pub fn render_grid(ui: &Ui, r: Rect, columns: usize, rows: usize, color: Option<Color32>) {
    let stroke = Stroke::new(
        1.0,
        match color {
            Some(c) => c,
            None => Color32::from_rgb(120, 120, 80),
        },
    );

    let step_x: f32 = r.width() / columns as f32;
    let step_y: f32 = r.height() / rows as f32;

    for n in 1..rows {
        ui.painter().line_segment(
            [
                Pos2::new(r.left(), r.top() + n as f32 * step_y),
                Pos2::new(r.right(), r.top() + n as f32 * step_y),
            ],
            stroke,
        );
    }

    for n in 1..columns {
        ui.painter().line_segment(
            [
                Pos2::new(r.left() + n as f32 * step_x, r.top()),
                Pos2::new(r.left() + n as f32 * step_x, r.bottom()),
            ],
            stroke,
        );
    }
}

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
