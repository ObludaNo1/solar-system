use std::{iter::once, sync::Arc};

use wgpu::*;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::scene::Scene;

pub struct App {
    inner: Option<AppInner>,
}

impl App {
    pub fn new() -> App {
        App { inner: None }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if let Some(ref mut inner) = self.inner {
            inner.resize(new_size);
        }
    }

    fn render(&self) -> Result<(), SurfaceError> {
        if let Some(ref inner) = self.inner {
            inner.render()
        } else {
            Ok(())
        }
    }

    fn request_redraw(&self) {
        if let Some(ref inner) = self.inner {
            inner.window.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // TODO not ideal to block on here, unless other thread does it
        self.inner = Some(pollster::block_on(async {
            AppInner::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("Solar system"))
                    .unwrap(),
            )
            .await
        }));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                self.resize(new_size);
            }
            WindowEvent::RedrawRequested => {
                self.request_redraw();
                let render_result = self.render();
                match render_result {
                    Ok(()) => {}
                    Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                        todo!("there should be resize")
                    }
                    Err(SurfaceError::Timeout) => {
                        println!("frame timeout");
                    }
                    Err(e) => {
                        eprintln!("{e:?}");
                        event_loop.exit();
                    }
                }
            }
            // ignore other events like mouse events, keyboard events, etc
            _ => {}
        }
    }
}

struct AppInner {
    window: Arc<Window>,
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    scene: Scene,
}

impl AppInner {
    async fn new(window: Window) -> AppInner {
        let size = window.inner_size();
        let window_ref = Arc::new(window);
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window_ref.clone()).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                memory_hints: Default::default(),
                required_limits: Limits::default(),
                trace: Trace::Off,
            })
            .await
            .unwrap();

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
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode,
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        let scene = Scene::new(&device, &config);

        AppInner {
            window: window_ref,
            surface,
            device,
            queue,
            config,
            scene,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&self) -> Result<(), SurfaceError> {
        let render_target = self.surface.get_current_texture()?;
        let view = render_target
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.scene
            .record_draw_commands(&self.queue, &mut encoder, &view);

        self.queue.submit(once(encoder.finish()));
        render_target.present();

        Ok(())
    }
}
