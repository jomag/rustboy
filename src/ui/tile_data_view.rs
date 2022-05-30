use egui::{CtxRef, Ui};
use egui_wgpu_backend::RenderPass;
use wgpu::{Device, FilterMode, Queue};

use crate::{emu::Emu, ppu::PPU};

const TILES_PER_ROW: usize = 16;
const TILE_COUNT: usize = 384;
const TILE_WIDTH: usize = 8;
const TILE_HEIGHT: usize = 8;
const TILE_ROWS: usize = TILE_COUNT / TILES_PER_ROW;
const TILE_DATA_WIDTH: usize = TILES_PER_ROW * TILE_WIDTH;
const TILE_DATA_HEIGHT: usize = (TILE_COUNT / TILES_PER_ROW) * TILE_HEIGHT;
const PIXEL_SIZE: usize = 4;

pub struct TileDataView {
    texture_buf: Box<[u8]>,
    texture_id: Option<egui::TextureId>,
}

impl TileDataView {
    pub fn new() -> Self {
        TileDataView {
            texture_id: None,
            texture_buf: vec![0; TILE_COUNT * TILE_WIDTH * TILE_HEIGHT * PIXEL_SIZE]
                .into_boxed_slice(),
        }
    }

    fn render_texture(&mut self, ppu: &PPU) {
        for row in 0..TILE_ROWS {
            for col in 0..16 {
                //TILES_PER_ROW {
                // self.render_tile(
                //     ppu,
                //     row * TILE_HEIGHT,
                //     col * TILE_WIDTH,
                // )
                for y in 0..TILE_HEIGHT {
                    let offs =
                        (row * TILES_PER_ROW * TILE_HEIGHT * 2) + (y * 2) + col * TILE_HEIGHT * 2;
                    let lo = ppu.vram[offs];
                    let hi = ppu.vram[offs + 1];
                    for x in 0..TILE_WIDTH {
                        let v = ((lo >> (7 - x)) & 1) | (((hi >> (7 - x)) & 1) << 1);
                        let dst = (row * TILE_HEIGHT * TILES_PER_ROW * TILE_WIDTH
                            + y * TILES_PER_ROW * TILE_WIDTH
                            + col * TILE_WIDTH
                            + x)
                            * 4;
                        self.texture_buf[dst + 0] = v * 40;
                        self.texture_buf[dst + 1] = v * 40;
                        self.texture_buf[dst + 2] = v * 40;
                        self.texture_buf[dst + 3] = 255;
                    }
                }
            }
        }
    }

    fn update_texture(
        &mut self,
        ppu: &PPU,
        device: &Device,
        queue: &Queue,
        egui_rpass: &mut RenderPass,
    ) {
        // This is dumb. A new texture is created for every frame.
        // And to make it worse, the exact same thing happens for the
        // display texture in full.rs with full code duplication. FIXME!
        if self.texture_id.is_none() || true {
            let size = wgpu::Extent3d {
                width: TILE_DATA_WIDTH as u32,
                height: TILE_DATA_HEIGHT as u32,
                depth_or_array_layers: 1,
            };

            let txt = device.create_texture(&wgpu::TextureDescriptor {
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("tile data texture"),
            });

            self.render_texture(ppu);

            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &txt,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.texture_buf,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * size.width as u32),
                    rows_per_image: std::num::NonZeroU32::new(size.height as u32),
                },
                size,
            );

            let texture_id =
                egui_rpass.egui_texture_from_wgpu_texture(&device, &txt, FilterMode::Nearest);

            self.texture_id = Some(texture_id);
        }
    }

    pub fn render(
        &mut self,
        ui: &mut Ui,
        emu: &mut Emu,
        device: &Device,
        queue: &Queue,
        egui_rpass: &mut RenderPass,
    ) {
        self.update_texture(&emu.mmu.ppu, device, queue, egui_rpass);
        if let Some(texture_id) = self.texture_id {
            let scale: usize = 2;
            let size = egui::Vec2::new(
                (TILE_DATA_WIDTH * scale) as f32,
                (TILE_DATA_HEIGHT * scale) as f32,
            );
            ui.image(texture_id, size);
        };
    }
}
