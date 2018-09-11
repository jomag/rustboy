
use sdl2::render::Texture;
use memory::{ LCDC_REG, LY_REG, SCY_REG, Memory };

// Bits of LCDC register:
// 7 - LCD Enable (0 = off, 1 = on)
// 6 - Window Tile Map Address (0 = 0x9800..0x9BFF, 1 = 0x9C00..0x9FFF)
// 5 - Window enabled (0 = off, 1 = on)
// 4 - BG & Window Tile Data (0 = 0x8800..0x97FF, 1 = 0x8000..0x8FFF)
// 3 - BG Tile Map Address (0 = 0x9800..0x9BFF, 1 = 0x9C00..0x9FFF)
// 2 - OBJ Size (0 = 8x8, 1 = 8x16)
// 1 - OBJ Enable (0 = off, 1 = on)
// 0 - BG Mode (0 = BG Display Off, 1 = BG Display On)

// tmp: 1001 0001

pub struct LCD {
    scanline_cycles: u32
}

fn render_line(scanline: u8, txt: &mut Texture, mem: &Memory) {
    // println!("LCDC: 0x{:02x}", mem.read(LCDC_REG));
    let lcdc = mem.read(LCDC_REG);

    txt.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        // Start point in texture
        let mut txt_offs = scanline as usize * pitch;

        let y: u32 = scanline as u32 + mem.read(SCY_REG) as u32;

        let mut ty: u16 = (y / 8) as u16;
        let mut tile_map_offset = ty * 32;

        // Bit 3 of LCDC selects bg tile map address
        if lcdc & 8 == 0 {
            tile_map_offset += 0x9800;
        } else {
            tile_map_offset += 0x9C00;
        }

        for tx in 0..20 {
            let tile_index = mem.read(tile_map_offset + tx) as u16;
            let mut tile_data_offset = tile_index * 16 + (y & 7) as u16 * 2;

            if lcdc & 16 == 0 {
                tile_data_offset += 0x8800;
            } else {
                tile_data_offset += 0x8000;
            }

            let b1 = mem.read(tile_data_offset);
            let b2 = mem.read(tile_data_offset + 1);

            for x in 0..8 {
                let hi = b1 & (1 << (7 - x)) != 0;
                let lo = b2 & (1 << (7 - x)) != 0;
                let mut v = 0;
                if hi { v += 128; }
                if lo { v += 64; }
                buffer[txt_offs] = v;
                buffer[txt_offs + 1] = v;
                buffer[txt_offs + 2] = v;
                txt_offs += 3;
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

            let mut scanline = mem.read(LY_REG);

            if scanline < 144 {
                render_line(scanline, txt, mem);
                scanline += 1;
            } else if scanline < 153 {
                scanline += 1;
            } else {
                scanline = 0;
            }

            mem.write(LY_REG, scanline);
        }
    }
}