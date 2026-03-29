use winit::raw_window_handle;

pub mod vulkan;

pub struct WindowHandlePara<
    'a,
    W: raw_window_handle::HasWindowHandle,
    D: raw_window_handle::HasDisplayHandle,
> {
    pub window: &'a W,
    pub display: &'a D,
}

impl<'a, W: raw_window_handle::HasWindowHandle, D: raw_window_handle::HasDisplayHandle>
    WindowHandlePara<'a, W, D>
{
    pub fn new(window: &'a W, display: &'a D) -> Self {
        Self { window, display }
    }
}

pub trait GraphicsBackend {
    fn can_create_surface<
        W: raw_window_handle::HasWindowHandle,
        D: raw_window_handle::HasDisplayHandle,
    >(
        &mut self,
        window: &WindowHandlePara<W, D>,
        width: u32,
        height: u32,
    ) -> GraphicsResult<()>;
    fn draw(&self);
    fn clear(&mut self);
    fn resize(&mut self, width: u32, height: u32);
}

#[derive(Debug)]
pub enum GraphicsError {
    VulkanError(String),
}

pub type GraphicsResult<T> = Result<T, GraphicsError>;
