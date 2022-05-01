// TODO:
// - copy all IRQ behavior from lcd.rs

// The PPU is always in one of the following modes:
//
// Mode 2: OAM scan
// Mode 3: PixelTransfer pixels
// Mode 0: Horizontal blank
// Mode 1: Vertical blank
//
// The modes and timing for each scanline is the following:
//
// +---------+-----------------------------------+-------------+
// | Mode 2  | Mode 3                            | Mode 0      |
// +---------+-----------------------------------+-------------+
// | 80 dots | 172-289 dots                      | 87-204 dots |
// | <------------ OAM inaccessible -----------> |             |
// |         | <-- VRAM inaccessible ----------> |             |
// |         | <-- CGB palettes inaccessible --> |             |
// +---------+-----------------------------------+-------------+
//
//  Total dots per scanline is always 456.
//
// Timing:
// Pandocs use the term "dot" for the shortest period over which
// the PPU can output a pixel. It is equivalent to one T-cycle on
// DMG and CGB in single-speed mode. For CGB in double-speed mode
// it is equivalent to 2 T-cycles.
//
// Implementation details:
//
// - Since the OAM memory is inaccessible during mode 2 I assume that
//   all the OAM search can be done immediately as the PPU enters
//   that mode.

use crate::{
    interrupt::IF_VBLANK_BIT,
    mmu::{
        MemoryMapped, BGP_REG, LCDC_REG, LYC_REG, LY_REG, OAM_OFFSET, OBP0_REG, OBP1_REG, SCX_REG,
        SCY_REG, STAT_REG,
    },
};

use super::fifo::FIFO;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const OAM_SIZE: usize = 0xA0;
pub const OAM_OBJECT_SIZE: usize = 4;
pub const OAM_END: usize = OAM_OFFSET + OAM_SIZE - 1;
pub const VRAM_SIZE: usize = 0x2000;
pub const VRAM_OFFSET: usize = 0x8000;
pub const VRAM_END: usize = VRAM_OFFSET + VRAM_SIZE - 1;
pub const MAX_SPRITES_PER_SCANLINE: usize = 10;

pub const WINDOW_TILE_MAP_OFFSET_0: usize = 0x9800;
pub const WINDOW_TILE_MAP_OFFSET_1: usize = 0x9C00;
pub const BG_TILE_MAP_OFFSET_0: usize = 0x9800;
pub const BG_TILE_MAP_OFFSET_1: usize = 0x9C00;
pub const BG_AND_WINDOW_TILE_DATA_OFFSET_0: usize = 0x8800;
pub const BG_AND_WINDOW_TILE_DATA_OFFSET_1: usize = 0x8000;

enum Mode {
    HorizontalBlank,
    VerticalBlank,
    OAMSearch,
    PixelTransfer,
}

enum FetchState {
    FetchTileNo = 0,
    FetchTileDataLow = 1,
    FetchTileDataHigh = 2,
    PushToFIFO = 3,
}

struct Fetcher {
    state: FetchState,
    tile_index: usize,
    tile_id: usize,
    tile_line: usize,
    tile_data_low: u8,
    tile_data_high: u8,
    fifo: FIFO,
}

/// Struct used for the OAM memory
#[derive(Copy, Clone)]
struct Sprite {
    // Y position. Byte 0.
    y: u8,

    // X position. Byte 1.
    x: u8,

    // Tile index. Byte 2.
    tile_index: u8,

    // If true, BG and Window is rendered on top of sprite. Byte 3, bit 7.
    bg_and_window_over_obj: bool,

    // If true, sprite should be flipped vertically. Byte 3, bit 6.
    flip_y: bool,

    // If true, sprite should be flipped horizontally. Byte 3, bit 5.
    flip_x: bool,

    // If true, second palette (OBP1) should be used instead of OBP0.
    // Not for CGB. Byte 3, bit 4.
    dmg_use_second_palette: bool,

    // If true, second VRAM bank is used. CGB only. Byte 3, bit 3.
    tile_vram_bank: bool,

    // Which palette to use. CGB only. Byte 3, bit 0-2.
    cgb_palette_number: u8,
}

