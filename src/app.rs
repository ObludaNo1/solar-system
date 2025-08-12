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

use crate::{render_target::RenderTargetConfig, scene::Scene};

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
        let result = pollster::block_on(async {
            AppInner::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("Solar system"))
                    .unwrap(),
            )
            .await
        });
        match result {
            Ok(inner) => self.inner = Some(inner),
            Err(e) => {
                eprintln!("Failed to create app: {e:?}");
                event_loop.exit();
            }
        }
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
    device: Device,
    render_target: RenderTargetConfig<'static>,
    queue: Queue,
    scene: Scene,
}

impl AppInner {
    async fn new(window: Window) -> Result<AppInner, SurfaceError> {
        let window = Arc::new(window);
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
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

        let render_target =
            RenderTargetConfig::new(window.inner_size(), &device, surface, &adapter)?;

        let scene = Scene::new(&device, &queue, &render_target);

        Ok(AppInner {
            window,
            device,
            render_target,
            queue,
            scene,
        })
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.render_target.resize(&self.device, new_size);
        self.scene.resize(&self.queue, new_size);
    }

    fn render(&self) -> Result<(), SurfaceError> {
        let render_target = self.render_target.next_frame()?;

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.scene
            .record_draw_commands(&self.queue, &mut encoder, &render_target);

        self.queue.submit(once(encoder.finish()));
        render_target.present();

        Ok(())
    }
}
