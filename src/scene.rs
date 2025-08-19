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
    model::{Model, VertexBindGroupDescriptor},
    model_render_pass::ModelRenderPass,
    render_target::{RenderTarget, RenderTargetConfig},
    solar_object::{render_solar_object::RenderSolarObject, solar_object::SolarObject},
};

#[derive(Debug)]
pub struct SceneModel {
    pub model: Model,
    pub model_bind_group: BindGroup,
    pub mvp_matrix: Buffer,
    pub mv_matrix: Buffer,
    pub normal_matrix: Buffer,
}

impl SceneModel {
    pub fn new(
        device: &Device,
        model: Model,
        model_normal_matrix_layout: VertexBindGroupDescriptor,
    ) -> Self {
        let mvp_matrix = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("model buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[Matrix4x4::identity()]),
        });
        let mv_matrix = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("model buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[Matrix4x4::identity()]),
        });
        let normal_matrix = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("normal buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: cast_slice(&[Matrix3x3::identity().byte_aligned()]),
        });
        let model_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("model bind group"),
            layout: &model_normal_matrix_layout.layout,
            entries: &[
                BindGroupEntry {
                    binding: model_normal_matrix_layout.mvp_binding,
                    resource: mvp_matrix.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: model_normal_matrix_layout.mv_binding,
                    resource: mv_matrix.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: model_normal_matrix_layout.normal_binding,
                    resource: normal_matrix.as_entire_binding(),
                },
            ],
        });
        Self {
            model,
            model_bind_group,
            mvp_matrix,
            mv_matrix,
            normal_matrix,
        }
    }
}

#[derive(Debug)]
pub struct Scene {
    init_time: Instant,
    model_render_pass: ModelRenderPass,
    camera: Camera,
    sun: RenderSolarObject,
}

impl Scene {
    pub fn new(
        device: &Device,
        queue: &Queue,
        render_target: &RenderTargetConfig,
        now: Instant,
        camera_control: Arc<Mutex<CameraControl>>,
        sun: SolarObject,
    ) -> Scene {
        let camera = Camera::new(camera_control, Projection::default());

        let model_render_pass = ModelRenderPass::new(device, render_target);

        let texture_layout = model_render_pass.texture_layout();
        let vertex_matrix_layout = model_render_pass.vertex_matrix_layout();

        let sun = RenderSolarObject::new(sun, queue, device, vertex_matrix_layout, texture_layout);

        Scene {
            init_time: now,
            model_render_pass,
            camera,
            sun,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.camera.resize(new_size);
    }

    pub fn update_buffers(&mut self, queue: &Queue, now: Instant) {
        self.camera.update_view_proj_matrices(now);
        self.model_render_pass.update_buffers(queue, &self.camera);
        self.sun
            .update_buffers(now - self.init_time, queue, &self.camera);
    }

    pub fn record_draw_commands(&self, encoder: &mut CommandEncoder, render_target: &RenderTarget) {
        self.model_render_pass.record_draw_commands(
            encoder,
            render_target,
            self.sun.models().into_iter(),
        );
    }
}
