use std::{error::Error, path::Path};

use ac_mini_model_viewer::{app::App, graphics::vulkan::VulkanGraphics};
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    let event = EventLoop::new()?;
    event.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let shader_path =
        std::env::var("SHADER_OUT_PATH").unwrap_or_else(|_| "assets/shaders".to_string());
    let graphics = VulkanGraphics::default().set_shader_path(Path::new(&shader_path));
    let mut app = App::new(graphics);
    event.run_app(&mut app)?;
    Ok(())
}
