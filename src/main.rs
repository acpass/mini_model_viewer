use std::error::Error;

use ac_mini_model_viewer::{app::App, graphics::vulkan::VulkanGraphics};
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    let event = EventLoop::new()?;
    event.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let graphics = VulkanGraphics::default();
    let mut app = App::new(graphics);
    event.run_app(&mut app)?;
    Ok(())
}
