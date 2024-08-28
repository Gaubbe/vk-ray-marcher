use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::device::{Device, Queue};

mod instance;
mod device;

pub struct VulkanContext {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue_family_index: u32,
    pub queues: Vec<Arc<Queue>>,
}

impl VulkanContext {
    pub fn new() -> VulkanContext {
        let instance = instance::create_vulkan_instance();

        let (device, queue_family_index, mut queues) = device::create_device(&instance);

        VulkanContext {
            instance,
            device,
            queue_family_index,
            queues,
        }
    }
}
