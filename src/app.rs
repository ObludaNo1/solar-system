use std::{
    iter::once,
    sync::{Arc, Mutex},
    time::Instant,
};

use cgmath::{InnerSpace, Point3, Vector3};
use wgpu::*;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};

use crate::{
    camera::{camera_control::CameraControl, movement_control::MovementControl},
    render_target::RenderTargetConfig,
    scene::Scene,
    solar_object::solar_object::load_solar_objects,
};

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

    fn render(&mut self) -> Result<(), SurfaceError> {
        if let Some(ref mut inner) = self.inner {
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
            // ignore other events
            _ => {
                if let Some(ref mut inner) = self.inner {
                    inner.movement_control.process_window_event(event);
                } else {
                    eprintln!("Inner app is not initialized");
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(ref mut inner) = self.inner {
            inner.movement_control.process_device_event(event);
        } else {
            eprintln!("Inner app is not initialized");
        }
    }
}

#[derive(Debug)]
struct AppInner {
    window: Arc<Window>,
    device: Device,
    render_target: RenderTargetConfig<'static>,
    queue: Queue,
    scene: Scene,
    movement_control: MovementControl,
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

        let camera_control = Arc::new(Mutex::new(CameraControl::new(
            Point3::new(0.0, 100.0, -200.0),
            Vector3::new(0.0, -1.0, 2.0).normalize(),
        )));
        let movement_control = MovementControl::new(camera_control.clone(), {
            let window = window.clone();
            move |dragging| {
                if dragging {
                    match window
                        .set_cursor_grab(CursorGrabMode::Locked)
                        .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Confined))
                    {
                        Ok(()) => window.set_cursor_visible(false),
                        Err(e) => eprintln!("Failed to grab cursor: {}", e),
                    }
                } else {
                    window
                        .set_cursor_grab(CursorGrabMode::None)
                        .expect("Releasing cursor grab cannot fail");
                    window.set_cursor_visible(true);
                }
            }
        });

        let scene = Scene::new(
            &device,
            &queue,
            &render_target,
            Instant::now(),
            camera_control.clone(),
            load_solar_objects("data/definitions.toml"),
        );

        Ok(AppInner {
            window,
            device,
            render_target,
            queue,
            scene,
            movement_control,
        })
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.render_target.resize(&self.device, new_size);
        self.scene.resize(&self.queue, new_size, Instant::now());
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        let render_target = self.render_target.next_frame()?;

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.scene.update_buffers(&self.queue, Instant::now());

        self.scene
            .record_draw_commands(&mut encoder, &render_target);

        self.queue.submit(once(encoder.finish()));
        render_target.present();

        Ok(())
    }
}
