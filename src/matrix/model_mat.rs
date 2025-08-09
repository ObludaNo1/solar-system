use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Matrix4, Rad, SquareMatrix, Vector3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelMat {
    // The `data` field is required for uniform layout, even if not read directly.
    #[allow(dead_code)]
    data: [[f32; 4]; 4],
}

unsafe impl Pod for ModelMat {}
unsafe impl Zeroable for ModelMat {}

impl ModelMat {
    pub fn identity() -> Self {
        ModelMat {
            data: Matrix4::identity().into(),
        }
    }

    pub fn rotate(axis: Vector3<f32>, angle: f32) -> Self {
        ModelMat {
            data: Matrix4::from_axis_angle(axis.normalize(), Rad(angle % (2.0 * PI))).into(),
        }
    }
}
