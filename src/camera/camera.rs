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
}

impl Camera {
    pub fn new(camera_control: Arc<Mutex<CameraControl>>, projection: Projection) -> Self {
        Self {
            camera_control,
            projection,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.projection.resize(new_size);
    }

    pub fn view_proj_matrix(&mut self, now: Instant) -> Matrix4x4 {
        Matrix4x4::view_proj(
            self.camera_control.lock().unwrap().snapshot(now),
            self.projection.matrix(),
        )
    }
}
