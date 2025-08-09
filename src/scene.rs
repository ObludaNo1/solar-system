use wgpu::*;

use crate::{
    model::{Model, sprite::create_sprite},
    model_render_pass::ModelRenderPass,
};

pub struct Scene {
    model_render_pass: ModelRenderPass,
    models: Vec<Model>,
}

impl Scene {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Scene {
        let model_render_pass = ModelRenderPass::new(device, config);
        Scene {
            model_render_pass,
            models: vec![create_sprite(device)],
        }
    }

    pub fn record_draw_commands(
        &self,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        render_target: &TextureView,
    ) {
        self.model_render_pass
            .record_draw_commands(queue, encoder, render_target, &self.models);
    }
}
