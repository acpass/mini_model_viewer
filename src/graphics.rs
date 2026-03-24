pub mod vulkan;

pub trait GraphicsBackend {
    fn can_create_surface(&mut self, width: u32, height: u32) -> GraphicsResult<()>;
    fn draw(&self);
    fn clear(&mut self);
    fn resize(&mut self, width: u32, height: u32);
}

#[derive(Debug)]
pub enum GraphicsError {
    VulkanError(String),
}

pub type GraphicsResult<T> = Result<T, GraphicsError>;
