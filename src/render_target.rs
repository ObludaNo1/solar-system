use wgpu::*;
use winit::dpi::PhysicalSize;

pub struct RenderTarget<'window> {
    surface: Surface<'window>,
    config: SurfaceConfiguration,
}

impl<'window> RenderTarget<'window> {
    pub fn new(size: PhysicalSize<u32>, surface: Surface<'window>, adapter: &Adapter) -> Self {
        let surface_caps = surface.get_capabilities(&adapter);

        // Shader code in this project assumes an Srgb surface texture. Using a different one will
        // result all the colors comming out darker. If you want to support non Srgb surfaces,
        // you'll need to account for that when drawing to the frame.
        let format = *surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .unwrap_or(&surface_caps.formats[0]);
        // TODO using FIFO can make the app really non-interactive. It is best to use MailBox
        // present mode for better responsiveness of the application. However many frames can be
        // generated without being presented. Therefore the best solution would be to use MailBox
        // with some CPU limited frame rate.
        let present_mode = *surface_caps
            .present_modes
            .iter()
            .find(|pm| **pm == PresentMode::Mailbox)
            .unwrap_or(&surface_caps.present_modes[0]);
        let alpha_mode = *surface_caps
            .alpha_modes
            .iter()
            .find(|a| **a == CompositeAlphaMode::PostMultiplied)
            .unwrap_or(&surface_caps.alpha_modes[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        RenderTarget { surface, config }
    }

    pub fn resize(&mut self, device: &Device, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(device, &self.config);
    }

    pub fn target_texture_format(&self) -> TextureFormat {
        self.config.format
    }

    pub fn get_current_texture(&self) -> Result<(SurfaceTexture, TextureView), SurfaceError> {
        let colour_buffer = self.surface.get_current_texture()?;
        let view = colour_buffer
            .texture
            .create_view(&TextureViewDescriptor::default());
        Ok((colour_buffer, view))
    }
}
