mod debug;
use ash::vk;
use winit::raw_window_handle::{self, HasDisplayHandle};

use super::GraphicsBackend;
use crate::graphics::{GraphicsError, GraphicsResult};
use std::{collections::HashSet, ffi::CStr, io, path::Path};

impl From<vk::Result> for GraphicsError {
    fn from(result: vk::Result) -> Self {
        GraphicsError::VulkanError(format!("Vulkan error: {:?}", result))
    }
}

impl From<io::Error> for GraphicsError {
    fn from(error: std::io::Error) -> Self {
        GraphicsError::VulkanError(format!("IO error: {:?}", error))
    }
}

struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

struct SwapChainSupportDetails {
    #[allow(dead_code)]
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

#[derive(Default)]
pub struct VulkanGraphics {
    entry: Option<ash::Entry>,
    instance: Option<ash::Instance>,
    surface_loader: Option<ash::khr::surface::Instance>,
    swapchain_loader: Option<ash::khr::swapchain::Device>,

    physical_device: Option<vk::PhysicalDevice>,
    logical_device: Option<ash::Device>,
    surface: Option<vk::SurfaceKHR>,
    queues: Vec<vk::Queue>,

    swap_chain: Option<vk::SwapchainKHR>,
    images: Vec<vk::Image>,
    swap_chain_format: Option<vk::SurfaceFormatKHR>,
    swap_chain_present_mode: Option<vk::PresentModeKHR>,
    swap_chain_extent: Option<vk::Extent2D>,

    image_views: Vec<vk::ImageView>,
    render_pass: Option<vk::RenderPass>,

    debug_util: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<ash::vk::DebugUtilsMessengerEXT>,

    shader_path: Option<String>,
}

impl VulkanGraphics {
    pub fn set_shader_path(mut self, path: &Path) -> Self {
        println!("Setting shader path to: {}", path.display());
        self.shader_path = Some(path.to_string_lossy().to_string());
        self
    }

    fn init_vulkan<
        W: raw_window_handle::HasWindowHandle,
        D: raw_window_handle::HasDisplayHandle,
    >(
        &mut self,
        handle: &super::WindowHandlePara<W, D>,
        width: u32,
        height: u32,
    ) -> GraphicsResult<()> {
        self.entry = Some(unsafe { ash::Entry::load().expect("No vulkan support") });
        self.init_instance(handle)?;
        self.setup_debug_messenger();
        self.init_surface(handle)?;
        self.pick_physical_device()?;
        self.init_logical_device()?;
        self.init_queue()?;
        self.create_swap_chain(width, height)?;
        self.create_image_views()?;
        self.create_render_pass()?;
        self.create_pipeline()?;
        Ok(())
    }

