mod debug;
use ash::vk;
use winit::raw_window_handle::{self, HasDisplayHandle};

use super::GraphicsBackend;
use crate::graphics::{GraphicsError, GraphicsResult};
use std::{collections::HashSet, ffi::CStr};

struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

#[derive(Default)]
pub struct VulkanGraphics {
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    surface_loader: Option<ash::khr::surface::Instance>,

    physical_device: Option<vk::PhysicalDevice>,
    logical_device: Option<ash::Device>,
    surface: Option<vk::SurfaceKHR>,
    queues: Vec<vk::Queue>,

    debug_util: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<ash::vk::DebugUtilsMessengerEXT>,
}

impl VulkanGraphics {
    fn init_vulkan<
        W: raw_window_handle::HasWindowHandle,
        D: raw_window_handle::HasDisplayHandle,
    >(
        &mut self,
        handle: &super::WindowHandlePara<W, D>,
    ) -> GraphicsResult<()> {
        self.entry = Some(unsafe { ash::Entry::load().expect("No vulkan support") });
        self.init_instance(handle)?;
        self.init_surface(handle)?;
        self.setup_debug_messenger();
        self.pick_physical_device()?;
        self.init_logical_device()?;
        self.init_queue()?;
        Ok(())
    }

    fn init_surface<
        W: raw_window_handle::HasWindowHandle,
        D: raw_window_handle::HasDisplayHandle,
    >(
        &mut self,
        handle: &super::WindowHandlePara<W, D>,
    ) -> GraphicsResult<()> {
        self.surface = Some(
            unsafe {
                ash_window::create_surface(
                    self.entry.as_ref().unwrap(),
                    self.instance.as_ref().unwrap(),
                    handle.display.display_handle().unwrap().as_raw(),
                    handle.window.window_handle().unwrap().as_raw(),
                    None,
                )
            }
            .map_err(|e| {
                GraphicsError::VulkanError(format!("Failed to create surface: {:?}", e))
            })?,
        );
        println!(
            "Vulkan surface created successfully: {:?}",
            self.surface.unwrap()
        );
        Ok(())
    }

