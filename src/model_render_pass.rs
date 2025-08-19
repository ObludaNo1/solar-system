use bytemuck::cast_slice;
use cgmath::{Matrix4, Vector4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use crate::{
    camera::camera::Camera,
    model::{MeshBuffers, Vertex, VertexBindGroupDescriptor},
    render_target::{RenderTarget, RenderTargetConfig},
    scene::SceneModel,
    texture::texture::TextureBindGroupDescriptor,
};

#[derive(Debug)]
pub struct ModelRenderPass {
    render_pipeline: RenderPipeline,
    vertex_mat_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
    light_pos_group: BindGroup,
    light_pos_buffer: Buffer,
}

impl ModelRenderPass {
    pub fn new(device: &Device, render_target: &RenderTargetConfig) -> ModelRenderPass {
        let vertex_bind_group_entry = |binding: u32| BindGroupLayoutEntry {
            binding,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let vertex_mat_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("view-proj layout"),
            entries: &[
                vertex_bind_group_entry(0),
                vertex_bind_group_entry(1),
                vertex_bind_group_entry(2),
            ],
        });

        let light_pos_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("camera space light pos layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let light_pos_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Light Position Buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[0.0f32, 0.0, 0.0, 0.0]),
        });
        let light_pos_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Light Position Bind Group"),
            layout: &light_pos_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: light_pos_buffer.as_entire_binding(),
            }],
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
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
                ],
            });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                // mvp matrix, mv matrix, normal matrix
                &vertex_mat_layout,
                &light_pos_layout,
                &texture_bind_group_layout,
            ],
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
                    format: render_target.target_texture_format(),
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
            depth_stencil: Some(DepthStencilState {
                format: render_target.depth_texture_format(),
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState {
                    constant: 2, // Corresponds to bilinear filtering
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
            }),
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

        ModelRenderPass {
            render_pipeline,
            vertex_mat_layout,
            light_pos_buffer,
            light_pos_group,
            texture_bind_group_layout,
        }
    }

    pub fn vertex_matrix_layout(&self) -> VertexBindGroupDescriptor<'_> {
        VertexBindGroupDescriptor {
            layout: &self.vertex_mat_layout,
            mvp_binding: 0,
            mv_binding: 1,
            normal_binding: 2,
        }
    }

    pub fn texture_layout(&self) -> TextureBindGroupDescriptor<'_> {
        TextureBindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            binding_view: 0,
            binding_sampler: 1,
        }
    }

    pub fn update_buffers(&self, queue: &Queue, camera: &Camera) {
        let view_matrix: Matrix4<f32> = camera.view_matrix().data.into();
        let light_pos = Vector4::new(0.0, 0.0, 0.0, 1.0);
        let camera_space_light: [f32; 4] = (view_matrix * light_pos).into();
        queue.write_buffer(&self.light_pos_buffer, 0, cast_slice(&[camera_space_light]));
    }

    pub fn record_draw_commands<'a>(
        &self,
        encoder: &mut CommandEncoder,
        render_target: &RenderTarget,
        models: impl Iterator<Item = &'a SceneModel>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &render_target.surface_texture_view(),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: render_target.config.depth_texture_view(),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.light_pos_group, &[]);
        for scene_model in models {
            render_pass.set_bind_group(0, &scene_model.model_bind_group, &[]);
            render_pass.set_bind_group(2, scene_model.model.texture_bind_group(), &[]);
            for MeshBuffers {
                texture_bind_group,
                vertex_buffer,
                index_buffer,
                index_format,
            } in scene_model.model.meshes()
            {
                render_pass.set_bind_group(2, texture_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer);
                render_pass.set_index_buffer(index_buffer, index_format);
                // Index buffer contains u16 indices stored in u8 array. The number of elements is
                // therefore half of its size.
                render_pass.draw_indexed(0..index_buffer.size().get() as u32 / 2, 0, 0..1);
            }
        }
    }
}
