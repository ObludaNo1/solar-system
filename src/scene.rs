use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use bytemuck::cast_slice;
use cgmath::Vector3;
use image::ImageReader;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::dpi::PhysicalSize;

use crate::{
    camera::{camera::Camera, camera_control::CameraControl, projection::Projection},
    matrix::Matrix,
    model::{Model, ModelBindGroupDescriptor, sphere::create_sphere},
    model_render_pass::ModelRenderPass,
    render_target::{RenderTarget, RenderTargetConfig},
    texture::texture::RgbaTexture,
};

#[derive(Debug)]
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
            contents: cast_slice(&[Matrix::identity()]),
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

#[derive(Debug)]
pub struct Scene {
    init_time: Instant,
    model_render_pass: ModelRenderPass,
    models: Vec<SceneModel>,
    camera: Camera,
    view_proj_buffer: Buffer,
}

impl Scene {
    pub fn new(
        device: &Device,
        queue: &Queue,
        render_target: &RenderTargetConfig,
        now: Instant,
        camera_control: Arc<Mutex<CameraControl>>,
    ) -> Scene {
        let mut camera = Camera::new(camera_control, Projection::default());
        let view_proj_mat = camera.view_proj_matrix(now);
        let view_proj_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("view-proj buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[view_proj_mat]),
        });

        let model_render_pass = ModelRenderPass::new(device, render_target, &view_proj_buffer);

        let texture_layout = model_render_pass.texture_layout();
        let model_layout = model_render_pass.model_layout();

        Scene {
            init_time: now,
            models: vec![
                SceneModel::new(
                    device,
                    create_sphere(
                        device,
                        RgbaTexture::from_image(
                            device,
                            queue,
                            ImageReader::open("resources/2k_sun.jpg")
                                .unwrap()
                                .decode()
                                .unwrap(),
                        ),
                        texture_layout,
                        0.5,
                        16,
                        32,
                        Matrix::scale(Vector3 {
                            x: 695.7,
                            y: 695.7,
                            z: 695.7,
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
                            ImageReader::open("resources/2k_earth_daymap.jpg")
                                .unwrap()
                                .decode()
                                .unwrap(),
                        ),
                        texture_layout,
                        0.5,
                        16,
                        32,
                        Matrix::scale(Vector3 {
                            x: 6.378,
                            y: 6.356,
                            z: 6.378,
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
                            ImageReader::open("resources/2k_mars.jpg")
                                .unwrap()
                                .decode()
                                .unwrap(),
                        ),
                        texture_layout,
                        0.5,
                        16,
                        32,
                        Matrix::scale(Vector3 {
                            x: 3.396,
                            y: 3.376,
                            z: 3.396,
                        }),
                    ),
                    &model_layout,
                ),
            ],
            model_render_pass,
            camera,
            view_proj_buffer,
        }
    }

    pub fn resize(&mut self, queue: &Queue, new_size: PhysicalSize<u32>, now: Instant) {
        self.camera.resize(new_size);
        queue.write_buffer(
            &self.view_proj_buffer,
            0,
            cast_slice(&[self.camera.view_proj_matrix(now)]),
        );
    }

    pub fn update_buffers(&mut self, queue: &Queue, now: Instant) {
        queue.write_buffer(
            &self.view_proj_buffer,
            0,
            cast_slice(&[self.camera.view_proj_matrix(now)]),
        );

        let rotation = (now - self.init_time).as_secs_f32();

        for (i, scene_model) in self.models.iter().enumerate() {
            let translate = Matrix::translate(Vector3 {
                x: 0.0,
                y: 0.0,
                z: i as f32 * 1.0,
            });
            let rotate = Matrix::rotate(Vector3::unit_y(), rotation * (i as f32 + 0.3));
            queue.write_buffer(
                &scene_model.model_buffer,
                0,
                cast_slice(&[translate * rotate * *scene_model.model.model_matrix()]),
            );
        }
    }

    pub fn record_draw_commands(&self, encoder: &mut CommandEncoder, render_target: &RenderTarget) {
        self.model_render_pass
            .record_draw_commands(encoder, render_target, &self.models);
    }
}
