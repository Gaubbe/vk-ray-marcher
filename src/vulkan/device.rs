use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::device::physical::PhysicalDevice;

pub fn create_device(instance: &Arc<Instance>) -> (Arc<Device>, u32, Vec<Arc<Queue>>) {
    let physical_device = choose_physical_device(instance);

    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_i, props)| {
            props.queue_flags.contains(QueueFlags::GRAPHICS)
        })
        .expect("Couldn't find a graphical queue family") as u32;

    let (device, queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        })
        .expect("Could not create Vulkan logical device.");

    return (device, queue_family_index, queues.collect());
}

fn choose_physical_device(instance: &Arc<Instance>) -> Arc<PhysicalDevice> {
    let physical_device = instance
        .enumerate_physical_devices()
        .expect("Could not enumerate Vulkan physical devices.")
        .next()
        .expect("No Vulkan physical devices available");

    return physical_device;
}