    fn get_instance_extensions(display_handle: &dyn HasDisplayHandle) -> Vec<*const i8> {
        let mut extensions = ash_window::enumerate_required_extensions(
            display_handle.display_handle().unwrap().as_raw(),
        )
        .unwrap()
        .to_vec();
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            extensions.push(ash::khr::portability_enumeration::NAME.as_ptr());
        }
        extensions.push(ash::ext::debug_utils::NAME.as_ptr());
        extensions.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        extensions
    }

    fn get_device_extensions() -> Vec<*const i8> {
        #[allow(unused_mut)]
        let mut extensions = vec![ash::khr::swapchain::NAME.as_ptr()];
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            extensions.push(ash::khr::portability_subset::NAME.as_ptr());
        }
        extensions
    }

    fn get_instance_features_flags() -> ash::vk::InstanceCreateFlags {
        #[allow(unused_mut)]
        let mut flags = ash::vk::InstanceCreateFlags::empty();
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            flags = flags | ash::vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        }
        flags
    }

    fn init_instance<
        W: raw_window_handle::HasWindowHandle,
        D: raw_window_handle::HasDisplayHandle,
    >(
        &mut self,
        handle: &super::WindowHandlePara<W, D>,
    ) -> GraphicsResult<()> {
        if !Self::check_layer_support(self.entry.as_ref().unwrap(), c"VK_LAYER_KHRONOS_validation")
        {
            return Err(GraphicsError::VulkanError(
                "Validation layer VK_LAYER_KHRONOS_validation not found".to_string(),
            ));
        }

        let app_info = ash::vk::ApplicationInfo::default()
            .application_name(unsafe { CStr::from_ptr((c"a").as_ptr()) })
            .api_version(ash::vk::make_api_version(0, 1, 0, 0));

        let extensions = Self::get_instance_extensions(handle.display);
        let validation_layer_names = Self::get_validation_layer_names();

        let create_info = ash::vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions) // needed by MoltenVK on macOS
            .enabled_layer_names(&validation_layer_names)
            .flags(Self::get_instance_features_flags());
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

        self.surface_loader = Some(ash::khr::surface::Instance::new(
            self.entry.as_ref().unwrap(),
            self.instance.as_ref().unwrap(),
        ));
        Ok(())
    }

    fn find_queue_family_indice(
        &self,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &ash::khr::surface::Instance,
    ) -> QueueFamilyIndices {
        let properties = unsafe {
            self.instance
                .as_ref()
                .unwrap()
                .get_physical_device_queue_family_properties(device)
        };
        let graphics_family = properties
            .iter()
            .enumerate()
            .find(|(_, prop)| prop.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|(index, _)| index as u32);
        let present_family = properties
            .iter()
            .enumerate()
            .find(|(index, _)| {
                unsafe {
                    surface_loader.get_physical_device_surface_support(
                        device,
                        *index as u32,
                        surface,
                    )
                }
                .unwrap_or(false)
            })
            .map(|(index, _)| index as u32);
        let indices = QueueFamilyIndices {
            graphics_family,
            present_family,
        };
        indices
    }

    fn is_device_suitable(
        &self,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &ash::khr::surface::Instance,
    ) -> bool {
        self.find_queue_family_indice(device, surface, surface_loader)
            .is_complete()
    }

    fn pick_physical_device(&mut self) -> GraphicsResult<()> {
        unsafe { self.instance.as_ref().unwrap().enumerate_physical_devices() }
            .map_err(|e| {
                GraphicsError::VulkanError(format!("Failed to enumerate physical devices: {:?}", e))
            })?
            .into_iter()
            .find(|&device| {
                self.is_device_suitable(
                    device,
                    self.surface.unwrap(),
                    self.surface_loader.as_ref().unwrap(),
                )
            })
            .ok_or_else(|| GraphicsError::VulkanError("Failed to find a suitable GPU".to_string()))
            .map(|device| {
                self.physical_device = Some(device);
                println!("Selected physical device: {:?}", device);
            })
    }

    fn init_logical_device(&mut self) -> GraphicsResult<()> {
        // retrieve queue family indices for graphics and presentation
        let queue_family_index = self.find_queue_family_indice(
            self.physical_device.unwrap(),
            self.surface.unwrap(),
            self.surface_loader.as_ref().unwrap(),
        );
        queue_family_index
            .is_complete()
            .then(|| ())
            .ok_or_else(|| {
                GraphicsError::VulkanError("Failed to find suitable queue families".to_string())
            })?;
        let unique_queue_families = HashSet::from([
            queue_family_index.graphics_family.unwrap(),
            queue_family_index.present_family.unwrap(),
        ]);
        let device_queue_create_info = Vec::from_iter(unique_queue_families.iter().map(|&index| {
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(index)
                .queue_priorities(&[1.0])
        }));
        let device_extensions = Self::get_device_extensions();
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&device_queue_create_info)
            .enabled_extension_names(&device_extensions);
        self.logical_device = Some(unsafe {
            self.instance
                .as_ref()
                .unwrap()
                .create_device(self.physical_device.unwrap(), &create_info, None)
                .map_err(|e| {
                    GraphicsError::VulkanError(
                        format!("Failed to create logical device, error code: {:?}", e).to_string(),
                    )
                })?
        });
        Ok(())
    }

    fn destroy_logical_device(&mut self) {
        if let Some(device) = &self.logical_device {
            unsafe {
                device.destroy_device(None);
            }
            self.logical_device = None;
            println!("Logical device destroyed");
        }
    }

    fn init_queue(&mut self) -> GraphicsResult<()> {
        let queue_family_index = self.find_queue_family_indice(
            self.physical_device.unwrap(),
            self.surface.unwrap(),
            self.surface_loader.as_ref().unwrap(),
        );
        let indice_set = HashSet::from([
            queue_family_index.graphics_family.unwrap(),
            queue_family_index.present_family.unwrap(),
        ]);
        for &index in &indice_set {
            println!("Queue family index: {}", index);
            let queue = unsafe {
                self.logical_device
                    .as_ref()
                    .unwrap()
                    .get_device_queue(index, 0)
            };
            self.queues.push(queue);
            println!("Graphics queue obtained: {:?}", queue);
        }
        Ok(())
    }

    fn destroy_vulkan(&mut self) {
        self.destroy_logical_device();
        self.destroy_debug_messenger();
        if let Some(instance) = &self.instance {
            unsafe {
                instance.destroy_instance(None);
            }
            self.instance = None;
            println!("Vulkan instance destroyed");
        }
    }
}

impl GraphicsBackend for VulkanGraphics {
    fn can_create_surface<
        W: raw_window_handle::HasWindowHandle,
        D: raw_window_handle::HasDisplayHandle,
    >(
        &mut self,
        window: &super::WindowHandlePara<W, D>,
        width: u32,
        height: u32,
    ) -> GraphicsResult<()> {
        self.init_vulkan(window)?;
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
