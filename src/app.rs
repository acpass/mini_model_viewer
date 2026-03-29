use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop,
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::WindowAttributes,
};

use crate::graphics::{self, GraphicsBackend};

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

        let window = self.window.as_ref().unwrap();
        let wh = window.window_handle().unwrap();
        let dh = window.display_handle().unwrap();
        let window_handle = graphics::window_handle::new(&wh, &dh);
        self.graphics
            .can_create_surface(&window_handle, 800, 600)
            .inspect_err(|e| println!("Failed to create graphics surface: {:?}", e))
            .unwrap();
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
                self.graphics.clear();
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
            WindowEvent::KeyboardInput {
                device_id,
                event: keyboard_event,
                is_synthetic,
            } => match keyboard_event.physical_key {
                PhysicalKey::Code(KeyCode::Escape) => {
                    println!("Escape key pressed, exiting");
                    event_loop.exit();
                }
                _ => {}
            },
            _ => {}
        }
    }
}
