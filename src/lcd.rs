
use sdl2::render::Texture;
use mmu::{ LCDC_REG, LY_REG, SCY_REG, IF_REG, MMU };
use interrupt::{ IF_VBLANK_BIT, IF_LCDC_BIT };

// Bits of LCDC register:
// 7 - LCD Enable (0 = off, 1 = on)
// 6 - Window Tile Map Address (0 = 0x9800..0x9BFF, 1 = 0x9C00..0x9FFF)
// 5 - Window enabled (0 = off, 1 = on)
// 4 - BG & Window Tile Data (0 = 0x8800..0x97FF, 1 = 0x8000..0x8FFF)
// 3 - BG Tile Map Address (0 = 0x9800..0x9BFF, 1 = 0x9C00..0x9FFF)
// 2 - OBJ Size (0 = 8x8, 1 = 8x16)
// 1 - OBJ Enable (0 = off, 1 = on)
// 0 - BG Mode (0 = BG Display Off, 1 = BG Display On)

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const BUFFER_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 3;

pub struct LCD {
    scanline_cycles: u32,

    pub scanline: u8,

    pub lcdc: u8,
    pub lyc: u8,
    pub scy: u8,
    pub scx: u8,

    // Current mode (2 bits)
    // 00 - Horizontal blanking
    // 01 - Vertical blanking
    // 10 - Using OAM
    // 11 - Using OAM and RAM
    mode: u8,

    // Interrupt mode selection
    isel_mode00: bool,
    isel_mode01: bool,
    isel_mode10: bool,
    isel_ly: bool,  // When LY == LYC

    // 8k of display RAM (address 0x8000-0x9FFF)
    ram: [u8; 8192],

    // Buffer to hold all pixel data
    //
    // Each scanline is rendered to this buffer and then
    // a texture is locked just once per screen refresh
    // for much better performance than drawing directly
    // to the texture.
    //
    // The pixel format is RGB24, 3 bytes per pixel.
    buf: [u8; BUFFER_SIZE],

    // Interrupt Request
    pub irq: u8
}


impl LCD {
    pub fn new() -> Self {
        LCD {
            scanline_cycles: 0,
            scanline: 0,
            lcdc: 0,
            lyc: 0,
            scy: 0,
            scx: 0,
            mode: 0,
            isel_mode00: false,
            isel_mode01: false,
            isel_mode10: false,
            isel_ly: false,
            ram: [0; 8192],
            buf: [0; BUFFER_SIZE],
            irq: 0
        }
    }

    pub fn get_stat_reg(&self) -> u8 {
        let mut stat = self.mode;
        if self.scanline == self.lyc { stat |= 4 }
        if self.isel_mode00 { stat |= 8 }
        if self.isel_mode01 { stat |= 16 }
        if self.isel_mode10 { stat |= 32 }
        if self.isel_ly { stat |= 64 }
        stat
    }

    pub fn set_stat_reg(&mut self, v: u8) {
        self.isel_mode00 = (v & 8) != 0;
        self.isel_mode01 = (v & 16) != 0;
        self.isel_mode10 = (v & 32) != 0;
        self.isel_ly = (v & 64) != 0;
    }

    fn render_line(&mut self, scanline: u8) {
        // Length of one row of pixels in bytes
        let pitch = SCREEN_WIDTH * 3;

        // Start point in texture
        let mut buf_offs = scanline as usize * pitch;

        let y: u8 = scanline.wrapping_add(self.scy);

        let ty: u16 = (y / 8) as u16;
        let mut tile_map_offset = ty * 32;

        // Bit 3 of LCDC selects bg tile map address
        if self.lcdc & 8 == 0 {
            tile_map_offset += 0x9800 - 0x8000;
        } else {
            tile_map_offset += 0x9C00 - 0x8000;
        }

        for tx in 0..20 {
            let tile_index = self.ram[(tile_map_offset + tx) as usize] as u16;
            let mut tile_data_offset = tile_index * 16 + (y & 7) as u16 * 2;

            if self.lcdc & 16 == 0 {
                tile_data_offset += 0x8800 - 0x8000;
            } else {
                tile_data_offset += 0x8000 - 0x8000;
            }

            let b1 = self.ram[tile_data_offset as usize];
            let b2 = self.ram[(tile_data_offset + 1) as usize];

            for x in 0..8 {
                let hi = b1 & (1 << (7 - x)) != 0;
                let lo = b2 & (1 << (7 - x)) != 0;
                let mut v = 0;
                if hi { v += 128; }
                if lo { v += 64; }
                self.buf[buf_offs] = v;
                self.buf[buf_offs + 1] = v;
                self.buf[buf_offs + 2] = v;
                buf_offs += 3;
            }
        }
    }

