use winit::{
    application::ApplicationHandler,
    dpi,
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
                .create_window(
                    WindowAttributes::default()
                        .with_title("AC Mini Model Viewer")
                        .with_inner_size(dpi::PhysicalSize::new(800, 600)),
                )
                .unwrap(),
        );

        let window = self.window.as_ref().unwrap();
        let wh = window.window_handle().unwrap();
        let dh = window.display_handle().unwrap();
        let window_handle = graphics::WindowHandlePara::new(&wh, &dh);
        let size = window.inner_size();
        self.graphics
            .can_create_surface(&window_handle, size.width, size.height)
            .inspect_err(|e| println!("Failed to create graphics surface: {:?}", e))
            .unwrap();
        window.request_redraw();
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
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::Resized(_) => {
                let inner_size = self.window.as_ref().unwrap().inner_size();
                self.graphics.resize(inner_size.width, inner_size.height);
                println!(
                    "Window resized to {}x{}",
                    inner_size.width, inner_size.height
                );
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event: keyboard_event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(KeyCode::Escape) = keyboard_event.physical_key {
                    println!("Escape key pressed, exiting");
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }
}
