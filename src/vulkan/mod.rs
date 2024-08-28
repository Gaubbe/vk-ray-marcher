use std::sync::Arc;
use vulkano::instance::Instance;

mod instance;

pub struct VulkanContext {
    pub instance: Arc<Instance>,
}

impl VulkanContext {
    pub fn new() -> VulkanContext {
        let instance = instance::create_vulkan_instance();

        VulkanContext {
            instance
        }
    }
}
