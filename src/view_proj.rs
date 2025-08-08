use bytemuck::{Pod, Zeroable};
use cgmath::{Deg, Matrix4, Point3, Vector3, perspective};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Debug, Clone, Copy)]
pub struct ViewProjMat {
    // The `data` field is required for uniform layout, even if not read directly.
    #[allow(dead_code)]
    data: [[f32; 4]; 4],
}

unsafe impl Pod for ViewProjMat {}
unsafe impl Zeroable for ViewProjMat {}

impl ViewProjMat {
    pub fn look_at_center(eye: Point3<f32>) -> Self {
        let view =
            Matrix4::look_at_rh(eye, Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
        let proj = perspective(Deg(60.0), 1.0, 0.1, 100.0);
        ViewProjMat {
            data: (OPENGL_TO_WGPU_MATRIX * proj * view).into(),
        }
    }
}
