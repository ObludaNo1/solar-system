use std::{
    f64::consts::PI,
    time::{SystemTime, UNIX_EPOCH},
};

use bytemuck::checked::cast_slice;
use cgmath::{Point3, Vector3};
use image::{Rgba, RgbaImage};
use rand::Rng;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use crate::{
    matrix::{model_mat::ModelMat, view_proj_mat::ViewProjMat},
    model::{MeshBuffers, Model, Vertex},
    render_target::{RenderTarget, RenderTargetConfig},
    texture::texture::RgbaTexture,
};

pub struct ModelRenderPass {
    render_pipeline: RenderPipeline,
    view_proj_bind_group: BindGroup,
    model_buffer: Buffer,
    model_bind_group: BindGroup,
    #[allow(unused)]
    texture: RgbaTexture,
    texture_bind_group: BindGroup,
}

impl ModelRenderPass {
    pub fn new(
        device: &Device,
        queue: &Queue,
        render_target: &RenderTargetConfig,
    ) -> ModelRenderPass {
        // define, how the uniforms look like
        let matrix_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("matrix 4x4 layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
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

        let view_proj_mat = ViewProjMat::look_at_center(Point3::new(2.0, 2.0, 2.0));
        let model_mat = ModelMat::identity();

        let view_proj_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("view-proj buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[view_proj_mat]),
        });
        let model_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("model buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[model_mat]),
        });

        let view_proj_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("view-proj bind group"),
            layout: &matrix_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: view_proj_buffer.as_entire_binding(),
            }],
        });
        let model_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("model bind group"),
            layout: &matrix_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: model_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &matrix_bind_group_layout,
                &matrix_bind_group_layout,
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

        let mut rng = rand::rng();
        let image = RgbaImage::from_fn(128, 128, |_, _| {
            Rgba([rng.random(), rng.random(), rng.random(), 255])
        });

        let texture = RgbaTexture::from_image(device, queue, image.into());

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ],
            layout: &texture_bind_group_layout,
        });

        ModelRenderPass {
            render_pipeline,
            view_proj_bind_group,
            model_buffer,
            model_bind_group,
            texture,
            texture_bind_group,
        }
    }

    pub fn record_draw_commands(
        &self,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        render_target: &RenderTarget,
        models: &[Model],
    ) {
        queue.write_buffer(&self.model_buffer, 0, cast_slice(&[create_rot_matrix()]));
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &render_target.surface_texture_view(),
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
        render_pass.set_bind_group(0, &self.view_proj_bind_group, &[]);
        render_pass.set_bind_group(1, &self.model_bind_group, &[]);
        render_pass.set_bind_group(2, &self.texture_bind_group, &[]);
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

fn create_rot_matrix() -> ModelMat {
    ModelMat::rotate(
        Vector3::unit_y(),
        (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time is larger than UNIX EPOCH")
            .as_secs_f64()
            % (2.0 * PI)) as f32,
    )
}
