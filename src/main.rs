use cgmath::prelude::*;
use std::iter;
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod camera;
mod dynamics;
mod model;
mod post;
mod quad;
mod rand_util;
mod sampler;
mod screenshot;
mod sphere;
mod tail_buffer;
mod texture;
mod util;

use model::Vertex;
use quad::DrawQuad;
use sphere::DrawSphere;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    // UPDATED!
    fn update_view_proj(&mut self, camera: &camera::Camera, projection: &camera::Projection) {
        self.view_position = camera.position.to_homogeneous().into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).into()
    }
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    render_pipeline_no_light: wgpu::RenderPipeline,
    render_pipeline_tails: wgpu::RenderPipeline,
    camera: camera::Camera,
    projection: camera::Projection,
    camera_controller: camera::CameraController,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    sphere_mesh: sphere::SphereMesh,
    sphere_instances: Vec<sphere::SphereInstance>,

    sphere_instance_buffer: wgpu::Buffer,
    tail_buffers: Vec<wgpu::Buffer>,
    #[allow(dead_code)]
    depth_texture: texture::Texture,
    size: winit::dpi::PhysicalSize<u32>,
    post: post::Post,
    #[allow(dead_code)]
    mouse_pressed: bool,
    paused: bool,
    need_screenshot: bool,
    chaos: rand_util::Chaos,
}

