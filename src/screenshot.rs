use chrono::Utc;

use crate::texture;

pub struct ScreenShot {
    size: winit::dpi::PhysicalSize<u32>,
    output_buffer: wgpu::Buffer,
    pub output_texture: texture::Texture,
}

pub fn build_path() -> std::path::PathBuf {
    let mut fullpath = std::env::current_dir().unwrap();

    let now = Utc::now();
    let fname = format!("{}.png", now.to_rfc3339());
    let p = std::path::PathBuf::from(fname);
    fullpath.push("screenshots");
    fullpath.push(p);

    fullpath
}

impl ScreenShot {
    pub fn init(
        size: winit::dpi::PhysicalSize<u32>,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
    ) -> Self {
        let u32_size = std::mem::size_of::<u32>() as u32;

        let output_buffer_size = (u32_size * size.width * size.height) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            // this tells wpgu that we want to read this buffer from the cpu
            usage: wgpu::BufferUsage::COPY_DST | wgpu::BufferUsage::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);

        let output_texture = texture::Texture::create_target_texture(&device, size, format);

        Self {
            size,
            output_buffer,
            output_texture,
        }
    }

    pub fn copy_back_buffer(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let u32_size = std::mem::size_of::<u32>() as u32;
        let bytes_per_row =
            unsafe { std::num::NonZeroU32::new_unchecked(u32_size * self.size.width) };
        let rows_per_image = unsafe { std::num::NonZeroU32::new_unchecked(self.size.height) };
        let texture_size = wgpu::Extent3d {
            width: self.size.width,
            height: self.size.height,
            depth_or_array_layers: 1,
        };

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.output_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(rows_per_image),
                },
            },
            texture_size,
        );
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, device: &wgpu::Device, path: P) {
        {
            let buffer_slice = self.output_buffer.slice(..);

            // NOTE: We have to create the mapping THEN device.poll() before await
            // the future. Otherwise the application will freeze.
            use futures::executor::block_on;
            let f = async {
                let mapping = buffer_slice.map_async(wgpu::MapMode::Read);
                device.poll(wgpu::Maintain::Wait);
                mapping.await.unwrap();
            };
            block_on(f);

            let data = buffer_slice.get_mapped_range();

            use image::{ImageBuffer, Rgba};
            let buffer =
                ImageBuffer::<Rgba<u8>, _>::from_raw(self.size.width, self.size.height, data)
                    .unwrap();
            buffer.save(path).unwrap();
        }
        self.output_buffer.unmap();
    }
}
