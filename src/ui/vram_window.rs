use egui::{Context, Ui};
use egui_wgpu_backend::RenderPass;
use wgpu::{Device, Queue};

use crate::emu::Emu;

use super::{tile_data_view::TileDataView, tile_map_view::TileMapView};

pub struct VRAMWindow {
    selected_tab: String,
    tile_data_view: TileDataView,
    tile_map_view: TileMapView,
}

impl VRAMWindow {
    pub fn new() -> Self {
        VRAMWindow {
            selected_tab: "tile-data".to_string(),
            tile_data_view: TileDataView::new(),
            tile_map_view: TileMapView::new(),
        }
    }

    pub fn init(&mut self, device: &Device, rpass: &mut RenderPass) {
        self.tile_data_view.init(device, rpass);
        self.tile_map_view.init(device, rpass);
    }

    fn render_tabs(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.selected_tab.as_str() == "tile-data", "Tile Data")
                .clicked()
            {
                self.selected_tab = "tile-data".to_string();
            }

            if ui
                .selectable_label(self.selected_tab.as_str() == "tile-map", "Tile Map")
                .clicked()
            {
                self.selected_tab = "tile-map".to_string();
            }
        });
    }

    pub fn render(&mut self, ctx: &Context, emu: &mut Emu, queue: &Queue, open: &mut bool) {
        egui::Window::new("Video RAM").open(open).show(ctx, |ui| {
            self.render_tabs(ui);
            match self.selected_tab.as_str() {
                "tile-data" => self.tile_data_view.render(ui, emu, queue),
                "tile-map" => self.tile_map_view.render(ui, emu, queue),
                _ => {}
            };
        });
    }
}
