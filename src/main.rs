use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod matrix;
mod model;
mod model_render_pass;
mod render_target;
mod scene;

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    pollster::block_on(run());
}