impl Default for Sprite {
    fn default() -> Sprite {
        Sprite {
            x: 0,
            y: 0,
            tile_index: 0,
            bg_and_window_over_obj: false,
            flip_y: false,
            flip_x: false,
            dmg_use_second_palette: false,
            tile_vram_bank: false,
            cgb_palette_number: 0,
        }
    }
}

impl Sprite {
    fn read(&self, offset: usize) -> u8 {
        match offset {
            0 => self.x,
            1 => self.y,
            2 => self.tile_index,
            3 => {
                let mut v = self.cgb_palette_number;
                if self.bg_and_window_over_obj {
                    v |= 128;
                }
                if self.flip_y {
                    v |= 64;
                }
                if self.flip_x {
                    v |= 32;
                }
                if self.dmg_use_second_palette {
                    v |= 16;
                }
                if self.tile_vram_bank {
                    v |= 8;
                }
                v
            }
            _ => panic!("Invalid offset"),
        }
    }

    fn write(&mut self, offset: usize, value: u8) {
        match offset {
            0 => self.x = value,
            1 => self.y = value,
            2 => self.tile_index = value,
            3 => {
                self.bg_and_window_over_obj = (value & 128) != 0;
                self.flip_y = (value & 64) != 0;
                self.flip_x = (value & 32) != 0;
                self.dmg_use_second_palette = (value & 16) != 0;
                self.tile_vram_bank = (value & 8) != 0;
                self.cgb_palette_number = value & 7;
            }
            _ => panic!("Invalid offset"),
        }
    }
}

pub struct PPU {
    // LCD + PPU enabled. Bit 7 in LCDC.
    enabled: bool,

    // Offset to the window tile map. Controlled through LCDC, bit 6:
    // 0: 9800..9BFF
    // 1: 9C00..9FFF
    window_tile_map_offset: usize,

    // Window area enabled. Controlled through LCDC, bit 5.
    window_enabled: bool,

    // Offset to BG and window tile data. Controlled through LCDC, bit 4:
    // 0: 8800..97FF
    // 1: 8000..8FFF
    bg_and_window_tile_data_offset: usize,

    // Offset to the BG tile map. Controlled through LCDC, bit 3:
    // 0: 9800..9BFF
    // 1: 9C00..9FFF
    bg_tile_map_offset: usize,

    // Size of objects (sprites). Controlled through LCDC, bit 2:
    // 0: 8x8
    // 1: 8x16
    objects_are_8x16: bool,

    // Objects (sprites) enabled status. Controlled through LCDC, bit 1.
    objects_enabled: bool,

    // LY compare interrupt enabled.
    // Accessed through register STAT, bit 6.
    lyc_interrupt_enabled: bool,

    // Mode 2 (OAM search) interrupt enabled. Register: STAT, bit 5
    oam_search_interrupt_enabled: bool,

    // Mode 1 (VBlank) interrupt enabled. Register: STAT, bit 4
    vblank_interrupt_enabled: bool,

    // Mode 0 (HBlank) interrupt enabled. RegisteR: STAT, bit 3
    hblank_interrupt_enabled: bool,

    // Various meaning. Controlled through LCDC, bit 0.
    // TODO: document me!
    bg_and_window_enable_prio: bool,

    // Video RAM (0x8000..0x9FFF)
    vram: [u8; VRAM_SIZE],

