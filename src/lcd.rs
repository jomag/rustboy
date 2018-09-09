
use sdl2::render::Texture;
use memory::Memory;

pub struct LCD {
    scanline_cycles: u32
}

fn render_line(scanline: u8, txt: &mut Texture, mem: &Memory) {
    txt.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        // Start point in texture
        let txt_offs = scanline as usize * pitch;

        // Tile index (X and Y)
        let mut ty: u16 = (scanline / 8) as u16;
        let mut tile_map_offset = 0x9C00 + ty * 32;

        for tx in 0..20 {
            let tile_index = mem.read(tile_map_offset + tx) as u16;
            let tile_data_offset = 0x8800 + tile_index * 16 + (scanline & 7) as u16 * 2;

            for x in 0..4 {
                let mut v = mem.read(tile_data_offset);
                v = (v >> (7 - x - x)) & 3;
                v = v * 64;
                buffer[txt_offs] = v;
                buffer[txt_offs + 1] = v;
                buffer[txt_offs + 2] = v;
            }

            for x in 0..4 {
                let mut v = mem.read(tile_data_offset + 1);
                v = (v >> (7 - x - x)) & 3;
                v = v * 64;
                buffer[txt_offs + 3] = v;
                buffer[txt_offs + 4] = v;
                buffer[txt_offs + 5] = v;
            }
        }
    }).unwrap();
}

impl LCD {
    pub fn new() -> Self {
        LCD {
            scanline_cycles: 0
        }
    }

    pub fn update(&mut self, cycles: u32, mem: &mut Memory, txt: &mut Texture) {
        self.scanline_cycles += cycles;
        if cycles > 16 {
            panic!("cycles = {}", cycles);
        }

        if self.scanline_cycles > 453 {
            self.scanline_cycles -= 453;

            let mut scanline = mem.read(0xFF44);

            if scanline < 144 {
                render_line(scanline, txt, mem);
                scanline += 1;
            } else if scanline < 153 {
                scanline += 1;
            } else {
                scanline = 0;
            }

            mem.write(0xFF44, scanline)
        }
    }
}