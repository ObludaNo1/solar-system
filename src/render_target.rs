use wgpu::*;
use winit::dpi::PhysicalSize;

pub struct RenderTargetConfig<'window> {
    surface: Surface<'window>,
    config: SurfaceConfiguration,
    depth_texture: (Texture, TextureView),
}

const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

impl<'window> RenderTargetConfig<'window> {
    pub fn new(
        size: PhysicalSize<u32>,
        device: &Device,
        surface: Surface<'window>,
        adapter: &Adapter,
    ) -> Result<Self, SurfaceError> {
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

        let depth_texture = create_depth_texture(
            &device,
            PhysicalSize {
                width: config.width,
                height: config.height,
            },
        );

        Ok(RenderTargetConfig {
            surface,
            config,
            depth_texture,
        })
    }

    pub fn resize(&mut self, device: &Device, new_size: PhysicalSize<u32>) {
        self.config.width = new_size.width.max(1);
        self.config.height = new_size.height.max(1);
        self.surface.configure(device, &self.config);
        self.depth_texture = create_depth_texture(&device, new_size);
    }

    /// Gets new render target with surface colour buffer attached to it.
    ///
    /// # Panics
    ///
    /// Panics, if previous result is not yet dropped.
    pub fn next_frame(&self) -> Result<RenderTarget<'_>, SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;
        Ok(RenderTarget {
            config: self,
            surface_texture,
        })
    }

    pub fn depth_texture_view(&self) -> &TextureView {
        &self.depth_texture.1
    }

    pub fn target_texture_format(&self) -> TextureFormat {
        self.config.format
    }

    pub fn depth_texture_format(&self) -> TextureFormat {
        DEPTH_FORMAT
    }
}

pub struct RenderTarget<'window> {
    pub config: &'window RenderTargetConfig<'window>,
    pub surface_texture: SurfaceTexture,
}

impl<'window> RenderTarget<'window> {
    pub fn present(self) {
        self.surface_texture.present();
    }

    pub fn surface_texture_view(&self) -> TextureView {
        self.surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default())
    }
}

fn create_depth_texture(device: &Device, size: PhysicalSize<u32>) -> (Texture, TextureView) {
    let size = Extent3d {
        width: size.width.max(1),
        height: size.height.max(1),
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("Depth Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[DEPTH_FORMAT],
    });
    let view = texture.create_view(&TextureViewDescriptor::default());

    (texture, view)
}