    // Buffer for final pixel data.
    // Each byte in the buffer holds the final color plus
    // some extra meta-data:
    //
    // Bit 7..6: Color source
    //           00: Background
    //           01: Window
    //           10: Sprite
    // Bit 0..1: Color (DMG). 0 = darkest, 3 = lightest
    pub buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],

    // Interrupt Request
    pub irq: u8,

    // OAM - Object Attribute Memory
    oam: [Sprite; OAM_SIZE / OAM_OBJECT_SIZE],

    // Current mode (0-3)
    mode: Mode,

    // Current horizontal line being rendered
    ly: usize,

    // X-coordinate of the pixel currently being rendered
    lx: usize,

    scanline_timer: usize,

    // Selected OAM objects (sprites) for current scanline. Max 10.
    scanline_objects: [u8; 10],

    // OAM object selection count for current scanline. Max 10.
    scanline_object_count: u8,

    // Pointer to the next pixel to be written to.
    // Reset to zero on every vblank.
    buffer_pos: usize,

    fetcher: Fetcher,

    // Assigns gray shades for bg and window color indexes. DMG only.
    // Accessed through register BGP (0xFF47).
    bg_palette: [u8; 4],

    // First object palette. Accessed through register OBP0.
    obj0_palette: [u8; 4],

    // Second object palette. Accessed through register OBP1.
    obj1_palette: [u8; 4],

    // Scroll Y. Accessed through register SCY (0xFF42)
    scy: usize,

    // Scroll X. Accessed through register SCX (0xFF43)
    scx: usize,

    // LY compare register.
    ly_compare: usize,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            irq: 0,
            vram: [0; VRAM_SIZE],
            buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            oam: [Sprite::default(); OAM_SIZE / OAM_OBJECT_SIZE],
            fetcher: Fetcher {
                state: FetchState::FetchTileNo,
                tile_index: 0,
                tile_id: 0,
                tile_line: 0,
                tile_data_low: 0,
                tile_data_high: 0,
                fifo: FIFO::new(),
            },
            mode: Mode::OAMSearch,
            ly: 0,
            scanline_timer: 0,
            lx: 0,
            buffer_pos: 0,

            // FIXME: or should it be initialized to all zeros?
            bg_palette: [0, 1, 2, 3],
            obj0_palette: [0, 1, 2, 3],
            obj1_palette: [0, 1, 2, 3],
            scx: 0,
            scy: 0,
            enabled: false,
            window_tile_map_offset: WINDOW_TILE_MAP_OFFSET_0,
            window_enabled: false,
            bg_and_window_tile_data_offset: BG_AND_WINDOW_TILE_DATA_OFFSET_0,
            bg_tile_map_offset: BG_TILE_MAP_OFFSET_0,
            objects_are_8x16: false,
            objects_enabled: false,
            bg_and_window_enable_prio: false,
            lyc_interrupt_enabled: false,
            oam_search_interrupt_enabled: false,
            hblank_interrupt_enabled: false,
            vblank_interrupt_enabled: false,
            ly_compare: 0,
            scanline_objects: [0; MAX_SPRITES_PER_SCANLINE],
            scanline_object_count: 0,
        }
    }

    pub fn init_fetcher(&mut self) {
        self.fetcher.state = FetchState::FetchTileNo;
        self.fetcher.tile_index = 0;
        self.fetcher.fifo.clear();
    }

    fn select_scanline_objects(&mut self) {}

    pub fn step_fetcher_2t(&mut self) {
        // Background fetch
        match self.fetcher.state {
            FetchState::FetchTileNo => {
                let tile_map_offset =
                    self.bg_tile_map_offset - 0x8000 + ((self.scy + self.ly) / 8) * 32;
                self.fetcher.tile_id =
                    self.vram[tile_map_offset + self.fetcher.tile_index] as usize;
                self.fetcher.state = FetchState::FetchTileDataLow;
            }

            FetchState::FetchTileDataLow => {
                let offset = self.fetcher.tile_id * 16 + self.fetcher.tile_line * 2;
                let offset = offset + self.bg_and_window_tile_data_offset - 0x8000;
                //println!(
                //    "tile index: {}, tile id: {}. offset: {:x}",
                //    self.fetcher.tile_index, self.fetcher.tile_id, offset,
                //);
                self.fetcher.tile_data_low = self.vram[offset];
                self.fetcher.state = FetchState::FetchTileDataHigh;
            }

            FetchState::FetchTileDataHigh => {
                let offset = self.fetcher.tile_id * 16 + self.fetcher.tile_line * 2 + 1;
                let offset = offset + self.bg_and_window_tile_data_offset - 0x8000;
                self.fetcher.tile_data_high = self.vram[offset];
                self.fetcher.state = FetchState::PushToFIFO;
            }

            FetchState::PushToFIFO => {
                let lo = self.fetcher.tile_data_low;
                let hi = self.fetcher.tile_data_high;
                if self.fetcher.fifo.len() <= 8 {
                    for i in 0..8 {
                        let pxl = ((lo >> (7 - i)) & 1) | (((hi >> (7 - i)) & 1) << 1);
                        self.fetcher.fifo.enqueue(pxl as u8, 0, 0, false);
                    }
                    self.fetcher.tile_index += 1;
                    self.fetcher.state = FetchState::FetchTileNo;
                }
            }
        }
    }

    pub fn step_2t(&mut self) -> bool {
        self.scanline_timer += 2;

        match self.mode {
            Mode::OAMSearch => {
                // TODO: implement OAM search
                if self.scanline_timer == 80 {
                    self.lx = 0;
                    self.init_fetcher();
                    self.fetcher.tile_line = (self.scy + self.ly) % 8;
                    self.mode = Mode::PixelTransfer;
                }
            }

            Mode::PixelTransfer => {
                self.step_fetcher_2t();
                if self.fetcher.fifo.len() > 0 {
                    let data = self.fetcher.fifo.dequeue();
                    self.buffer[self.buffer_pos] = data.0;
                    self.buffer_pos += 1;
                    self.lx += 1;
                    if self.lx == 160 {
                        self.mode = Mode::HorizontalBlank;
                    }
                }
            }

            Mode::HorizontalBlank => {
                if self.scanline_timer == 456 {
                    self.scanline_timer = 0;
                    self.ly += 1;
                    if self.ly == SCREEN_HEIGHT {
                        self.mode = Mode::VerticalBlank;
                    } else {
                        self.mode = Mode::OAMSearch;
                    }
                }
            }

            Mode::VerticalBlank => {
                if self.scanline_timer == 456 {
                    self.ly += 1;
                    self.scanline_timer = 0;
                    if self.ly == 154 {
                        // if self.vblank_interrupt_enabled {
                        self.irq |= IF_VBLANK_BIT;
                        // }
                        self.mode = Mode::OAMSearch;
                        self.scanline_timer = 0;
                        self.buffer_pos = 0;
                        self.ly = 0;
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn update(&mut self, cycles: u32) -> bool {
        assert!(cycles % 2 == 0);
        let mut display_update = false;
        for _ in 0..(cycles / 2) {
            display_update = display_update || self.step_2t();
        }
        display_update
    }
}

impl MemoryMapped for PPU {
    fn read(&self, address: usize) -> u8 {
        match address {
            LCDC_REG => {
                let mut lcdc = 0;
                if self.bg_and_window_enable_prio {
                    lcdc = 1;
                }
                if self.objects_enabled {
                    lcdc |= 2;
                }
                if self.objects_are_8x16 {
                    lcdc |= 4;
                }
                if self.bg_tile_map_offset == BG_TILE_MAP_OFFSET_1 {
                    lcdc |= 8;
                }
                if self.bg_and_window_tile_data_offset == BG_AND_WINDOW_TILE_DATA_OFFSET_1 {
                    lcdc |= 16;
                }
                if self.window_enabled {
                    lcdc |= 32;
                }
                if self.window_tile_map_offset == WINDOW_TILE_MAP_OFFSET_1 {
                    lcdc |= 64;
                }
                if self.enabled {
                    lcdc |= 128;
                }
                lcdc
            }
            STAT_REG => {
                let mut stat: u8 = match self.mode {
                    Mode::HorizontalBlank => 0,
                    Mode::VerticalBlank => 1,
                    Mode::OAMSearch => 2,
                    Mode::PixelTransfer => 3,
                };
                if self.lyc_interrupt_enabled {
                    stat |= 64;
                }
                if self.oam_search_interrupt_enabled {
                    stat |= 32;
                }
                if self.vblank_interrupt_enabled {
                    stat |= 16;
                }
                if self.hblank_interrupt_enabled {
                    stat |= 8;
                }
                if self.ly == self.ly_compare {
                    stat |= 4;
                }

                stat
            }
            SCY_REG => self.scy as u8,
            SCX_REG => self.scx as u8,
            LY_REG => (self.ly & 0xFF) as u8,
            LYC_REG => self.ly_compare as u8,
            BGP_REG => {
                let p = self.bg_palette;
                return p[0] | (p[1] << 2) | (p[2] << 4) | (p[3] << 6);
            }
            OBP0_REG => {
                let p = self.obj0_palette;
                return (p[1] << 2) | (p[2] << 4) | (p[3] << 6);
            }
            OBP1_REG => {
                let p = self.obj1_palette;
                return (p[1] << 2) | (p[2] << 4) | (p[3] << 6);
            }
            VRAM_OFFSET..=VRAM_END => self.vram[(address - VRAM_OFFSET) as usize],
            OAM_OFFSET..=OAM_END => {
                match self.mode {
                    Mode::HorizontalBlank | Mode::VerticalBlank => {
                        let idx = (address - OAM_OFFSET) / OAM_SIZE;
                        self.oam[idx].read(address)
                    }

                    // Reading from OAM memory is only possible during hblank and vblank
                    _ => 0xFF,
                }
            }
            _ => panic!("0x{:04x} is not mapped to PPU", address),
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            VRAM_OFFSET..=VRAM_END => self.vram[(address - VRAM_OFFSET) as usize] = value,
            OAM_OFFSET..=OAM_END => {
                match self.mode {
                    Mode::HorizontalBlank | Mode::VerticalBlank => {
                        let idx = (address - OAM_OFFSET) / OAM_SIZE;
                        self.oam[idx].write(address, value);
                    }

                    // Writing to OAM memory is only possible during hblank and vblank
                    _ => {}
                };
            }
            SCX_REG => self.scx = value as usize,
            SCY_REG => self.scy = value as usize,
            BGP_REG => {
                // Palette for background/window color indices
                self.bg_palette[0] = value & 3;
                self.bg_palette[1] = (value >> 2) & 3;
                self.bg_palette[2] = (value >> 4) & 3;
                self.bg_palette[3] = (value >> 6) & 3;
            }
            OBP0_REG => {
                self.obj0_palette[0] = 0;
                self.obj0_palette[1] = (value >> 2) & 3;
                self.obj0_palette[2] = (value >> 4) & 3;
                self.obj0_palette[3] = (value >> 6) & 3;
            }
            OBP1_REG => {
                self.obj1_palette[0] = 0;
                self.obj1_palette[1] = (value >> 2) & 3;
                self.obj1_palette[2] = (value >> 4) & 3;
                self.obj1_palette[3] = (value >> 6) & 3;
            }
            LCDC_REG => {
                self.enabled = (value & 128) != 0;
                self.window_tile_map_offset = if value & 64 == 0 {
                    WINDOW_TILE_MAP_OFFSET_0
                } else {
                    WINDOW_TILE_MAP_OFFSET_1
                };
                self.window_enabled = (value & 32) != 0;
                self.bg_and_window_tile_data_offset = if value & 16 == 0 {
                    BG_AND_WINDOW_TILE_DATA_OFFSET_0
                } else {
                    BG_AND_WINDOW_TILE_DATA_OFFSET_1
                };
                self.bg_tile_map_offset = if value & 8 == 0 {
                    BG_TILE_MAP_OFFSET_0
                } else {
                    BG_TILE_MAP_OFFSET_1
                };
                self.objects_are_8x16 = value & 4 != 0;
                self.objects_enabled = value & 2 != 0;
                self.bg_and_window_enable_prio = value & 1 != 0;
            }
            STAT_REG => {
                self.lyc_interrupt_enabled = value & 64 != 0;
                self.oam_search_interrupt_enabled = value & 32 != 0;
                self.vblank_interrupt_enabled = value & 16 != 0;
                self.hblank_interrupt_enabled = value & 8 != 0;
            }
            LYC_REG => self.ly_compare = value as usize,
            _ => panic!("0x{:04x} is not mapped to PPU for writing", address),
        };
    }

    fn reset(&mut self) {
        // 3 is the brightest color for DMG
        self.buffer.fill(3);
        self.vram.fill(0);
        self.oam = [Sprite::default(); OAM_SIZE / OAM_OBJECT_SIZE];
        self.irq = 0;
    }
}
