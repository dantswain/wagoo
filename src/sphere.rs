use bitflags::bitflags;
use std::ops::Range;
use wgpu::util::DeviceExt;

use crate::dynamics;
use crate::model;
use crate::rand_util::Chaos;
use crate::sampler;
use crate::tail_buffer;

pub struct SphereInstance {
    pub dynamics: Box<dyn dynamics::DynamicSystem>,
    pub radius: f32,
    pub color: [f32; 4],
    pub heading: f32,
    pub tail: tail_buffer::TailBuffer,
    sampler: sampler::Sampler,
    pub enabled: bool,
}

bitflags! {
    struct SphereAttrs: i32 {
        const NONE = 0b00;
        const ENABLED = 0b01;
    }
}

impl SphereInstance {
    pub fn randomized(chaos: &mut Chaos, dynamics: Box<dyn dynamics::DynamicSystem>) -> Self {
        let tail_capacity = 1024;

        Self {
            dynamics,
            radius: 0.1,
            color: chaos.random_solid_color(),
            heading: chaos.unit_radian_noise(),
            tail: tail_buffer::TailBuffer::new(tail_capacity),
            sampler: sampler::Sampler::new(4),
            enabled: false,
        }
    }

    pub fn update(&mut self, chaos: &mut Chaos) {
        self.dynamics.step(chaos);
        self.push_tail();
    }

    pub fn push_tail(&mut self) {
        if self.sampler.check() {
            self.tail.push(self.dynamics.get_position());
        }
    }

    pub fn raw_tail(&self) -> Vec<SphereVertex> {
        self.tail.to_vec()
    }

    pub fn tail_len(&self) -> usize {
        self.tail.len()
    }

    pub fn to_raw(&self) -> SphereInstanceRaw {
        SphereInstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.dynamics.get_position())
                * cgmath::Matrix4::from_scale(self.radius))
            .into(),
            color: self.color,
            attrs: self.attrs().bits(),
        }
    }

    fn attrs(&self) -> SphereAttrs {
        let mut attrs = SphereAttrs::NONE;
        if self.enabled {
            attrs = attrs | SphereAttrs::ENABLED;
        }
        attrs
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphereInstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
    attrs: i32,
}

impl model::Vertex for SphereInstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SphereInstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // color
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // attrs
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Sint32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphereVertex {
    pub position: [f32; 3],
}

impl model::Vertex for SphereVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SphereVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                // vertices
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct SphereMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

impl SphereMesh {
    pub fn new(device: &wgpu::Device, nx: u32, nz: u32) -> SphereMesh {
        assert!(nx >= 4, "nx must be >= 4");
        assert!(nz >= 4, "nz must be >= 4");

        let name = format!("Sphere {} x {}", nx, nz);

        let mut vertices: Vec<SphereVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let north_pole = SphereVertex {
            position: [0.0, 0.0, 1.0],
        };
        let south_pole = SphereVertex {
            position: [0.0, 0.0, -1.0],
        };

        let dtheta = 2.0 * std::f32::consts::PI / nx as f32;
        let theta0: f32 = 0.0;
        let dphi = std::f32::consts::PI / (nz - 1) as f32;
        let phi0 = -0.5 * std::f32::consts::PI;

        vertices.push(south_pole);

        /* CCW triangles: [k0, k2, k1], [k0, k3, k2]
         * k1 -- k2
         * |   /  |
         * | /    |
         * k0 -- k3
         */

        /* south pole is vertex 0, north pole is the last vertex
         * each vertical slice adds (nz - 2) vertices
         * total number of vertexes = 2 + (nz - 2) * nx
         */

        let mut kx = 1;

        for ix in 0..nx {
            // bottom row, k0 == 0
            let k0 = 0;
            let k1 = (ix * (nz - 2)) + 1;

            // if this is the last row, wrap back to ix = 0
            let k2 = if ix < nx - 1 { k1 + (nz - 1) - 1 } else { 1 };

            indices.push(k0);
            indices.push(k2);
            indices.push(k1);

            // exclude the poles
            for iz in 1..(nz - 1) {
                let theta = theta0 + (ix as f32) * dtheta;
                let phi = phi0 + (iz as f32) * dphi;
                let x = theta.cos() * phi.cos();
                let y = theta.sin() * phi.cos();
                let z = phi.sin();

                kx = kx + 1;

                vertices.push(SphereVertex {
                    position: [x, y, z],
                });

                if iz < nz - 2 {
                    let k0: u32 = ix * (nz - 2) + iz;
                    let k1: u32 = k0 + 1;
                    let k2: u32 = if ix < (nx - 1) { k1 + (nz - 2) } else { iz + 1 };
                    let k3: u32 = k2 - 1;

                    indices.push(k0);
                    indices.push(k2);
                    indices.push(k1);
                    indices.push(k0);
                    indices.push(k3);
                    indices.push(k2);
                }
            }

            // top row, k1 == north_pole index = (2 + nx * (nz - 2)) - 1
            let k0 = ix * (nz - 2) + (nz - 2);
            let k1 = 1 + nx * (nz - 2);
            let k2 = if ix < (nx - 1) { k0 + (nz - 2) } else { nz - 2 };
            indices.push(k2);
            indices.push(k1);
            indices.push(k0);
        }

        vertices.push(north_pole);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", name)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsage::INDEX,
        });

        let num_elements = indices.len() as u32;

        SphereMesh {
            vertex_buffer,
            index_buffer,
            num_elements,
        }
    }
}

pub trait DrawSphere<'a, 'b>
where
    'b: 'a,
{
    fn draw_sphere_instanced(
        &mut self,
        mesh: &'b SphereMesh,
        uniforms: &'b wgpu::BindGroup,
        instances: Range<u32>,
    );
}

impl<'a, 'b> DrawSphere<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_sphere_instanced(
        &mut self,
        mesh: &'b SphereMesh,
        uniforms: &'b wgpu::BindGroup,
        instances: Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.set_bind_group(0, &uniforms, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}
