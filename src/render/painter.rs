use super::gpu::GpuContext;
use bytemuck::{Pod, Zeroable};
use wgpu::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Self::rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }
}

#[derive(Debug, Clone)]
pub enum DisplayCommand {
    SolidRect {
        rect: Rect,
        color: Color,
        border_radius: f32,
        opacity: f32,
    },
    Border {
        rect: Rect,
        color: Color,
        width: f32,
        border_radius: f32,
    },
    Text {
        text: String,
        x: f32,
        y: f32,
        color: Color,
        font_size: f32,
        opacity: f32,
    },
    Image {
        rect: Rect,
        texture_id: usize,
        opacity: f32,
    },
    BoxShadow {
        rect: Rect,
        color: Color,
        offset_x: f32,
        offset_y: f32,
        blur_radius: f32,
        spread_radius: f32,
        border_radius: f32,
    },
}

#[derive(Debug, Default)]
pub struct DisplayList {
    pub commands: Vec<DisplayCommand>,
}

impl DisplayList {
    pub fn new() -> Self {
        Self { commands: Vec::new() }
    }

    pub fn push_rect(&mut self, rect: Rect, color: Color) {
        self.commands.push(DisplayCommand::SolidRect {
            rect,
            color,
            border_radius: 0.0,
            opacity: 1.0,
        });
    }

    pub fn push_rect_with_radius(&mut self, rect: Rect, color: Color, border_radius: f32, opacity: f32) {
        self.commands.push(DisplayCommand::SolidRect {
            rect,
            color,
            border_radius,
            opacity,
        });
    }

    pub fn push_border(&mut self, rect: Rect, color: Color, width: f32) {
        self.commands.push(DisplayCommand::Border {
            rect,
            color,
            width,
            border_radius: 0.0,
        });
    }

    pub fn push_border_with_radius(&mut self, rect: Rect, color: Color, width: f32, border_radius: f32) {
        self.commands.push(DisplayCommand::Border {
            rect,
            color,
            width,
            border_radius,
        });
    }

    pub fn push_text(&mut self, text: String, x: f32, y: f32, color: Color, font_size: f32) {
        self.commands.push(DisplayCommand::Text {
            text,
            x,
            y,
            color,
            font_size,
            opacity: 1.0,
        });
    }

    pub fn push_text_with_opacity(&mut self, text: String, x: f32, y: f32, color: Color, font_size: f32, opacity: f32) {
        self.commands.push(DisplayCommand::Text {
            text,
            x,
            y,
            color,
            font_size,
            opacity,
        });
    }

    pub fn push_image(&mut self, rect: Rect, texture_id: usize) {
        self.commands.push(DisplayCommand::Image {
            rect,
            texture_id,
            opacity: 1.0,
        });
    }

    pub fn push_image_with_opacity(&mut self, rect: Rect, texture_id: usize, opacity: f32) {
        self.commands.push(DisplayCommand::Image {
            rect,
            texture_id,
            opacity,
        });
    }