    // Each line takes 456 cycles (114 clocks) to draw.
    // This time is split in three parts:
    //
    // Clock 0 - 20: OAM search
    // Clock 20 - 63: Pixel transfer
    // Clock 63 - 114: H blank
    //
    // The first two bits of the STAT register holds
    // the current mode, and the mode correlates to the
    // three stages, plus one for vertical blanking:
    //
    // OAM Search: Mode 2 (10)
    // Pixel transfer: Mode 3 (11)
    // H blank: Mode 0 (00)
    // V blank: Mode 1 (01)
    //
    // The display is 144 lines high. When all 144 lines
    // have been rendered the v-blank period starts and
    // it lasts for the same time it would take to draw
    // another 10 lines
    pub fn step(&mut self, mmu: &mut [u8; 0x10000]) -> bool {
        let mut display_update = false;

        if self.scanline < 144 {
            match self.scanline_cycles {
                0 => {
                    // OAM search. Mode: 2
                    self.mode = 2;
                    if self.isel_mode10 {
                        self.irq |= IF_LCDC_BIT;
                    }
                    self.scanline_cycles += 1;
                }

                80 => {
                    // Transfer data to LCD. Mode: 3
                    self.mode = 3;
                    self.scanline_cycles += 1;
                }

                252 => {
                    // Horizontal blanking
                    if self.mode != 0 {
                        self.mode = 0;
                        if self.isel_mode00 {
                            self.irq |= IF_LCDC_BIT;
                        }

                        let scanline = self.scanline;
                        self.render_line(scanline);
                    }
                    self.scanline_cycles += 1;
                }

                456 => {
                    // End of line. Start next.
                    self.scanline += 1;
                    self.scanline_cycles = 0;
                }

                _ => {
                    self.scanline_cycles += 1;
                }
            }
        } else {
            self.scanline_cycles += 1;

            if self.scanline_cycles == 456 {
                self.scanline += 1;
                self.scanline_cycles = 0;

                if self.scanline == 154 {
                    self.irq |= IF_VBLANK_BIT;
                    display_update = true;
                    self.scanline = 0;
                }
            }
        }

        display_update
    }

    pub fn update(&mut self, cycles: u32, mmu: &mut [u8; 0x10000]) -> bool {
        let mut display_update = false;
        for _ in 0..cycles {
            display_update = display_update || self.step(mmu);
        }
        display_update
    }

    pub fn old_update(&mut self, cycles: u32, mmu: &mut [u8; 0x10000]) -> bool {
        let mut display_update = false;
        self.scanline_cycles += cycles;

        if cycles > 16 {
            panic!("cycles = {}", cycles);
        }

        if self.scanline_cycles < 80 {
            // OAM search. Mode: 2
            if self.mode != 2 {
                self.mode = 2;
                if self.isel_mode10 {
                    self.irq |= IF_LCDC_BIT;
                }
            }
        } else if self.scanline_cycles < 80 + 172 {
            // Transfer data to LCD. Mode: 3
            if self.mode != 3 {
                self.mode = 3;
            }
        } else if self.scanline_cycles < 80 + 172 + 204 {
            // Horizontal blanking
            if self.mode != 0 {
                self.mode = 0;
                if self.isel_mode00 {
                    self.irq |= IF_LCDC_BIT;
                }

                let scanline = self.scanline;
                self.render_line(scanline);
            }
        } else {
            self.scanline += 1;
            self.scanline_cycles -= 456;
        }

        return display_update

        // Old code:
        /*
        if self.scanline_cycles > 453 {
            let mode = self.stat & 3;
            self.scanline_cycles -= 453;

            if self.scanline < 144 {

                if mode != 

            } else if self.scanline < 153 {
                self.scanline += 1;
            } else {
                self.irq |= IF_VBLANK_BIT;
                self.scanline = 0;
            }

            (self.scanline == 0)
        } else {
            false
        }
        */
    }

    pub fn read_display_ram(&self, address: u16) -> u8 {
        self.ram[address as usize - 0x8000]
    }

    pub fn write_display_ram(&mut self, address: u16, value: u8) {
        self.ram[address as usize - 0x8000] = value;
    }

    pub fn copy_to_texture(&self, txt: &mut Texture) {
        txt.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            buffer.copy_from_slice(&self.buf);
        }).unwrap();
    }
}