impl State {
    async fn new(window: &Window, size: winit::dpi::PhysicalSize<u32>) -> Self {
        let mut chaos = rand_util::Chaos::new();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // UPDATED!
        let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection =
            camera::Projection::new(sc_desc.width, sc_desc.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = camera::CameraController::new(4.0, 0.4);

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera, &projection);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let sphere_mesh = sphere::SphereMesh::new(&device, 64, 64);

        let lims = 4.0;
        let n_spheres = 1000;
        let sphere_instances = (0..n_spheres)
            .map(|_ix| {
                /*
                let dynamics = dynamics::Circler::new(0.01, 0.01, lims, &mut chaos);
                sphere::SphereInstance::randomized(
                    &mut chaos,
                    Box::new(dynamics),
                )
                */
                let s = 0.1;
                let sigma = 18.0;
                let rho = 8.0;
                let beta = 8.0 / 3.0;
                let dynamics = dynamics::Lorenz::new(sigma, rho, beta, s, lims, &mut chaos);
                sphere::SphereInstance::randomized(&mut chaos, Box::new(dynamics))
            })
            .collect::<Vec<_>>();
        let sphere_instance_data = sphere_instances
            .iter()
            .map(sphere::SphereInstance::to_raw)
            .collect::<Vec<_>>();
        let sphere_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere instance buffer"),
            contents: bytemuck::cast_slice(&sphere_instance_data),
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
        });

        let buffer_fill = (0..1024)
            .map(|_ix| sphere::SphereVertex {
                position: [0.0, 0.0, 0.0],
            })
            .collect::<Vec<_>>();
        let tail_buffers = sphere_instances
            .iter()
            .map(|_s| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Tails buffer"),
                    contents: bytemuck::cast_slice(&buffer_fill),
                    usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                })
            })
            .into_iter()
            .collect::<Vec<_>>();

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

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let depth_texture = texture::Texture::create_depth_texture(&device, size, "depth_texture");

        let render_pipeline_layout_no_light =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout (No Light)"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline_no_light = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader (No Light)"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader_no_light.wgsl").into()),
            };
            util::create_render_pipeline(
                &device,
                &render_pipeline_layout_no_light,
                sc_desc.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[
                    sphere::SphereVertex::desc(),
                    sphere::SphereInstanceRaw::desc(),
                ],
                shader,
            )
        };

        let render_pipeline_layout_tails =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout (Tails)"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline_tails = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader (No Light)"),
                flags: wgpu::ShaderFlags::all(),
                source: wgpu::ShaderSource::Wgsl(include_str!("tail_shader.wgsl").into()),
            };
            let primitive = wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLAMPING
                clamp_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            };

            util::create_render_pipeline_with_primitive(
                &device,
                &render_pipeline_layout_tails,
                sc_desc.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[
                    sphere::SphereVertex::desc(),
                    sphere::SphereInstanceRaw::desc(),
                ],
                shader,
                primitive,
            )
        };

        let post = post::Post::new(&device, size, sc_desc.format);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline_no_light,
            render_pipeline_tails,
            camera,
            projection,
            camera_controller,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            depth_texture,
            size,
            post,
            sphere_mesh,
            sphere_instances,
            sphere_instance_buffer,
            tail_buffers,
            #[allow(dead_code)]
            mouse_pressed: false,
            paused: false,
            need_screenshot: false,
            chaos,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.projection.resize(new_size.width, new_size.height);
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        //self.depth_texture =
        //    texture::Texture::create_depth_texture(&self.device, &self.sc_desc, self.size, "depth_texture");
    }

    fn input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(VirtualKeyCode::Space),
                state,
                ..
            }) => {
                if *state == ElementState::Pressed {
                    self.paused = !self.paused;
                }
                true
            }
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(VirtualKeyCode::Return),
                state,
                ..
            }) => {
                if *state == ElementState::Pressed {
                    self.need_screenshot = true
                }
                true
            }
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => self.camera_controller.process_keyboard(*key, *state),
            DeviceEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            DeviceEvent::Button {
                button: 1, // Left Mouse Button
                state,
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_pressed {
                    self.camera_controller.process_mouse(delta.0, delta.1);
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.uniforms
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );

        // Update the light
        if !self.paused {
            for ix in 0..self.sphere_instances.len() {
                if self.sphere_instances[ix].enabled {
                    self.sphere_instances[ix].update(&mut self.chaos);
                } else {
                    // if not enabled, randomly enable
                    let p_enable = 0.001;
                    if self.chaos.bernoulli(p_enable) {
                        self.sphere_instances[ix].enabled = true;
                    }
                }
            }
            let sphere_instance_data = self
                .sphere_instances
                .iter()
                .map(sphere::SphereInstance::to_raw)
                .collect::<Vec<_>>();

            self.queue.write_buffer(
                &self.sphere_instance_buffer,
                0,
                bytemuck::cast_slice(&sphere_instance_data),
            );
            for ix in 0..self.sphere_instances.len() {
                let raw = self.sphere_instances[ix].raw_tail();
                self.queue
                    .write_buffer(&self.tail_buffers[ix], 0, bytemuck::cast_slice(&raw))
            }

            /*
            let old_position: cgmath::Vector3<_> = self.light.position.into();
            self.light.position =
                (cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0))
                    * old_position)
                    .into();
            self.queue
                .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light]));
                */
        }

        if self.need_screenshot {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Screenshot Render Encoder"),
                });

            let mut screenshot =
                screenshot::ScreenShot::init(self.size, self.sc_desc.format, &self.device);

            self.render_to(&screenshot.output_texture.view, &mut encoder)
                .unwrap();

            screenshot.copy_back_buffer(&mut encoder);
            self.queue.submit(iter::once(encoder.finish()));

            screenshot.save(&self.device, screenshot::build_path());

            self.need_screenshot = false;
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let frame = self.swap_chain.get_current_frame()?.output;
        self.render_to(&frame.view, &mut encoder).unwrap();
        self.queue.submit(iter::once(encoder.finish()));
        Ok(())
    }

    fn render_to(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), wgpu::SwapChainError> {
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &self.post.ping_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_vertex_buffer(1, self.sphere_instance_buffer.slice(..));
            render_pass.set_pipeline(&self.render_pipeline_no_light);
            render_pass.draw_sphere_instanced(
                &self.sphere_mesh,
                &self.uniform_bind_group,
                0..self.sphere_instances.len() as u32,
            );

            render_pass.set_vertex_buffer(1, self.sphere_instance_buffer.slice(..));
            for (ix, s) in self.sphere_instances.iter().enumerate() {
                let n = s.tail_len();
                render_pass.set_vertex_buffer(0, self.tail_buffers[ix].slice(..));
                render_pass.set_pipeline(&self.render_pipeline_tails);
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.draw(0..(n as u32), (ix as u32)..((ix as u32) + 1));
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass 2"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &self.post.pong_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.post.render_pipeline);
            render_pass.set_bind_group(0, &self.post.ping_texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.post.ping_uniform_bind_group, &[]);
            render_pass.draw_quad(&self.post.fullscreen_quad);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass 3"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.post.render_pipeline);
            render_pass.set_bind_group(0, &self.post.pong_texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.post.pong_uniform_bind_group, &[]);
            render_pass.draw_quad(&self.post.fullscreen_quad);
        }

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();

    let monitor = event_loop
        .available_monitors()
        .nth(0)
        .expect("No monitor 0");
    let video_mode = monitor.video_modes().nth(0).expect("No video mode 0");

    let size = video_mode.size().clone();
    let fullscreen = Some(winit::window::Fullscreen::Exclusive(video_mode));

    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        //        .with_inner_size(winit::dpi::LogicalSize::new(1280, 1024))
        .with_fullscreen(fullscreen.clone())
        .build(&event_loop)
        .unwrap();

    use futures::executor::block_on;
    let mut state = block_on(State::new(&window, size)); // NEW!
    let mut last_render_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent {
                ref event,
                .. // We're not using device_id currently
            } => {
                state.input(event);
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                state.update(dt);
                match state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    });
}
