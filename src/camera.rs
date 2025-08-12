use cgmath::Point3;
use winit::dpi::PhysicalSize;

use crate::matrix::view_proj_mat::ViewProjMat;

pub struct Camera {
    pub position: Point3<f32>,
    pub wh_ratio: f32,
}

impl Camera {
    pub fn set_wh_ratio(&mut self, size: PhysicalSize<u32>) {
        self.wh_ratio = size.width as f32 / size.height as f32;
    }

    pub fn view_proj_matrix(&self) -> ViewProjMat {
        ViewProjMat::look_at_center(self.position, self.wh_ratio)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Point3::new(2.0, 2.0, 2.0),
            wh_ratio: 60.0,
        }
    }
}
