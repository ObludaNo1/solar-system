use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use winit::dpi::PhysicalSize;

use crate::{
    camera::{camera_control::CameraControl, projection::Projection},
    matrix::Matrix4x4,
};

#[derive(Debug, Clone)]
pub struct Camera {
    pub camera_control: Arc<Mutex<CameraControl>>,
    pub projection: Projection,

    view_matrix: Matrix4x4,
    projection_matrix: Matrix4x4,
}

impl Camera {
    pub fn new(camera_control: Arc<Mutex<CameraControl>>, projection: Projection) -> Self {
        Self {
            camera_control,
            projection,
            view_matrix: Matrix4x4::identity(),
            projection_matrix: Matrix4x4::identity(),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.projection.resize(new_size);
    }

    pub fn update_view_proj_matrices(&mut self, now: Instant) {
        self.view_matrix = self.camera_control.lock().unwrap().snapshot(now).into();
        self.projection_matrix = self.projection.matrix().into();
    }

    pub fn view_matrix(&self) -> Matrix4x4 {
        self.view_matrix
    }

    pub fn projection_matrix(&self) -> Matrix4x4 {
        self.projection_matrix
    }
}
