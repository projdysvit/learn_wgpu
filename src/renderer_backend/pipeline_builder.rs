use std::{env::current_dir, fs};

use wgpu::{BindGroupLayout, BlendState, ColorTargetState, ColorWrites, Device, Face, FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexState};

use crate::state::renderer_backend::vertex::Vertex;

pub struct PipelineBuilder {
    shader_filename: String,
    vertex_entry: String,
    fragment_entry: String,
    pixel_format: TextureFormat
}

impl PipelineBuilder {
    pub fn builder() -> Self
    {
        Self {
            shader_filename: String::from("shader.wgsl"),
            vertex_entry: String::from("vs_main"),
            fragment_entry: String::from("fs_main"),
            pixel_format: TextureFormat::Rgba8Unorm
        }
    }

    pub fn set_shader_module(
        &mut self,
        shader_filename: &str,
        vertex_entry: &str,
        fragment_entry: &str
    ) -> &mut Self
    {
        self.shader_filename = String::from(shader_filename);
        self.vertex_entry = String::from(vertex_entry);
        self.fragment_entry = String::from(fragment_entry);

        self
    }

    pub fn set_pixel_format(&mut self, pixel_format: TextureFormat) -> &mut Self
    {
        self.pixel_format = pixel_format;

        self
    }

    pub fn build(
        &mut self,
        device: &Device,
        bind_group_layouts: &[&BindGroupLayout]
    ) -> RenderPipeline
    {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let source_code = self.shader_filename.as_str();
            } else {
                let filepath = current_dir()
                    .unwrap()
                    .join("src")
                    .join("shaders")
                    .join(self.shader_filename.as_str())
                    .into_os_string()
                    .into_string()
                    .unwrap();

                let source_code = fs::read_to_string(filepath)
                    .expect("Can't read the shader source file.");
            }
        }

        let shader_module = device.create_shader_module(
            ShaderModuleDescriptor {
                label: Some("Shader"),
                source: ShaderSource::Wgsl(source_code.into())
            }
        );
        let render_pipeline_layout = device.create_pipeline_layout(
            &PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: bind_group_layouts,
                push_constant_ranges: &[]
            }
        );

        device.create_render_pipeline(
            &RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: &self.vertex_entry,
                    buffers: &[
                        Vertex::get_vertex_buffer_layout()
                    ]
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false
                },
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: &self.fragment_entry,
                    targets: &self.get_render_targets()
                }),
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false
                },
                multiview: None
            }
        )
    }

    fn get_render_targets(&self) -> [Option<ColorTargetState>; 1]
    {
        [
            Some(ColorTargetState {
                format: self.pixel_format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL
            })
        ]
    }
}
