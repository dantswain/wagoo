use anyhow::*;
use wgpu::util::DeviceExt;

use crate::model;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

const QUAD_VERTICES: [QuadVertex; 4] = [
    QuadVertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    }, // top-left
    QuadVertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    }, // bottom-right
    QuadVertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    }, // bottom-left
    QuadVertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    }, // top-right
];

const QUAD_INDECES: [u32; 6] = [0, 2, 1, 0, 1, 3];

impl model::Vertex for QuadVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                // vertices
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // texture coordinates
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ],
        }
    }
}

pub struct Quad {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
}

impl Quad {
    pub fn make_fullscreen_quad(device: &wgpu::Device) -> Result<Self> {
        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("the quad Vertex Buffer")),
            contents: bytemuck::cast_slice(&QUAD_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("the quad Index Buffer")),
            contents: bytemuck::cast_slice(&QUAD_INDECES),
            usage: wgpu::BufferUsage::INDEX,
        });

        Ok(Self {
            name: "fullscreen quad".to_string(),
            vertex_buffer: quad_vertex_buffer,
            index_buffer: quad_index_buffer,
        })
    }
}

pub trait DrawQuad<'a, 'b>
where
    'b: 'a,
{
    fn draw_quad(&mut self, quad: &'b Quad);
}

impl<'a, 'b> DrawQuad<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_quad(&mut self, quad: &'b Quad) {
        self.set_vertex_buffer(0, quad.vertex_buffer.slice(..));
        self.set_index_buffer(quad.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..6, 0, 0..1);
    }
}
