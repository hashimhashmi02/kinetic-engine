//! The solid-colour pipeline.

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::Frame;

/// Samples per pixel for antialiasing.
pub const SAMPLE_COUNT: u32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Camera {
    scale: [f32; 2],
    _pad: [f32; 2],
}

struct Msaa {
    view: wgpu::TextureView,
    size: (u32, u32),
}

/// Draws [`Frame`]s into a texture view. Owns the pipeline and the multisampled buffer;
/// does not own the target, so the same renderer serves windows, exports and tests.
pub struct Renderer {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    camera: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    msaa: Option<Msaa>,
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer").field("format", &self.format).finish_non_exhaustive()
    }
}

impl Renderer {
    /// Builds the pipeline for targets of `format`.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let camera = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("kinetic camera"),
            size: std::mem::size_of::<Camera>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("kinetic camera"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kinetic camera"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: camera.as_entire_binding() }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("kinetic solid"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("kinetic solid"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
                })],
            },
            primitive: wgpu::PrimitiveState {
                // 2D shapes have no back faces; tessellators do not promise a winding.
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState { count: SAMPLE_COUNT, ..Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self { format, pipeline, camera, bind_group, msaa: None }
    }

    fn msaa_view(&mut self, device: &wgpu::Device, size: (u32, u32)) -> &wgpu::TextureView {
        if self.msaa.as_ref().is_none_or(|m| m.size != size) {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("kinetic msaa"),
                size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: SAMPLE_COUNT,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            self.msaa = Some(Msaa { view: texture.create_view(&Default::default()), size });
        }
        // Just set above when it was missing.
        &self.msaa.as_ref().expect("msaa target exists").view
    }

    /// Encodes one pass that clears `target` and draws `frame` into it.
    ///
    /// `target` must be a single-sample view of this renderer's format, `size` pixels
    /// large (both non-zero).
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        size: (u32, u32),
        frame: &Frame,
    ) {
        // Scene units to clip space: the visible rectangle spans [-1, 1] on both axes.
        let camera = Camera {
            scale: [(2.0 / frame.size.x) as f32, (2.0 / frame.size.y) as f32],
            _pad: [0.0; 2],
        };
        queue.write_buffer(&self.camera, 0, bytemuck::bytes_of(&camera));

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for item in &frame.items {
            let base = vertices.len() as u32;
            let color = item.color.to_array();
            let (ox, oy) = (item.offset.x as f32, item.offset.y as f32);
            vertices.extend(
                item.mesh
                    .vertices
                    .iter()
                    .map(|v| Vertex { position: [v[0] + ox, v[1] + oy], color }),
            );
            indices.extend(item.mesh.indices.iter().map(|i| base + i));
        }
        let buffers = (!indices.is_empty()).then(|| {
            let make = |label, contents: &[u8], usage| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(label),
                    contents,
                    usage,
                })
            };
            (
                make(
                    "kinetic vertices",
                    bytemuck::cast_slice(&vertices),
                    wgpu::BufferUsages::VERTEX,
                ),
                make("kinetic indices", bytemuck::cast_slice(&indices), wgpu::BufferUsages::INDEX),
            )
        });

        let bg = frame.background;
        let clear = wgpu::Color {
            r: f64::from(bg.r),
            g: f64::from(bg.g),
            b: f64::from(bg.b),
            a: f64::from(bg.a),
        };
        let msaa = self.msaa_view(device, size).clone();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("kinetic frame"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &msaa,
                depth_slice: None,
                resolve_target: Some(target),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    // The multisampled buffer is resolved into the target and not reused.
                    store: wgpu::StoreOp::Discard,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if let Some((vertex_buffer, index_buffer)) = &buffers {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
    }
}
