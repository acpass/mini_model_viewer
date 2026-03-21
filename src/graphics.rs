pub mod vulkan;

pub trait GraphicsBackend {
    fn can_create_surface(&mut self, width: u32, height: u32);
    fn draw(&self);
    fn clear(&mut self);
    fn resize(&mut self, width: u32, height: u32);
}
