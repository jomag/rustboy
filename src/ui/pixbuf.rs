use egui_wgpu_backend::RenderPass;
use wgpu::{Device, FilterMode, Queue};

// Size of one pixel in bytes. Currently only RGBA is supported (4 bytes)
const PIXEL_SIZE: usize = 4;

pub struct PixBuf {
    pub buf: Box<[u8]>,
    pub width: usize,
    pub height: usize,
    pub dirty: bool,
    pub texture_id: Option<egui::TextureId>,
    texture: Option<wgpu::Texture>,
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

    pub fn get_offset(&self, x: usize, y: usize) -> usize {
        (y * self.width + x) * PIXEL_SIZE
    }

    pub fn get_stride(&self) -> usize {
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

        let view = texture.create_view(&Default::default());

        let texture_id = rpass.egui_texture_from_wgpu_texture(&device, &view, FilterMode::Nearest);

        self.texture = Some(texture);
        self.texture_id = Some(texture_id);
    }

    pub fn is_initialized(&self) -> bool {
        match self.texture {
            Some(_) => true,
            None => false,
        }
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
                        bytes_per_row: std::num::NonZeroU32::new(self.get_stride() as u32),
                        rows_per_image: std::num::NonZeroU32::new(self.height as u32),
                    },
                    self.get_extent3d(),
                );
            }

            self.dirty = false;
        }
    }
}
