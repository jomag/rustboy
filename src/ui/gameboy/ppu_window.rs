use egui::{Context, Ui};

use crate::{
    gameboy::{
        emu::Emu,
        mmu::{LCDC_REG, LY_REG, SCX_REG, SCY_REG, STAT_REG, WX_REG, WY_REG},
        ppu::{
            BG_AND_WINDOW_TILE_DATA_OFFSET_0, BG_AND_WINDOW_TILE_DATA_OFFSET_1,
            BG_TILE_MAP_OFFSET_0, BG_TILE_MAP_OFFSET_1, WINDOW_TILE_MAP_OFFSET_0,
            WINDOW_TILE_MAP_OFFSET_1,
        },
    },
    MemoryMapped,
};

fn read_only_checkbox(ui: &mut Ui, mut checked: bool) {
    ui.checkbox(&mut checked, "");
}

fn bool_option(ui: &mut Ui, mut value: bool, opt1: &str, opt2: &str) {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut value, false, opt1);
        ui.selectable_value(&mut value, true, opt2);
    });
}

fn selectable_area(ui: &mut Ui, value: bool, area0: usize, area1: usize, size: usize) {
    let opt1 = format!("{:04X}-{:04X}", area0, area0 + size);
    let opt2 = format!("{:04X}-{:04X}", area1, area1 + size);
    bool_option(ui, value, &opt1, &opt2);
}

fn render_property_grid(ui: &mut Ui, emu: &mut Emu) {
    let ppu = &emu.mmu.ppu;

    let ly = ppu.read(LY_REG);
    ui.label("LY");
    ui.label(format!("0x{:02X} ({})", ly, ly));
    ui.end_row();

    let scx = ppu.read(SCX_REG);
    ui.label("SCX");
    ui.label(format!("0x{:02X} ({})", scx, scx));
    ui.end_row();

    let scy = ppu.read(SCY_REG);
    ui.label("SCY");
    ui.label(format!("0x{:02X} ({})", scy, scy));
    ui.end_row();

    let lcdc = ppu.read(LCDC_REG);
    ui.label("LCDC");
    ui.label(format!("0x{:02X} ({})", lcdc, lcdc));
    ui.end_row();

    ui.label("\tEnabled");
    read_only_checkbox(ui, lcdc & 128 != 0);
    ui.end_row();

    ui.label("\tWindow enabled");
    read_only_checkbox(ui, lcdc & 32 != 0);
    ui.end_row();

    ui.label("\tWindow tile map");
    selectable_area(
        ui,
        lcdc & 16 != 0,
        WINDOW_TILE_MAP_OFFSET_0,
        WINDOW_TILE_MAP_OFFSET_1,
        0x3FF,
    );
    ui.end_row();

    ui.label("\tBG tile map");
    selectable_area(
        ui,
        lcdc & 8 != 0,
        BG_TILE_MAP_OFFSET_0,
        BG_TILE_MAP_OFFSET_1,
        0x3FF,
    );
    ui.end_row();

    ui.label("\tBG+Window tile data");
    selectable_area(
        ui,
        lcdc & 16 != 0,
        BG_AND_WINDOW_TILE_DATA_OFFSET_0,
        BG_AND_WINDOW_TILE_DATA_OFFSET_1,
        0xFFF,
    );
    ui.end_row();

    ui.label("\tOBJ enabled");
    read_only_checkbox(ui, lcdc & 2 != 0);
    ui.end_row();

    ui.label("\tOBJ size");
    bool_option(ui, lcdc & 4 != 0, "8x8", "8x16");
    ui.end_row();

    ui.label("\tBG and Win enable/prio (*)");
    read_only_checkbox(ui, lcdc & 1 != 0);
    ui.end_row();

    let stat = ppu.read(STAT_REG);
    ui.label("STAT");
    ui.label(format!("0x{:02X} ({})", stat, stat));
    ui.end_row();

    let mode = stat & 3;
    ui.label("\tMode");
    ui.label(match mode & 3 {
        0 => "0: HBlank",
        1 => "1: VBlank",
        2 => "2: OAM search",
        3 => "3: Transfer pixels",
        _ => panic!("Invalid mode"),
    });
    ui.end_row();

    ui.label("\tLYC = LY");
    read_only_checkbox(ui, stat & 4 != 0);
    ui.end_row();

    ui.label("\tHBlank int.")
        .on_hover_text("Enable interrupt on H-blank");
    read_only_checkbox(ui, stat & 8 != 0);
    ui.end_row();

    ui.label("\tVBlank int.")
        .on_hover_text("Enable interrupt on V-blank");
    read_only_checkbox(ui, stat & 16 != 0);
    ui.end_row();

    ui.label("\tOAM int.")
        .on_hover_text("Enable interrupt on OAM search mode");
    read_only_checkbox(ui, stat & 32 != 0);
    ui.end_row();

    ui.label("\tLYC=LY int.")
        .on_hover_text("Enable interrupt on LYC=LY");
    read_only_checkbox(ui, stat & 64 != 0);
    ui.end_row();

    let wx = ppu.read(WX_REG);
    let wy = ppu.read(WY_REG);
    ui.label("WX, WY");
    ui.label(format!("{}, {}", wx, wy));
    ui.end_row();
}

pub fn render_video_window(ctx: &Context, emu: &mut Emu, open: &mut bool) {
    egui::Window::new("Video / PPU").open(open).show(ctx, |ui| {
        egui::Grid::new("ppu_properties_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |grid_ui| render_property_grid(grid_ui, emu));
    });
}
