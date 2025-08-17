use cgmath::{Deg, Matrix4, perspective};
use winit::dpi::PhysicalSize;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection {
    aspect_ratio: f32,
    fov: f32,
    near: f32,
    far: f32,
}

impl Projection {
    pub fn new(size: PhysicalSize<u32>, fov: f32, near: f32, far: f32) -> Self {
        let aspect_ratio = size.width as f32 / size.height as f32;
        Projection {
            aspect_ratio,
            fov,
            near,
            far,
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.aspect_ratio = size.width as f32 / size.height as f32;
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(Deg(self.fov), self.aspect_ratio, self.near, self.far)
    }
}

impl Default for Projection {
    fn default() -> Self {
        Projection::new(PhysicalSize::new(1, 1), 45.0, 0.1, 100.0)
    }
}
