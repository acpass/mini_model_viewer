use std::ffi::{CStr, c_char};

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

    // todo: add debug messenger setup and callback
    // https://vulkan-tutorial.com/Drawing_a_triangle/Setup/Validation_layers#page_Using-validation-layers
}
