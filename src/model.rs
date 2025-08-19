use bytemuck::{Pod, Zeroable};
use wgpu::*;

use crate::{matrix::Matrix4x4, texture::texture::RgbaTexture};

pub mod sphere;
// pub mod sprite;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    normal: [f32; 3],
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Vertex {
    pub fn desc() -> &'static VertexBufferLayout<'static> {
        &VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

#[derive(Debug)]
pub struct Model {
    #[allow(unused)]
    texture: RgbaTexture,
    texture_bind_group: BindGroup,
    model_matrix: Matrix4x4,
    meshes: Vec<Mesh>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexBindGroupDescriptor<'a> {
    pub layout: &'a BindGroupLayout,
    pub mvp_binding: u32,
    pub mv_binding: u32,
    pub normal_binding: u32,
}

#[derive(Debug)]
pub struct MeshBuffers<'a> {
    pub texture_bind_group: &'a BindGroup,
    pub vertex_buffer: BufferSlice<'a>,
    pub index_buffer: BufferSlice<'a>,
    pub index_format: IndexFormat,
}

impl<'a> Model {
    pub fn model_matrix(&self) -> &Matrix4x4 {
        &self.model_matrix
    }

    pub fn texture_bind_group(&self) -> &BindGroup {
        &self.texture_bind_group
    }

    pub fn meshes(&'a self) -> impl Iterator<Item = MeshBuffers<'a>> {
        self.meshes.iter().map(|mesh| MeshBuffers {
            texture_bind_group: &self.texture_bind_group,
            vertex_buffer: mesh.vertex_buffer.slice(..),
            index_buffer: mesh.index_buffer.slice(..),
            index_format: IndexFormat::Uint16,
        })
    }
}
