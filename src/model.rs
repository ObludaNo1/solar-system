use bytemuck::{Pod, Zeroable};
use wgpu::*;

pub mod sprite;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

impl Vertex {
    pub fn desc() -> &'static VertexBufferLayout<'static> {
        &VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x3,
            }],
        }
    }
}

pub struct Mesh {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

pub struct Model {
    meshes: Vec<Mesh>,
}

pub struct MeshBuffers<'a> {
    pub vertex_buffer: BufferSlice<'a>,
    pub index_buffer: BufferSlice<'a>,
    pub index_format: IndexFormat,
}

impl<'a> Model {
    pub fn meshes(&'a self) -> impl Iterator<Item = MeshBuffers<'a>> {
        self.meshes.iter().map(|mesh| MeshBuffers {
            vertex_buffer: mesh.vertex_buffer.slice(..),
            index_buffer: mesh.index_buffer.slice(..),
            index_format: IndexFormat::Uint16,
        })
    }
}
