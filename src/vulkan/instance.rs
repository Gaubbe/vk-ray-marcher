use std::sync::Arc;
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo, InstanceExtensions};

pub fn create_vulkan_instance(enabled_extensions: InstanceExtensions) -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("No local Vulkan library found.");
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions,
            ..Default::default()
        }
    ).expect("Could not create Vulkan instance.");
    return instance;
}
