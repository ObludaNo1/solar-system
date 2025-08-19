use std::{f32::consts::PI, ops::Mul};

use bytemuck::{Pod, Zeroable};
use cgmath::{InnerSpace, Matrix, Matrix3, Matrix4, Rad, SquareMatrix, Vector3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4x4 {
    // The `data` field is required for uniform layout, even if not read directly.
    #[allow(dead_code)]
    data: [[f32; 4]; 4],
}

unsafe impl Pod for Matrix4x4 {}
unsafe impl Zeroable for Matrix4x4 {}

impl Matrix4x4 {
    pub fn identity() -> Self {
        Matrix4x4 {
            data: Matrix4::identity().into(),
        }
    }

    pub fn rotate(axis: Vector3<f32>, angle: f32) -> Self {
        Matrix4x4 {
            data: Matrix4::from_axis_angle(axis.normalize(), Rad(angle % (2.0 * PI))).into(),
        }
    }

    pub fn translate(translation: Vector3<f32>) -> Self {
        Matrix4x4 {
            data: Matrix4::from_translation(translation).into(),
        }
    }

    pub fn scale(scale: Vector3<f32>) -> Self {
        Matrix4x4 {
            data: Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z).into(),
        }
    }

    pub fn view_proj(camera: Matrix4<f32>, projection: Matrix4<f32>) -> Self {
        Matrix4x4 {
            data: (projection * camera).into(),
        }
    }
}

impl Mul for Matrix4x4 {
    type Output = Matrix4x4;

    fn mul(self, rhs: Matrix4x4) -> Self::Output {
        Matrix4x4 {
            data: (Matrix4::from(self.data) * Matrix4::from(rhs.data)).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x3 {
    data: [[f32; 3]; 3],
}

impl Matrix3x3 {
    pub fn identity() -> Self {
        Matrix3x3 {
            data: Matrix3::identity().into(),
        }
    }

    pub fn to_mat3_inverse_transpose(source: Matrix4x4) -> Self {
        let matrix3 = Matrix3::new(
            source.data[0][0],
            source.data[0][1],
            source.data[0][2],
            source.data[1][0],
            source.data[1][1],
            source.data[1][2],
            source.data[2][0],
            source.data[2][1],
            source.data[2][2],
        );

        Matrix3x3 {
            data: matrix3
                .invert()
                .map(|inv| inv.transpose())
                .unwrap_or(Matrix3::identity())
                .into(),
        }
    }

    pub fn scale(scale: Vector3<f32>) -> Self {
        Matrix3x3 {
            data: Matrix3::new(scale.x, 0.0, 0.0, 0.0, scale.y, 0.0, 0.0, 0.0, scale.z).into(),
        }
    }

    pub fn byte_aligned(&self) -> Matrix3x3ByteAligned {
        Matrix3x3ByteAligned {
            data: [
                [self.data[0][0], self.data[0][1], self.data[0][2], 0.0],
                [self.data[1][0], self.data[1][1], self.data[1][2], 0.0],
                [self.data[2][0], self.data[2][1], self.data[2][2], 0.0],
            ],
        }
    }
}

impl Mul for Matrix3x3 {
    type Output = Matrix3x3;

    fn mul(self, rhs: Matrix3x3) -> Self::Output {
        Matrix3x3 {
            data: (Matrix3::from(self.data) * Matrix3::from(rhs.data)).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x3ByteAligned {
    // The `data` field is required for uniform layout, even if not read directly.
    #[allow(dead_code)]
    // IMPORTANT: any vec3<f32>, which includes matrix3x3<f32>, must have alignment of 16 bytes!
    data: [[f32; 4]; 3],
}

unsafe impl Pod for Matrix3x3ByteAligned {}
unsafe impl Zeroable for Matrix3x3ByteAligned {}
