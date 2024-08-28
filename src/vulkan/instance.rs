use std::sync::Arc;
use vulkano::VulkanLibrary;
use vulkano::instance::{Instance, InstanceCreateInfo};

pub fn create_vulkan_instance() -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("No local Vulkan library found.");
    let instance = Instance::new(library, InstanceCreateInfo::default())
        .expect("Could not create Vulkan instance.");
    return instance;
}
