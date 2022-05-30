use egui_wgpu_backend::RenderPass;
use wgpu::{Device, FilterMode, Queue};

// Size of one pixel in bytes. Currently only RGBA is supported (4 bytes)
const PIXEL_SIZE: usize = 4;

pub struct PixBuf {
    dirty: bool,
    width: usize,
    height: usize,
    buf: Box<[u8]>,
    texture: Option<wgpu::Texture>,
    texture_id: Option<egui::TextureId>,
}

impl PixBuf {
    pub fn new(width: usize, height: usize) -> Self {
        PixBuf {
            width,
            height,
            dirty: true,
            buf: vec![0; width * height * PIXEL_SIZE].into_boxed_slice(),
            texture: None,
            texture_id: None,
        }
    }

    fn get_extent3d(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width as u32,
            height: self.height as u32,
            depth_or_array_layers: 1,
        }
    }

    fn get_bytes_per_row(&self) -> usize {
        return self.width * PIXEL_SIZE;
    }

    pub fn init(&mut self, device: &Device, rpass: &mut RenderPass) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: self.get_extent3d(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("PixBuf texture"),
        });

        let texture_id =
            rpass.egui_texture_from_wgpu_texture(&device, &texture, FilterMode::Nearest);

        self.texture = Some(texture);
        self.texture_id = Some(texture_id);
    }

    pub fn prepare(&mut self, queue: &wgpu::Queue) {
        if self.dirty {
            if let Some(ref txt) = self.texture {
                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &txt,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &self.buf,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: std::num::NonZeroU32::new(self.get_bytes_per_row() as u32),
                        rows_per_image: std::num::NonZeroU32::new(self.height as u32),
                    },
                    self.get_extent3d(),
                );
            }

            self.dirty = false;
        }
    }
}