    pub fn push_box_shadow(
        &mut self,
        rect: Rect,
        color: Color,
        offset_x: f32,
        offset_y: f32,
        blur_radius: f32,
        spread_radius: f32,
        border_radius: f32,
    ) {
        self.commands.push(DisplayCommand::BoxShadow {
            rect,
            color,
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            border_radius,
        });
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RectVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ImageVertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ImageUniforms {
    opacity: f32,
    _padding: [f32; 3],
}

pub struct Painter {
    rect_pipeline: RenderPipeline,
    rect_vertex_buffer: Buffer,
    rect_index_buffer: Buffer,
    max_rects: usize,
    // Image rendering
    image_pipeline: RenderPipeline,
    image_bind_group_layout: BindGroupLayout,
    image_sampler: Sampler,
    image_vertex_buffer: Buffer,
    image_index_buffer: Buffer,
    image_uniform_buffer: Buffer,
}

impl Painter {
    pub fn new(gpu: &GpuContext) -> Self {
        let shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Rect Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/rect.wgsl").into()),
        });

        let pipeline_layout = gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Rect Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let rect_pipeline = gpu.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Rect Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<RectVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x2,
                        },
                        VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: gpu.format(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let max_rects = 10000;
        let rect_vertex_buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some("Rect Vertex Buffer"),
            size: (max_rects * 4 * std::mem::size_of::<RectVertex>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let rect_index_buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some("Rect Index Buffer"),
            size: (max_rects * 6 * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create image rendering pipeline
        let image_shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Image Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/image.wgsl").into()),
        });

        let image_bind_group_layout = gpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Image Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let image_pipeline_layout = gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Image Pipeline Layout"),
            bind_group_layouts: &[&image_bind_group_layout],
            push_constant_ranges: &[],
        });

        let image_pipeline = gpu.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Image Pipeline"),
            layout: Some(&image_pipeline_layout),
            vertex: VertexState {
                module: &image_shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<ImageVertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x2,
                        },
                        VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: VertexFormat::Float32x2,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &image_shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: gpu.format(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let image_sampler = gpu.device.create_sampler(&SamplerDescriptor {
            label: Some("Image Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let image_vertex_buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some("Image Vertex Buffer"),
            size: (4 * std::mem::size_of::<ImageVertex>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let image_index_buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some("Image Index Buffer"),
            size: (6 * std::mem::size_of::<u32>()) as u64,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let image_uniform_buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some("Image Uniform Buffer"),
            size: std::mem::size_of::<ImageUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            rect_pipeline,
            rect_vertex_buffer,
            rect_index_buffer,
            max_rects,
            image_pipeline,
            image_bind_group_layout,
            image_sampler,
            image_vertex_buffer,
            image_index_buffer,
            image_uniform_buffer,
        }
    }

    pub fn draw_rects(
        &self,
        gpu: &GpuContext,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        rects: &[(Rect, Color)],
        viewport_width: f32,
        viewport_height: f32,
        scale_factor: f32,
    ) {
        if rects.is_empty() {
            return;
        }

        // Physical viewport dimensions
        let physical_w = viewport_width * scale_factor;
        let physical_h = viewport_height * scale_factor;

        let mut vertices = Vec::with_capacity(rects.len() * 4);
        let mut indices = Vec::with_capacity(rects.len() * 6);

        for (i, (rect, color)) in rects.iter().enumerate() {
            // Scale logical rect coordinates to physical
            let px = rect.x * scale_factor;
            let py = rect.y * scale_factor;
            let pw = rect.width * scale_factor;
            let ph = rect.height * scale_factor;

            let x0 = (px / physical_w) * 2.0 - 1.0;
            let y0 = 1.0 - (py / physical_h) * 2.0;
            let x1 = ((px + pw) / physical_w) * 2.0 - 1.0;
            let y1 = 1.0 - ((py + ph) / physical_h) * 2.0;

            let color_arr = color.to_array();
            let base_idx = (i * 4) as u32;

            vertices.push(RectVertex { position: [x0, y0], color: color_arr });
            vertices.push(RectVertex { position: [x1, y0], color: color_arr });
            vertices.push(RectVertex { position: [x1, y1], color: color_arr });
            vertices.push(RectVertex { position: [x0, y1], color: color_arr });

            indices.push(base_idx);
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);
            indices.push(base_idx);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        }

        gpu.queue.write_buffer(&self.rect_vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        gpu.queue.write_buffer(&self.rect_index_buffer, 0, bytemuck::cast_slice(&indices));

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Rect Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.rect_pipeline);
        render_pass.set_vertex_buffer(0, self.rect_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.rect_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }

    /// Draw a single image with the given texture view
    pub fn draw_image(
        &self,
        gpu: &GpuContext,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        texture_view: &TextureView,
        rect: &Rect,
        opacity: f32,
        viewport_width: f32,
        viewport_height: f32,
        scale_factor: f32,
    ) {
        // Physical viewport dimensions
        let physical_w = viewport_width * scale_factor;
        let physical_h = viewport_height * scale_factor;

        // Scale logical rect coordinates to physical
        let px = rect.x * scale_factor;
        let py = rect.y * scale_factor;
        let pw = rect.width * scale_factor;
        let ph = rect.height * scale_factor;

        let x0 = (px / physical_w) * 2.0 - 1.0;
        let y0 = 1.0 - (py / physical_h) * 2.0;
        let x1 = ((px + pw) / physical_w) * 2.0 - 1.0;
        let y1 = 1.0 - ((py + ph) / physical_h) * 2.0;

        let vertices = [
            ImageVertex { position: [x0, y0], tex_coords: [0.0, 0.0] },
            ImageVertex { position: [x1, y0], tex_coords: [1.0, 0.0] },
            ImageVertex { position: [x1, y1], tex_coords: [1.0, 1.0] },
            ImageVertex { position: [x0, y1], tex_coords: [0.0, 1.0] },
        ];

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        gpu.queue.write_buffer(&self.image_vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        gpu.queue.write_buffer(&self.image_index_buffer, 0, bytemuck::cast_slice(&indices));

        let uniforms = ImageUniforms {
            opacity,
            _padding: [0.0; 3],
        };
        gpu.queue.write_buffer(&self.image_uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let bind_group = gpu.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Image Bind Group"),
            layout: &self.image_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.image_sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: self.image_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Image Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.image_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.image_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.image_index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
