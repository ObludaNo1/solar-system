use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use winit::{
    event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::camera::camera_control::{CameraControl, MovementDirection};

#[derive(Debug)]
pub struct MovementControl {
    camera_control: Arc<Mutex<CameraControl>>,
    mouse_pressed: bool,
}

impl MovementControl {
    pub fn new(camera_control: Arc<Mutex<CameraControl>>) -> Self {
        MovementControl {
            camera_control,
            mouse_pressed: false,
        }
    }

    pub fn process_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        ..
                    },
                ..
            } => {
                use KeyCode::*;
                let pressed = state == ElementState::Pressed;
                match key {
                    KeyW => self
                        .camera_control
                        .lock()
                        .unwrap()
                        .move_forw_backw(Instant::now(), MovementDirection::positive(pressed)),
                    KeyS => self
                        .camera_control
                        .lock()
                        .unwrap()
                        .move_forw_backw(Instant::now(), MovementDirection::negative(pressed)),
                    KeyA => self
                        .camera_control
                        .lock()
                        .unwrap()
                        .move_sideways(Instant::now(), MovementDirection::negative(pressed)),
                    KeyD => self
                        .camera_control
                        .lock()
                        .unwrap()
                        .move_sideways(Instant::now(), MovementDirection::positive(pressed)),
                    Space => self
                        .camera_control
                        .lock()
                        .unwrap()
                        .move_vertical(Instant::now(), MovementDirection::positive(pressed)),
                    ControlLeft => self
                        .camera_control
                        .lock()
                        .unwrap()
                        .move_vertical(Instant::now(), MovementDirection::negative(pressed)),
                    _ => {}
                }
            }
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Right) => {
                    self.mouse_pressed = true;
                }
                (ElementState::Released, MouseButton::Right) => {
                    self.mouse_pressed = false;
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn process_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion {
                delta: (delta_x, delta_y),
            } => {
                if self.mouse_pressed {
                    self.camera_control.lock().unwrap().rotate(
                        Instant::now(),
                        delta_x as f32,
                        delta_y as f32,
                    );
                }
            }
            _ => {}
        }
    }
}
