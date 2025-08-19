use std::{f64::consts::PI, time::Duration};

use bytemuck::cast_slice;
use cgmath::Vector3;
use image::DynamicImage;
use wgpu::*;

use crate::{
    camera::{camera::Camera, camera_control::UP},
    matrix::{Matrix3x3, Matrix4x4},
    model::{VertexBindGroupDescriptor, sphere::create_sphere},
    scene::SceneModel,
    solar_object::solar_object::SolarObject,
    texture::texture::{RgbaTexture, TextureBindGroupDescriptor},
};

/// This function makes things in solar system reasonably in vision range. Otherwise all bodies are
/// so far they are not visible.
pub fn distance_scaling(distance: f64) -> f32 {
    (distance / 100000.0).powf(0.6) as f32
}

pub fn radius_scaling(radius: f64) -> f32 {
    (radius / 10000.0).powf(0.4) as f32
}

pub fn time_scaling(time: f64) -> f32 {
    (time * 10.0) as f32
}

#[derive(Debug)]
pub struct RenderSolarObject {
    pub radius_km: f64,
    pub distance_from_parent_km: f64,
    pub orbital_period_days: Option<f64>,
    pub rotation_period_days: f64,
    pub tilt: f64,
    pub children: Vec<RenderSolarObject>,
    pub scene_model: SceneModel,
    pub inverse_normals: bool,
}

struct SolarObjectInner {
    radius_km: f64,
    distance_from_parent_km: f64,
    orbital_period_days: Option<f64>,
    rotation_period_days: f64,
    tilt: f64,
    texture_image: Option<DynamicImage>,
    children: Vec<SolarObjectInner>,
}

impl SolarObjectInner {
    pub fn new(solar_object: SolarObject) -> Self {
        Self {
            radius_km: solar_object.radius_km,
            distance_from_parent_km: solar_object.distance_from_parent_km,
            orbital_period_days: solar_object.orbital_period_days,
            rotation_period_days: solar_object.rotation_period_days,
            tilt: solar_object.tilt,
            texture_image: Some(solar_object.texture_image),
            children: solar_object
                .children
                .into_iter()
                .map(SolarObjectInner::new)
                .collect(),
        }
    }
}

impl RenderSolarObject {
    pub fn new(
        solar_object: SolarObject,
        queue: &Queue,
        device: &Device,
        model_normal_matrix_layout: VertexBindGroupDescriptor,
        texture_layout: TextureBindGroupDescriptor,
    ) -> Self {
        RenderSolarObject::new_inner(
            SolarObjectInner::new(solar_object),
            queue,
            device,
            model_normal_matrix_layout,
            texture_layout,
            true,
        )
    }

    fn new_inner(
        mut solar_object: SolarObjectInner,
        queue: &Queue,
        device: &Device,
        model_normal_matrix_layout: VertexBindGroupDescriptor,
        texture_layout: TextureBindGroupDescriptor,
        inverse_normals: bool,
    ) -> Self {
        let texture = RgbaTexture::from_image(
            device,
            queue,
            solar_object
                .texture_image
                .take()
                .expect("Texture is present"),
        );
        Self {
            radius_km: solar_object.radius_km,
            distance_from_parent_km: solar_object.distance_from_parent_km,
            orbital_period_days: solar_object.orbital_period_days,
            rotation_period_days: solar_object.rotation_period_days,
            tilt: solar_object.tilt,
            children: solar_object
                .children
                .into_iter()
                .map(|child| {
                    RenderSolarObject::new_inner(
                        child,
                        queue,
                        device,
                        model_normal_matrix_layout,
                        texture_layout,
                        false,
                    )
                })
                .collect(),
            scene_model: SceneModel::new(
                device,
                create_sphere(
                    device,
                    texture,
                    texture_layout,
                    1.0,
                    64,
                    128,
                    Matrix4x4::identity(),
                ),
                model_normal_matrix_layout,
            ),
            inverse_normals,
        }
    }

    pub fn update_buffers(&self, time: Duration, queue: &Queue, camera: &Camera) {
        self.update_buffers_inner(time, queue, camera, Matrix4x4::identity(), None);
    }

    fn update_buffers_inner(
        &self,
        time: Duration,
        queue: &Queue,
        camera: &Camera,
        parent_matrix: Matrix4x4,
        parent_radius: Option<f32>,
    ) {
        let scale = radius_scaling(self.radius_km);
        let scale = Matrix4x4::scale(Vector3::new(scale, scale, scale));
        let rotate = Matrix4x4::rotate(
            UP,
            time_scaling(PI * 2.0 * time.as_secs_f64() / self.rotation_period_days),
        );
        let tilt = Matrix4x4::rotate(Vector3::unit_x(), self.tilt as f32);
        let translate = Matrix4x4::translate(Vector3 {
            x: distance_scaling(self.distance_from_parent_km)
                + parent_radius
                    .map(|r| radius_scaling(r as f64) + radius_scaling(self.radius_km))
                    .unwrap_or(0.0),
            y: 0.0,
            z: 0.0,
        });
        let orbit = if let Some(orbital_period_days) = self.orbital_period_days {
            Matrix4x4::rotate(
                UP,
                time_scaling(PI * 2.0 * time.as_secs_f64() / orbital_period_days),
            )
        } else {
            Matrix4x4::identity()
        };
        let model_matrix = parent_matrix
            * orbit
            * translate
            * tilt
            * rotate
            * scale
            * *self.scene_model.model.model_matrix();
        let mut normal_matrix = Matrix3x3::to_mat3_inverse_transpose(model_matrix);
        if self.inverse_normals {
            normal_matrix = Matrix3x3::scale(Vector3::new(-1.0, -1.0, -1.0)) * normal_matrix;
        }

        let view_matrix = camera.view_matrix();
        let projection_matrix = camera.projection_matrix();

        queue.write_buffer(
            &self.scene_model.mvp_matrix,
            0,
            cast_slice(&[projection_matrix * view_matrix * model_matrix]),
        );
        queue.write_buffer(
            &self.scene_model.mv_matrix,
            0,
            cast_slice(&[view_matrix * model_matrix]),
        );
        queue.write_buffer(
            &self.scene_model.normal_matrix,
            0,
            cast_slice(&[normal_matrix.byte_aligned()]),
        );

        for child in &self.children {
            child.update_buffers_inner(
                time,
                queue,
                camera,
                parent_matrix * orbit * translate,
                Some(self.radius_km as f32),
            );
        }
    }

    pub fn models(&self) -> Vec<&SceneModel> {
        let mut models = Vec::new();
        self.collect_models(&mut models);
        models
    }

    #[inline]
    fn collect_models<'a>(&'a self, data: &mut Vec<&'a SceneModel>) {
        data.push(&self.scene_model);
        for child in &self.children {
            child.collect_models(data);
        }
    }
}
