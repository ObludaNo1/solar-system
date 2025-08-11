use std::f32::consts::PI;

use bytemuck::cast_slice;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};

use crate::texture::texture::{RgbaTexture, TextureBindGroupDescriptor};

use super::{Mesh, Model, Vertex};

pub fn create_sphere(
    device: &Device,
    texture: RgbaTexture,
    texture_layout: TextureBindGroupDescriptor,
    radius: f32,
    lat_segments: u32,
    long_segments: u32,
) -> Model {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for y in 0..=lat_segments {
        let theta = PI * (y as f32) / (lat_segments as f32);
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        for x in 0..=long_segments {
            let phi = 2.0 * PI * (x as f32) / (long_segments as f32);
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let px = radius * sin_theta * cos_phi;
            let py = radius * cos_theta;
            let pz = radius * sin_theta * sin_phi;

            vertices.push(Vertex {
                position: [px, py, pz],
                tex_coords: [
                    x as f32 / long_segments as f32,
                    1.0 - y as f32 / lat_segments as f32,
                ],
            });
        }
    }

    // Generate indices
    for y in 0..lat_segments {
        for x in 0..long_segments {
            let i0 = y * (long_segments + 1) + x;
            let i1 = i0 + 1;
            let i2 = i0 + long_segments + 1;
            let i3 = i2 + 1;

            indices.push(i0 as u16);
            indices.push(i2 as u16);
            indices.push(i1 as u16);

            indices.push(i1 as u16);
            indices.push(i2 as u16);
            indices.push(i3 as u16);
        }
    }

    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Sphere Vertex Buffer"),
        contents: cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("Sphere Index Buffer"),
        contents: cast_slice(&indices),
        usage: BufferUsages::INDEX,
    });

    let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("Texture Bind Group"),
        entries: &[
            BindGroupEntry {
                binding: texture_layout.binding_view,
                resource: BindingResource::TextureView(&texture.view),
            },
            BindGroupEntry {
                binding: texture_layout.binding_sampler,
                resource: BindingResource::Sampler(&texture.sampler),
            },
        ],
        layout: &texture_layout.layout,
    });

    Model {
        texture,
        texture_bind_group,
        meshes: vec![Mesh {
            vertex_buffer,
            index_buffer,
        }],
    }
}
