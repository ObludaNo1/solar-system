use image::{Rgba, RgbaImage};
use rand::Rng;
use wgpu::*;

use crate::{
    model::{Model, sphere::create_sphere, sprite::create_sprite},
    model_render_pass::ModelRenderPass,
    render_target::{RenderTarget, RenderTargetConfig},
    texture::texture::RgbaTexture,
};

pub struct Scene {
    model_render_pass: ModelRenderPass,
    models: Vec<Model>,
}

impl Scene {
    pub fn new(device: &Device, queue: &Queue, render_target: &RenderTargetConfig) -> Scene {
        let model_render_pass = ModelRenderPass::new(device, render_target);

        let texture_layout = model_render_pass.texture_layout();

        let mut rng = rand::rng();
        let image_sphere = RgbaImage::from_fn(128, 128, |_, _| {
            Rgba([rng.random(), rng.random(), rng.random(), 255])
        });
        let texture_sphere = RgbaTexture::from_image(device, queue, image_sphere.into());

        let image_sprite =
            RgbaImage::from_fn(4, 4, |x, y| Rgba([(x * 64) as u8, (y * 64) as u8, 0, 255]));
        let texture_sprite = RgbaTexture::from_image(device, queue, image_sprite.into());

        Scene {
            models: vec![
                create_sphere(device, texture_sphere, texture_layout, 1.0, 16, 32),
                create_sprite(device, texture_sprite, texture_layout, 1.0),
            ],
            model_render_pass,
        }
    }

    pub fn record_draw_commands(
        &self,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        render_target: &RenderTarget,
    ) {
        self.model_render_pass
            .record_draw_commands(queue, encoder, render_target, &self.models);
    }
}
