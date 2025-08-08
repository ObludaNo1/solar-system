use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod model;
mod model_render_pass;
mod scene;
mod view_proj;

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}

fn main() {
    pollster::block_on(run());
}
