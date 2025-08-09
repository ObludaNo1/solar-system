use wgpu::*;

use crate::{
    model::{Model, sphere::create_sphere},
    model_render_pass::ModelRenderPass,
    render_target::{RenderTarget, RenderTargetConfig},
};

pub struct Scene {
    model_render_pass: ModelRenderPass,
    models: Vec<Model>,
}

impl Scene {
    pub fn new(device: &Device, render_target: &RenderTargetConfig) -> Scene {
        let model_render_pass = ModelRenderPass::new(device, render_target);
        Scene {
            model_render_pass,
            models: vec![create_sphere(device, 1.0, 16, 32)],
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