    fn init_surface<
        W: raw_window_handle::HasWindowHandle,
        D: raw_window_handle::HasDisplayHandle,
    >(
        &mut self,
        handle: &super::WindowHandlePara<W, D>,
    ) -> GraphicsResult<()> {
        self.surface = Some(unsafe {
            ash_window::create_surface(
                self.entry.as_ref().unwrap(),
                self.instance.as_ref().unwrap(),
                handle.display.display_handle().unwrap().as_raw(),
                handle.window.window_handle().unwrap().as_raw(),
                None,
            )
        }?);
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

    fn check_device_extension_support(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        required_extensions: &[&CStr],
    ) -> bool {
        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(device)
                .expect("Failed to enumerate device extension properties")
        };
        let available_extension_names: HashSet<&CStr> = available_extensions
            .iter()
            .map(|ext| unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) })
            .collect();
        required_extensions
            .iter()
            .all(|&req_ext| available_extension_names.contains(req_ext))
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
            flags |= ash::vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
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
        QueueFamilyIndices {
            graphics_family,
            present_family,
        }
    }

    fn get_swap_chain_details(
        &self,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &ash::khr::surface::Instance,
    ) -> GraphicsResult<SwapChainSupportDetails> {
        let capabilities =
            unsafe { surface_loader.get_physical_device_surface_capabilities(device, surface)? };

        let formats =
            unsafe { surface_loader.get_physical_device_surface_formats(device, surface)? };

        let present_modes =
            unsafe { surface_loader.get_physical_device_surface_present_modes(device, surface)? };
        Ok(SwapChainSupportDetails {
            capabilities,
            formats,
            present_modes,
        })
    }

    fn is_device_suitable(
        &self,
        device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        surface_loader: &ash::khr::surface::Instance,
    ) -> bool {
        let queue_family_complete = self
            .find_queue_family_indice(device, surface, surface_loader)
            .is_complete();
        let extensions_supported = Self::check_device_extension_support(
            self.instance.as_ref().unwrap(),
            device,
            Self::get_device_extensions()
                .iter()
                .map(|&ext| unsafe { CStr::from_ptr(ext) })
                .collect::<Vec<_>>()
                .as_slice(),
        );
        let swap_chain_adequate = if extensions_supported {
            match self.get_swap_chain_details(device, surface, surface_loader) {
                Ok(details) => !details.formats.is_empty() && !details.present_modes.is_empty(),
                Err(_) => false,
            }
        } else {
            false
        };

        queue_family_complete && extensions_supported && swap_chain_adequate
    }

    fn pick_physical_device(&mut self) -> GraphicsResult<()> {
        unsafe {
            self.instance
                .as_ref()
                .unwrap()
                .enumerate_physical_devices()?
        }
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
            .then_some(())
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
            self.instance.as_ref().unwrap().create_device(
                self.physical_device.unwrap(),
                &create_info,
                None,
            )?
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

    fn choose_swap_chain_format(
        available_formats: &[vk::SurfaceFormatKHR],
    ) -> vk::SurfaceFormatKHR {
        available_formats
            .iter()
            .cloned()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_SRGB
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| available_formats[0])
    }

    fn choose_swap_chain_present_mode(
        available_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        available_present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    fn choose_swap_chain_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        width: u32,
        height: u32,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D {
                width: width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    fn create_swap_chain(&mut self, width: u32, height: u32) -> GraphicsResult<()> {
        let swap_chain_support = self.get_swap_chain_details(
            self.physical_device.unwrap(),
            self.surface.unwrap(),
            self.surface_loader.as_ref().unwrap(),
        )?;
        let surface_format = Self::choose_swap_chain_format(&swap_chain_support.formats);
        let present_mode = Self::choose_swap_chain_present_mode(&swap_chain_support.present_modes);
        let extent =
            Self::choose_swap_chain_extent(&swap_chain_support.capabilities, width, height);

        let image_count = swap_chain_support.capabilities.min_image_count + 1;
        let image_count = if swap_chain_support.capabilities.max_image_count > 0 {
            image_count.min(swap_chain_support.capabilities.max_image_count)
        } else {
            image_count
        };

        let mut create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface.unwrap())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(swap_chain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let queue_family_index = self.find_queue_family_indice(
            self.physical_device.unwrap(),
            self.surface.unwrap(),
            self.surface_loader.as_ref().unwrap(),
        );

        let queue_family_vec: Vec<u32> = HashSet::from([
            queue_family_index.graphics_family.unwrap(),
            queue_family_index.present_family.unwrap(),
        ])
        .into_iter()
        .collect();

        if queue_family_vec.len() > 1 {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_vec);
        } else {
            create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        };

        self.swapchain_loader = Some(ash::khr::swapchain::Device::new(
            self.instance.as_ref().unwrap(),
            self.logical_device.as_ref().unwrap(),
        ));

        self.swap_chain = unsafe {
            self.swapchain_loader
                .as_ref()
                .unwrap()
                .create_swapchain(&create_info, None)?
        }
        .into();

        self.images = unsafe {
            self.swapchain_loader
                .as_ref()
                .unwrap()
                .get_swapchain_images(self.swap_chain.unwrap())?
        };

        self.swap_chain_format = Some(surface_format);
        self.swap_chain_present_mode = Some(present_mode);
        self.swap_chain_extent = Some(extent);
        Ok(())
    }

    fn destroy_swap_chain(&mut self) {
        self.swap_chain_format = None;
        self.swap_chain_present_mode = None;
        self.swap_chain_extent = None;
        self.images.clear();
        if let Some(swap_chain) = self.swap_chain {
            unsafe {
                self.swapchain_loader
                    .as_ref()
                    .unwrap()
                    .destroy_swapchain(swap_chain, None);
            }
            self.swap_chain = None;
        }
    }

    fn create_image_views(&mut self) -> GraphicsResult<()> {
        // Placeholder for image view creation logic
        for &image in &self.images {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(self.swap_chain_format.unwrap().format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            let image_view = unsafe {
                self.logical_device
                    .as_ref()
                    .unwrap()
                    .create_image_view(&create_info, None)?
            };
            self.image_views.push(image_view);
        }
        Ok(())
    }

    fn read_shader_code(path: &str) -> GraphicsResult<Vec<u8>> {
        Ok(std::fs::read(path)?)
    }

    fn create_shader_module(device: &ash::Device, code: &[u8]) -> GraphicsResult<vk::ShaderModule> {
        let create_info = vk::ShaderModuleCreateInfo::default().code(unsafe {
            std::slice::from_raw_parts(
                code.as_ptr() as *const u32,
                code.len() / std::mem::size_of::<u32>(),
            )
        });
        Ok(unsafe { device.create_shader_module(&create_info, None)? })
    }

    fn create_render_pass(&mut self) -> GraphicsResult<()> {
        let color_attachment = [vk::AttachmentDescription::default()
            .format(self.swap_chain_format.unwrap().format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)];

        let color_attachment_ref = [vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
        let subpass = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_ref)];

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(&color_attachment)
            .subpasses(&subpass);

        self.render_pass = Some(unsafe {
            self.logical_device
                .as_ref()
                .unwrap()
                .create_render_pass(&render_pass_info, None)?
        });

        Ok(())
    }

    fn create_pipeline(&mut self) -> GraphicsResult<()> {
        let vert_shader_name = self
            .shader_path
            .as_ref()
            .ok_or_else(|| GraphicsError::VulkanError("Shader path not set".to_string()))?
            .clone()
            + "/main.vert.spv";
        let frag_shader_name = self
            .shader_path
            .as_ref()
            .ok_or_else(|| GraphicsError::VulkanError("Shader path not set".to_string()))?
            .clone()
            + "/main.frag.spv";
        let vertex_shader_module = Self::create_shader_module(
            self.logical_device.as_ref().unwrap(),
            &Self::read_shader_code(&vert_shader_name)?,
        )?;
        let fragment_shader_module = Self::create_shader_module(
            self.logical_device.as_ref().unwrap(),
            &Self::read_shader_code(&frag_shader_name)?,
        )?;
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader_module)
                .name(unsafe { CStr::from_ptr((c"main").as_ptr()) }),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader_module)
                .name(unsafe { CStr::from_ptr((c"main").as_ptr()) }),
        ];

        for stage in shader_stages {
            println!(
                "Shader stage: {:?}, module: {:?}",
                stage.stage, stage.module
            );
        }
        Ok(())
    }

    fn destroy_image_views(&mut self) {
        for &image_view in &self.image_views {
            unsafe {
                self.logical_device
                    .as_ref()
                    .unwrap()
                    .destroy_image_view(image_view, None);
            }
        }
    }

    fn destroy_vulkan(&mut self) {
        self.destroy_image_views();
        self.destroy_swap_chain();
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
        self.init_vulkan(window, width, height)?;
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
