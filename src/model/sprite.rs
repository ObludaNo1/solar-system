use bytemuck::cast_slice;
use wgpu::{
    BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

use super::{Mesh, Model, Vertex};

pub fn create_sprite(device: &Device, z_offset: f32) -> Model {
    #[rustfmt::skip]
    let vertices = [
        [-0.5, -0.5,  z_offset,  1.0,  0.0,  0.0],
        [-0.5,  0.5,  z_offset,  1.0,  1.0,  0.0],
        [ 0.5,  0.5,  z_offset,  0.0,  1.0,  1.0],
        [ 0.5, -0.5,  z_offset,  0.0,  0.0,  1.0],
    ];

    let vertices = vertices
        .into_iter()
        .map(|data| Vertex {
            position: [data[0], data[1], data[2]],
            colour: [data[3], data[4], data[5]],
        })
        .collect::<Vec<_>>();

    let indices = [0u16, 1, 2, 0, 2, 3];

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Cube Vertex Buffer"),
        contents: cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Cube Index Buffer"),
        contents: cast_slice(&indices),
        usage: BufferUsages::INDEX,
    });

    Model {
        meshes: vec![Mesh {
            vertex_buffer,
            index_buffer,
        }],
    }
}
