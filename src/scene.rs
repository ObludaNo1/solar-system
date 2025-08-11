use std::time::SystemTime;

use bytemuck::cast_slice;
use cgmath::Vector3;
use image::{Rgba, RgbaImage};
use rand::Rng;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use crate::{
    matrix::model_mat::ModelMat,
    model::{Model, ModelBindGroupDescriptor, sphere::create_sphere},
    model_render_pass::ModelRenderPass,
    render_target::{RenderTarget, RenderTargetConfig},
    texture::texture::RgbaTexture,
};

pub struct SceneModel {
    pub model: Model,
    pub model_buffer: Buffer,
    pub model_bind_group: BindGroup,
}

impl SceneModel {
    pub fn new(device: &Device, model: Model, model_layout: &ModelBindGroupDescriptor) -> Self {
        let model_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("model buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[ModelMat::identity()]),
        });
        let model_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("model bind group"),
            layout: &model_layout.layout,
            entries: &[BindGroupEntry {
                binding: model_layout.binding,
                resource: model_buffer.as_entire_binding(),
            }],
        });
        Self {
            model,
            model_buffer,
            model_bind_group,
        }
    }
}

pub struct Scene {
    init_time: SystemTime,
    model_render_pass: ModelRenderPass,
    models: Vec<SceneModel>,
}

impl Scene {
    pub fn new(device: &Device, queue: &Queue, render_target: &RenderTargetConfig) -> Scene {
        let model_render_pass = ModelRenderPass::new(device, render_target);

        let texture_layout = model_render_pass.texture_layout();
        let model_layout = model_render_pass.model_layout();

        let mut rng = rand::rng();

        Scene {
            init_time: SystemTime::now(),
            models: vec![
                SceneModel::new(
                    device,
                    create_sphere(
                        device,
                        RgbaTexture::from_image(
                            device,
                            queue,
                            RgbaImage::from_fn(128, 128, |_, _| Rgba([rng.random(), 0, 0, 255]))
                                .into(),
                        ),
                        texture_layout,
                        0.5,
                        16,
                        32,
                        ModelMat::identity(),
                    ),
                    &model_layout,
                ),
                SceneModel::new(
                    device,
                    create_sphere(
                        device,
                        RgbaTexture::from_image(
                            device,
                            queue,
                            RgbaImage::from_fn(128, 128, |_, _| Rgba([0, rng.random(), 0, 255]))
                                .into(),
                        ),
                        texture_layout,
                        0.5,
                        16,
                        32,
                        ModelMat::scale(Vector3 {
                            x: 1.0,
                            y: 1.5,
                            z: 0.8,
                        }),
                    ),
                    &model_layout,
                ),
                SceneModel::new(
                    device,
                    create_sphere(
                        device,
                        RgbaTexture::from_image(
                            device,
                            queue,
                            RgbaImage::from_fn(128, 128, |_, _| Rgba([0, 0, rng.random(), 255]))
                                .into(),
                        ),
                        texture_layout,
                        0.5,
                        16,
                        32,
                        ModelMat::translate(Vector3 {
                            x: 0.5,
                            y: 0.0,
                            z: 0.0,
                        }),
                    ),
                    &model_layout,
                ),
            ],
            model_render_pass,
        }
    }

    pub fn record_draw_commands(
        &self,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        render_target: &RenderTarget,
    ) {
        let rotation = SystemTime::now()
            .duration_since(self.init_time)
            .expect("current time is larger than UNIX EPOCH")
            .as_secs_f64() as f32;

        for (i, scene_model) in self.models.iter().enumerate() {
            let translate = ModelMat::translate(Vector3 {
                x: 0.0,
                y: 0.0,
                z: i as f32 * 1.0,
            });
            let rotate = ModelMat::rotate(Vector3::unit_y(), rotation * (i as f32 + 0.3));
            queue.write_buffer(
                &scene_model.model_buffer,
                0,
                bytemuck::cast_slice(&[translate * rotate * *scene_model.model.model_matrix()]),
            );
        }

        self.model_render_pass
            .record_draw_commands(encoder, render_target, &self.models);
    }
}
