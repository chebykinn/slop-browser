use super::gpu::GpuContext;
use image::{DynamicImage, GenericImageView};
use std::collections::HashMap;
use wgpu::*;

pub struct TextureManager {
    textures: HashMap<usize, Texture>,
    views: HashMap<usize, TextureView>,
    next_id: usize,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            views: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn load_image(&mut self, gpu: &GpuContext, image: &DynamicImage) -> usize {
        let rgba = image.to_rgba8();
        let (width, height) = image.dimensions();

        let texture = gpu.device.create_texture(&TextureDescriptor {
            label: Some("Image Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        gpu.queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&TextureViewDescriptor::default());

        let id = self.next_id;
        self.next_id += 1;

        self.textures.insert(id, texture);
        self.views.insert(id, view);

        id
    }

    pub fn get_view(&self, id: usize) -> Option<&TextureView> {
        self.views.get(&id)
    }

    pub fn remove(&mut self, id: usize) {
        self.textures.remove(&id);
        self.views.remove(&id);
    }

    pub fn clear(&mut self) {
        self.textures.clear();
        self.views.clear();
    }
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}
