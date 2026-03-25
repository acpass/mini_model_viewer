use std::{
    ffi::{CStr, c_char},
    ptr::null,
};

use ash::{
    Entry,
    ext::debug_utils,
    fuchsia::external_memory,
    vk::{self, DebugUtilsMessengerEXT, PFN_vkCreateDebugUtilsMessengerEXT},
};

impl super::VulkanGraphics {
    pub fn check_validation_layer_support(entry: &ash::Entry, layer: &CStr) -> bool {
        let layers = unsafe {
            entry
                .enumerate_instance_layer_properties()
                .expect("Failed to enumerate instance layers")
        };
        layers.iter().any(|layer_properties| {
            let layer_name = unsafe { CStr::from_ptr(layer_properties.layer_name.as_ptr()) };
            layer_name == layer
        })
    }

    pub fn get_validation_layer_names() -> Vec<*const c_char> {
        if std::env::var("PROFILE").unwrap_or_default() == "debug" {
            Vec::new()
        } else {
            vec![c"VK_LAYER_KHRONOS_validation".as_ptr()]
        }
    }

    unsafe extern "system" fn vulkan_debug_messenger_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_types: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut std::ffi::c_void,
    ) -> vk::Bool32 {
        let callback_data = unsafe { *p_callback_data };
        let message_id_number = callback_data.message_id_number;
        let message_id_name = unsafe {
            if callback_data.p_message_id_name.is_null() {
                CStr::from_bytes_with_nul(b"<no message id name>\0").unwrap()
            } else {
                CStr::from_ptr(callback_data.p_message_id_name)
            }
        };
        let message = unsafe {
            if callback_data.p_message.is_null() {
                CStr::from_bytes_with_nul(b"<no message>\0").unwrap()
            } else {
                CStr::from_ptr(callback_data.p_message)
            }
        };
        println!(
            "{:?}:\n{:?} [{} ({})] : {}\n",
            message_severity,
            message_types,
            message_id_name.to_str().unwrap(),
            message_id_number,
            message.to_str().unwrap()
        );
        vk::FALSE
    }

    pub fn setup_debug_messenger(&mut self) -> Option<DebugUtilsMessengerEXT> {
        let entry = Entry::linked();
        self.debug_util = Some(debug_utils::Instance::new(
            &entry,
            &self.instance.as_ref().unwrap(),
        ));
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(Self::vulkan_debug_messenger_callback));
        Some(unsafe {
            self.debug_util
                .as_ref()
                .unwrap()
                .create_debug_utils_messenger(&create_info, None)
                .unwrap()
        })
    }

    pub fn destroy_debug_messenger(&mut self) {
        if let Some(debug_messenger) = self.debug_messenger {
            unsafe {
                self.debug_util
                    .as_ref()
                    .unwrap()
                    .destroy_debug_utils_messenger(debug_messenger, None);
            }
        }
    }
}
