use vulkano::VulkanLibrary;
use vulkano::instance::{ Instance, InstanceCreateInfo };

fn main() {
    // Setup library and Vulkan instance
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let instance = Instance::new(library, InstanceCreateInfo::default())
        .expect("failed to create instance");

    // Find physical device
    let physical_device = instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .next()
        .expect("no devices available");
}
