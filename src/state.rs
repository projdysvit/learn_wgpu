use std::iter::once;
use bytemuck::cast_slice;

use cgmath::{prelude::*, Deg, Quaternion, Vector3};
use wgpu::{util::{BufferInitDescriptor, DeviceExt}, Adapter, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, IndexFormat, Instance as WgpuInstance, InstanceDescriptor, Limits, LoadOp, Operations, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RequestAdapterOptions, ShaderStages, StoreOp, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::state::{camera::CameraUniform, renderer_backend::texture::Texture};

use self::{camera::{Camera, CameraController}, renderer_backend::{pipeline_builder::PipelineBuilder, vertex::Vertex}, instance::Instance};

#[path ="renderer_backend/mod.rs"]
mod renderer_backend;
#[path ="camera.rs"]
mod camera;
#[path ="instance.rs"]
mod instance;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4, 0.09]
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.11, 0.4]
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.3, 0.7]
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85, 0.85]
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.85, 0.45]
    } // E
];

const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4
];

const NUM_INSTANCES_PER_ROW: u32 = 10;
const INSTANCE_DISPLACEMENT: Vector3<f32> = Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

pub struct State<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub window: &'a Window,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    diffuse_texture: Texture,
    diffuse_bind_group: BindGroup,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    instances: Vec<Instance>,
    instance_buffer: Buffer
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> Self
    {
        let size = window.inner_size();
        let instance = WgpuInstance::new(Self::get_instance_descriptor());
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(&Self::get_adapter_descriptor(&surface))
            .await
            .unwrap();
        let (device, queue) = adapter.request_device(&Self::get_device_descriptor(), None)
            .await
            .unwrap();
        let config = Self::get_surface_configuration(&surface, &adapter, &size);

        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("../res/crycat.jpg");
        let diffuse_texture = Texture::from_bytes(&device, &queue, diffuse_bytes, "Cry Cat")
            .unwrap();
        let texture_bind_group_layout = Texture::get_texture_bind_group_layout(&device);
        let diffuse_bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("Diffuse Bind Group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&diffuse_texture.view)
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&diffuse_texture.sampler)
                    }
                ]
            }
        );

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let shader_name = include_str!("./shaders/vertex.wgsl");
            } else {
                let shader_name = "vertex.wgsl";
            }
        }

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0
        };

        let camera_controller = CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ]
            }
        );

        let camera_bind_group = device.create_bind_group(
            &BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &camera_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding()
                    }
                ]
            }
        );

        let render_pipeline = PipelineBuilder::builder()
            .set_shader_module(shader_name, "vs_main", "fs_main")
            .set_pixel_format(config.format)
            .build(&device, &[&texture_bind_group_layout, &camera_bind_group_layout]);

        let (vertex_buffer, index_buffer, num_indices) = Self::create_buffers(&device);

        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = Vector3 { x: x as f32, y: 0.0, z: z as f32 } - INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0))
                } else {
                    Quaternion::from_axis_angle(position.normalize(), Deg(45.0))
                };

                Instance {
                    position,
                    rotation
                }
            })
        }).collect::<Vec<_>>();
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX
            }
        );


        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_texture,
            diffuse_bind_group,
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            instances,
            instance_buffer
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>)
    {
        if new_size.width < 1 && new_size.height < 1 { return };

        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError>
    {
        let drawable = self.surface.get_current_texture()?;
        let image_view = drawable.texture.create_view(&Self::get_image_descriptor());
        let mut command_encoder = self.device
            .create_command_encoder(&Self::get_command_encoder_descriptor());

        let color_attachment = RenderPassColorAttachment {
            view: &image_view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(
                    Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0
                    }
                ),
                store: StoreOp::Store
            }
        };

        {
            let mut render_pass = command_encoder.begin_render_pass(
                &RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(color_attachment)],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None
                }
            );
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
        }
        
        self.queue.submit(once(command_encoder.finish()));

        drawable.present();

        Ok(())
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool
    {
        self.camera_controller.process_events(event)
    }

    pub fn update(&mut self)
    {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, cast_slice(&[self.camera_uniform]));
    }

    // new function
    fn get_instance_descriptor() -> InstanceDescriptor
    {
        InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        }
    }

    fn get_adapter_descriptor<'b>(surface: &'b Surface<'a>) -> RequestAdapterOptions<'b, 'a>
    {
        RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(surface),
            force_fallback_adapter: false
        }
    }

    fn get_device_descriptor() -> DeviceDescriptor<'a>
    {
        DeviceDescriptor {
            required_features: Features::empty(),
            required_limits: if cfg!(target_arch = "wasm32") {
                Limits::downlevel_webgl2_defaults()
            } else {
                Limits::default()
            },
            label: Some("Device")
        }
    }

    fn get_surface_configuration(
        surface: &Surface,
        adapter: &Adapter,
        size: &PhysicalSize<u32>
    ) -> SurfaceConfiguration
    {
        let surface_capabilities = surface.get_capabilities(adapter);
        let surface_format = surface_capabilities.formats.iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);

        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        }
    }

    fn create_buffers(device: &Device) -> (Buffer, Buffer, u32)
    {
        let vertex_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: BufferUsages::VERTEX
            }
        );
        let index_buffer = device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: BufferUsages::INDEX
            }
        );
        let num_indices = INDICES.len() as u32;

        (vertex_buffer, index_buffer, num_indices)
    }

    // render function
    fn get_image_descriptor() -> TextureViewDescriptor<'a>
    {
        TextureViewDescriptor::default()
    }

    fn get_command_encoder_descriptor() -> CommandEncoderDescriptor<'a>
    {
        CommandEncoderDescriptor {
            label: Some("Render Encoder")
        }
    }
}
