use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use bytemuck::cast_slice;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::dpi::PhysicalSize;

use crate::{
    camera::{camera::Camera, camera_control::CameraControl, projection::Projection},
    matrix::{Matrix3x3, Matrix4x4},
    model::{Model, ModelNormalBindGroupDescriptor},
    model_render_pass::ModelRenderPass,
    render_target::{RenderTarget, RenderTargetConfig},
    solar_object::{render_solar_object::RenderSolarObject, solar_object::SolarObject},
};

#[derive(Debug)]
pub struct SceneModel {
    pub model: Model,
    pub model_matrix_buffer: Buffer,
    pub model_bind_group: BindGroup,
    pub normal_matrix_buffer: Buffer,
}

impl SceneModel {
    pub fn new(
        device: &Device,
        model: Model,
        model_normal_matrix_layout: ModelNormalBindGroupDescriptor,
    ) -> Self {
        let model_matrix_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("model buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[Matrix4x4::identity()]),
        });
        let normal_matrix_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("normal buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[Matrix3x3::identity().byte_aligned()]),
        });
        let model_matrix_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("model bind group"),
            layout: &model_normal_matrix_layout.layout,
            entries: &[
                BindGroupEntry {
                    binding: model_normal_matrix_layout.model_binding,
                    resource: model_matrix_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: model_normal_matrix_layout.normal_binding,
                    resource: normal_matrix_buffer.as_entire_binding(),
                },
            ],
        });
        Self {
            model,
            model_matrix_buffer,
            model_bind_group: model_matrix_bind_group,
            normal_matrix_buffer,
        }
    }
}

#[derive(Debug)]
pub struct Scene {
    init_time: Instant,
    model_render_pass: ModelRenderPass,
    camera: Camera,
    view_proj_buffer: Buffer,
    solar_object: RenderSolarObject,
}

impl Scene {
    pub fn new(
        device: &Device,
        queue: &Queue,
        render_target: &RenderTargetConfig,
        now: Instant,
        camera_control: Arc<Mutex<CameraControl>>,
        solar_object: SolarObject,
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
        let model_normal_matrix_layout = model_render_pass.model_normal_matrix_layout();

        let solar_object = RenderSolarObject::new(
            solar_object,
            queue,
            device,
            model_normal_matrix_layout,
            texture_layout,
        );

        Scene {
            init_time: now,
            model_render_pass,
            camera,
            view_proj_buffer,
            solar_object,
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

        self.solar_object
            .update_buffers(now - self.init_time, queue);
    }

    pub fn record_draw_commands(&self, encoder: &mut CommandEncoder, render_target: &RenderTarget) {
        self.model_render_pass.record_draw_commands(
            encoder,
            render_target,
            self.solar_object.models().into_iter(),
        );
    }
}
