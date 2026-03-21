use super::GraphicsBackend;

#[derive(Default)]
pub struct VulkanGraphics {
    instance: Option<ash::Instance>,
}

impl VulkanGraphics {
    fn init_vulkan(&mut self) {
        let entry: ash::Entry = unsafe { ash::Entry::load().unwrap() };
        let app_info =
            ash::vk::ApplicationInfo::default().api_version(ash::vk::make_api_version(0, 1, 0, 0));
        let extensions = [ash::khr::portability_enumeration::NAME.as_ptr()];
        let create_info = ash::vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .flags(ash::vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR);
        let allocation_callbacks = None;
        self.instance = Some(
            unsafe { entry.create_instance(&create_info, allocation_callbacks) }
                .expect("instance create error"),
        );
        println!(
            "Vulkan instance created successfully: {:?}",
            self.instance.as_ref().unwrap().handle()
        );
    }
}

impl GraphicsBackend for VulkanGraphics {
    fn can_create_surface(&mut self, width: u32, height: u32) {
        self.init_vulkan();
        println!("Vulkan can create surface with size {}x{}", width, height);
    }

    fn draw(&self) {
        println!("Vulkan Draw");
    }

    fn clear(&mut self) {
        println!("Vulkan Clear");
    }

    fn resize(&mut self, width: u32, height: u32) {
        println!("Vulkan Resize to {}x{}", width, height);
    }
}
