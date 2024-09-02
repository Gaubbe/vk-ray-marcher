use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::device::{
    Device,
    DeviceCreateInfo,
    Queue,
    QueueCreateInfo,
    QueueFlags,
    DeviceExtensions,
};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::swapchain::Surface;

pub fn create_device(instance: &Arc<Instance>, surface: &Arc<Surface>) -> (
    DeviceExtensions,
    Arc<Device>,
    u32,
    Vec<Arc<Queue>>,
) {
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..Default::default()
    };

    let (physical_device, queue_family_index) = choose_physical_device(
        instance,
        surface,
        &device_extensions,
    );

    let (device, queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions,
            ..Default::default()
        })
        .expect("Could not create Vulkan logical device.");

    return (device_extensions, device, queue_family_index, queues.collect());
}

fn choose_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
    let physical_device = instance
        .enumerate_physical_devices()
        .expect("Could not enumerate Vulkan physical devices.")
        .filter(|p| p.supported_extensions().contains(device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, surface).unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("No suitable device available.");

    return physical_device;
}
