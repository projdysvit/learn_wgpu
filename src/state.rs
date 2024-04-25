use std::iter::once;
use wgpu::{Adapter, Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, LoadOp, Operations, PowerPreference, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor};
use winit::{dpi::PhysicalSize, window::Window};

use self::renderer_backend::pipeline_builder::PipelineBuilder;

#[path ="renderer_backend/mod.rs"]
mod renderer_backend;

pub struct State<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    pub window: &'a Window,
    render_pipeline: RenderPipeline
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> Self
    {
        let size = window.inner_size();
        let instance = Instance::new(Self::get_instance_descriptor());
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance.request_adapter(&Self::get_adapter_descriptor(&surface))
            .await
            .unwrap();
        let (device, queue) = adapter.request_device(&Self::get_device_descriptor(), None)
            .await
            .unwrap();
        let config = Self::get_surface_configuration(&surface, &adapter, &size);

        surface.configure(&device, &config);

        let render_pipeline = PipelineBuilder::builder()
            .set_shader_module("colorful_triangle.wgsl", "vs_main", "fs_main")
            .set_pixel_format(config.format)
            .build(&device);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline
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
            render_pass.draw(0..3, 0..1);
        }
        
        self.queue.submit(once(command_encoder.finish()));

        drawable.present();

        Ok(())
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
            required_limits: Limits::default(),
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
