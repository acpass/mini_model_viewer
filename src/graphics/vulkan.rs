mod debug;
use ash::vk;

use super::GraphicsBackend;
use crate::graphics::{GraphicsError, GraphicsResult};
use std::ffi::CStr;

#[derive(Default)]
pub struct VulkanGraphics {
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    device: Option<vk::PhysicalDevice>,

    debug_util: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<ash::vk::DebugUtilsMessengerEXT>,
}

impl VulkanGraphics {
    fn init_vulkan(&mut self) -> GraphicsResult<()> {
        self.entry = Some(unsafe { ash::Entry::load().expect("No vulkan support") });
        self.init_instance()?;
        self.init_physical_device()?;
        self.setup_debug_messenger();
        Ok(())
    }

    fn init_instance(&mut self) -> GraphicsResult<()> {
        if !Self::check_layer_support(
            &self.entry.as_ref().unwrap(),
            c"VK_LAYER_KHRONOS_validation",
        ) {
            return Err(GraphicsError::VulkanError(
                "Validation layer VK_LAYER_KHRONOS_validation not found".to_string(),
            ));
        }

        let app_info = ash::vk::ApplicationInfo::default()
            .application_name(&unsafe { CStr::from_ptr((c"a").as_ptr()) })
            .api_version(ash::vk::make_api_version(0, 1, 0, 0));

        let extensions = [
            ash::khr::portability_enumeration::NAME.as_ptr(),
            ash::khr::surface::NAME.as_ptr(),
            ash::ext::debug_utils::NAME.as_ptr(),
        ];
        let validation_layer_names = Self::get_validation_layer_names();

        let create_info = ash::vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions) // needed by MoltenVK on macOS
            .enabled_layer_names(&validation_layer_names)
            .flags(ash::vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR);
        let allocation_callbacks = None;

        self.instance = Some(
            unsafe {
                self.entry
                    .as_ref()
                    .unwrap()
                    .create_instance(&create_info, allocation_callbacks)
            }
            .expect("Instance create error"),
        );
        println!(
            "Vulkan instance created successfully: {:?}",
            self.instance.as_ref().unwrap().handle()
        );
        Ok(())
    }

    fn find_queue_families(
        &self,
        device: vk::PhysicalDevice,
    ) -> GraphicsResult<vk::QueueFamilyProperties> {
        let properties = unsafe {
            self.instance
                .as_ref()
                .unwrap()
                .get_physical_device_queue_family_properties(device)
        };
        properties
            .into_iter()
            .find(|&prop| prop.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .ok_or_else(|| {
                GraphicsError::VulkanError("Failed to find a graphics queue family".to_string())
            })
    }

    fn is_device_suitable(&self, device: vk::PhysicalDevice) -> bool {
        self.find_queue_families(device).is_ok()
    }

    fn init_physical_device(&mut self) -> GraphicsResult<()> {
        unsafe { self.instance.as_ref().unwrap().enumerate_physical_devices() }
            .map_err(|e| {
                GraphicsError::VulkanError(format!("Failed to enumerate physical devices: {:?}", e))
            })?
            .into_iter()
            .find(|&device| self.is_device_suitable(device))
            .ok_or_else(|| GraphicsError::VulkanError("Failed to find a suitable GPU".to_string()))
            .map(|device| {
                self.device = Some(device);
                println!("Selected physical device: {:?}", device);
            })
    }

    fn destroy_vulkan(&mut self) {
        if let Some(instance) = &self.instance {
            unsafe {
                instance.destroy_instance(None);
            }
            self.instance = None;
            println!("Vulkan instance destroyed");
        }
        self.destroy_debug_messenger();
    }
}

impl GraphicsBackend for VulkanGraphics {
    fn can_create_surface(&mut self, width: u32, height: u32) -> GraphicsResult<()> {
        self.init_vulkan()?;
        println!("Vulkan can create surface with size {}x{}", width, height);
        Ok(())
    }

    fn draw(&self) {
        println!("Vulkan Draw");
    }

    fn clear(&mut self) {
        self.destroy_vulkan();
        println!("Vulkan Clear");
    }

    fn resize(&mut self, width: u32, height: u32) {
        println!("Vulkan Resize to {}x{}", width, height);
    }
}
