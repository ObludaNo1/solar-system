use std::{f64::consts::PI, time::Duration};

use bytemuck::cast_slice;
use cgmath::Vector3;
use image::DynamicImage;
use wgpu::*;

use crate::{
    camera::camera_control::UP,
    matrix::Matrix,
    model::{ModelBindGroupDescriptor, sphere::create_sphere},
    scene::SceneModel,
    solar_object::solar_object::SolarObject,
    texture::texture::{RgbaTexture, TextureBindGroupDescriptor},
};

/// This function makes things in solar system reasonably in vision range. Otherwise all bodies are
/// so far they are not visible.
pub fn distance_scaling(distance: f64) -> f32 {
    (distance / 10000.0).powf(0.5) as f32
}

pub fn radius_scaling(radius: f64) -> f32 {
    (radius / 10000.0).powf(0.5) as f32
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
        model_layout: ModelBindGroupDescriptor,
        texture_layout: TextureBindGroupDescriptor,
    ) -> Self {
        RenderSolarObject::new_inner(
            SolarObjectInner::new(solar_object),
            queue,
            device,
            model_layout,
            texture_layout,
        )
    }

    fn new_inner(
        mut solar_object: SolarObjectInner,
        queue: &Queue,
        device: &Device,
        model_layout: ModelBindGroupDescriptor,
        texture_layout: TextureBindGroupDescriptor,
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
                    RenderSolarObject::new_inner(child, queue, device, model_layout, texture_layout)
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
                    Matrix::identity(),
                ),
                model_layout,
            ),
        }
    }

    pub fn update_buffers(&self, time: Duration, queue: &Queue) {
        self.update_buffers_inner(time, queue, Matrix::identity(), None);
    }

    fn update_buffers_inner(
        &self,
        time: Duration,
        queue: &Queue,
        parent_matrix: Matrix,
        parent_radius: Option<f32>,
    ) {
        let scale = radius_scaling(self.radius_km);
        let scale = Matrix::scale(Vector3::new(scale, scale, scale));
        let rotate = Matrix::rotate(
            UP,
            time_scaling(PI * 2.0 * time.as_secs_f64() / self.rotation_period_days),
        );
        let tilt = Matrix::rotate(Vector3::unit_x(), self.tilt as f32);
        let translate = Matrix::translate(Vector3 {
            x: distance_scaling(self.distance_from_parent_km)
                + parent_radius
                    .map(|r| radius_scaling(r as f64) + radius_scaling(self.radius_km))
                    .unwrap_or(0.0),
            y: 0.0,
            z: 0.0,
        });
        let orbit = if let Some(orbital_period_days) = self.orbital_period_days {
            Matrix::rotate(
                UP,
                time_scaling(PI * 2.0 * time.as_secs_f64() / orbital_period_days),
            )
        } else {
            Matrix::identity()
        };
        let model_matrix = parent_matrix
            * orbit
            * translate
            * tilt
            * rotate
            * scale
            * *self.scene_model.model.model_matrix();
        queue.write_buffer(
            &self.scene_model.model_buffer,
            0,
            cast_slice(&[model_matrix]),
        );
        for child in &self.children {
            child.update_buffers_inner(
                time,
                queue,
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
