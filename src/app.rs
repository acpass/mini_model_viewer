use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop, window::WindowAttributes,
};

use crate::graphics::GraphicsBackend;

pub struct App<G: GraphicsBackend> {
    graphics: G,
    window: Option<winit::window::Window>,
}

impl<G: GraphicsBackend> App<G> {
    pub fn new(graphics: G) -> App<G> {
        App {
            graphics,
            window: None,
        }
    }
}

impl<G: GraphicsBackend> ApplicationHandler for App<G> {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(WindowAttributes::default().with_title("AC Mini Model Viewer"))
                .unwrap(),
        );
        self.graphics.can_create_surface(800, 600);
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.graphics.draw();
                println!("Window redraw requested, window id: {:?}", window_id);
            }
            WindowEvent::Resized(size) => {
                self.graphics.resize(size.width, size.height);
                println!("Window resized to {}x{}", size.width, size.height);
            }
            _ => {}
        }
    }
}
