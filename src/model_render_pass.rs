use std::time::{SystemTime, UNIX_EPOCH};

use wgpu::*;

use crate::model::{MeshBuffers, Model, Vertex};

pub struct ModelRenderPass {
    render_pipeline: RenderPipeline,
}

impl ModelRenderPass {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> ModelRenderPass {
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("model_shader.wgsl").into()),
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc().clone()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState {
                        color: BlendComponent::REPLACE,
                        alpha: BlendComponent::REPLACE,
                    }),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            // Useful for optimizing shader compilation on Android
            cache: None,
        });

        ModelRenderPass { render_pipeline }
    }

    pub fn record_draw_commands(
        &self,
        encoder: &mut CommandEncoder,
        render_target: &TextureView,
        models: &[Model],
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: render_target,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("current time is larger than UNIX EPOCH")
                            .as_secs_f64()
                            .sin()
                            * 0.5
                            + 0.5,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        for model in models {
            for MeshBuffers {
                vertex_buffer,
                index_buffer,
                index_format,
            } in model.meshes()
            {
                render_pass.set_vertex_buffer(0, vertex_buffer);
                render_pass.set_index_buffer(index_buffer, index_format);
                // Index buffer contains u16 indices stored in u8 array. The number of elements is
                // therefore half of its size.
                render_pass.draw_indexed(0..index_buffer.size().get() as u32 / 2, 0, 0..1);
            }
        }
    }
}
