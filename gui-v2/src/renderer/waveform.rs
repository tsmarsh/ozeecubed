use std::collections::VecDeque;
use wgpu::util::DeviceExt;

use ozeecubed_core::oscilloscope::TriggerSettings;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct WaveformRenderer {
    pipeline: wgpu::RenderPipeline,
    grid_buffer: wgpu::Buffer,
    grid_vertex_count: u32,
}

impl WaveformRenderer {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Waveform Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/waveform.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Waveform Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Waveform Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create grid
        let grid_vertices = Self::create_grid();
        let grid_vertex_count = grid_vertices.len() as u32;
        let grid_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Buffer"),
            contents: bytemuck::cast_slice(&grid_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            grid_buffer,
            grid_vertex_count,
        }
    }

    fn create_grid() -> Vec<Vertex> {
        let mut vertices = Vec::new();
        let grid_color = [0.0, 1.0, 0.16, 0.3]; // Green with alpha

        // Vertical lines (10 divisions)
        for i in 0..=10 {
            let x = -1.0 + (i as f32 / 10.0) * 2.0;
            vertices.push(Vertex {
                position: [x, -1.0],
                color: grid_color,
            });
            vertices.push(Vertex {
                position: [x, 1.0],
                color: grid_color,
            });
        }

        // Horizontal lines (8 divisions)
        for i in 0..=8 {
            let y = -1.0 + (i as f32 / 8.0) * 2.0;
            vertices.push(Vertex {
                position: [-1.0, y],
                color: grid_color,
            });
            vertices.push(Vertex {
                position: [1.0, y],
                color: grid_color,
            });
        }

        vertices
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        waveform_history: &VecDeque<Vec<(f32, f32)>>,
        _trigger_settings: &TriggerSettings,
    ) {
        // Create all buffers before starting render pass
        let mut waveform_buffers = Vec::new();
        let num_frames = waveform_history.len();

        for (frame_idx, points) in waveform_history.iter().enumerate() {
            if points.len() < 2 {
                continue;
            }

            // Calculate alpha for persistence effect
            let alpha = (frame_idx as f32 + 1.0) / num_frames as f32;
            let color = [0.0, 1.0, 0.16, alpha]; // Green with varying alpha

            // Convert points to vertices
            let mut vertices = Vec::new();
            for window in points.windows(2) {
                let (x1, y1) = window[0];
                let (x2, y2) = window[1];

                // Convert from normalized coordinates to clip space
                let x1_clip = x1 * 2.0 - 1.0;
                let y1_clip = -(y1 * 2.0 - 1.0); // Flip Y
                let x2_clip = x2 * 2.0 - 1.0;
                let y2_clip = -(y2 * 2.0 - 1.0); // Flip Y

                vertices.push(Vertex {
                    position: [x1_clip, y1_clip],
                    color,
                });
                vertices.push(Vertex {
                    position: [x2_clip, y2_clip],
                    color,
                });
            }

            if !vertices.is_empty() {
                let vertex_count = vertices.len() as u32;
                let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Waveform Buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                waveform_buffers.push((buffer, vertex_count));
            }
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Waveform Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Waveform Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);

            // Draw grid
            render_pass.set_vertex_buffer(0, self.grid_buffer.slice(..));
            render_pass.draw(0..self.grid_vertex_count, 0..1);

            // Draw waveform history with persistence
            for (buffer, vertex_count) in &waveform_buffers {
                render_pass.set_vertex_buffer(0, buffer.slice(..));
                render_pass.draw(0..*vertex_count, 0..1);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
