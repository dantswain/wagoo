use bitflags::bitflags;
use wgpu::util::DeviceExt;

use crate::model::Vertex;
use crate::quad;
use crate::texture;
use crate::util;

bitflags! {
  struct Flags: i32 {
    const NONE = 0b0000000;
    const ENABLED = 0b00000001;
    const HORIZONTAL = 0b00000010;
  }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    flags: i32,
}

pub struct Post {
    pub fullscreen_quad: quad::Quad,
    pub ping_texture: texture::Texture,
    pub pong_texture: texture::Texture,
    pub ping_texture_bind_group: wgpu::BindGroup,
    pub pong_texture_bind_group: wgpu::BindGroup,
    pub ping_buffer: wgpu::Buffer,
    pub pong_buffer: wgpu::Buffer,
    pub ping_uniform_bind_group: wgpu::BindGroup,
    pub pong_uniform_bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Post {
    pub fn new(
        device: &wgpu::Device,
        size: winit::dpi::PhysicalSize<u32>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let fullscreen_quad = quad::Quad::make_fullscreen_quad(&device).unwrap();

        let ping_texture = texture::Texture::create_target_texture(&device, size, format);
        let pong_texture = texture::Texture::create_target_texture(&device, size, format);

        let texture_bind_group_layout = device.create_bind_group_layout(
            &texture::Texture::bind_group_layout_descriptor(Some("texture bind group layout")),
        );

        let ping_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&ping_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&ping_texture.sampler),
                },
            ],
            label: Some("ping_texture_bind_group"),
        });
        let pong_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&pong_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&pong_texture.sampler),
                },
            ],
            label: Some("pong_texture_bind_group"),
        });

        let base_flags = Flags::NONE;

        let ping_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ping Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                flags: base_flags.bits(),
            }]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let pong_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pong Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                flags: (base_flags | Flags::HORIZONTAL).bits(),
            }]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let ping_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ping_buffer.as_entire_binding(),
            }],
            label: Some("ping_uniform_bind_group"),
        });

        let pong_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: pong_buffer.as_entire_binding(),
            }],
            label: Some("pong_uniform_bind_group"),
        });

        let render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Post Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Post Shader"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("post.wgsl").into()),
            };
            util::create_render_pipeline(
                &device,
                &layout,
                format,
                None,
                &[quad::QuadVertex::desc()],
                shader,
            )
        };

        Self {
            fullscreen_quad,
            ping_texture,
            pong_texture,
            ping_texture_bind_group,
            pong_texture_bind_group,
            ping_buffer,
            pong_buffer,
            ping_uniform_bind_group,
            pong_uniform_bind_group,
            render_pipeline,
        }
    }
}
