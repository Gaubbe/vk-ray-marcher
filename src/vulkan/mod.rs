use std::sync::Arc;
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator,
    StandardCommandBufferAllocatorCreateInfo
};
use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::Instance;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{Surface, Swapchain};
use winit::event_loop::EventLoop;
use winit::window::Window;

use self::vertex::Vertex;

mod instance;
mod device;
mod swapchain;
mod render_pass;
mod vertex;
mod shaders;
mod pipeline;

pub struct VulkanContext {
    pub instance: Arc<Instance>,
    pub surface: Arc<Surface>,
    pub device_extensions: DeviceExtensions,
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub queue_family_index: u32,
    pub queues: Vec<Arc<Queue>>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
}

impl VulkanContext {
    pub fn new(event_loop: &EventLoop<()>, window: &Arc<Window>) -> VulkanContext {
        let required_extensions = Surface::required_extensions(event_loop);

        let instance = instance::create_vulkan_instance(required_extensions);

        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("Could not create surface.");

        let (
            device_extensions,
            physical_device,
            device,
            queue_family_index,
            mut queues,
        ) = device::create_device(&instance, &surface);

        let (swapchain, images) = swapchain::get_swapchain(
            &physical_device,
            &device,
            &surface,
            window.inner_size().into(),
        );

        let render_pass = render_pass::get_render_pass(&device, &swapchain);

        let framebuffers = swapchain::get_framebuffers(
            images.as_slice(),
            &render_pass,
        );

        let memory_allocator = Arc::new(
            StandardMemoryAllocator::new_default(device.clone())
        );

        let vertex_buffer = vertex::create_vertex_buffer(&memory_allocator);

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let vs = shaders::vs::load(device.clone())
            .expect("Could not load vertex shader.");
        let fs = shaders::fs::load(device.clone())
            .expect("Could not load fragment shader.");

        let pipeline = pipeline::get_pipeline::<Vertex>(
            &device,
            &swapchain,
            &vs,
            &fs,
            viewport,
            &render_pass,
        );

        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        );

        VulkanContext {
            instance,
            surface,
            device_extensions,
            physical_device,
            device,
            queue_family_index,
            queues,
            memory_allocator,
            swapchain,
            render_pass,
            framebuffers,
            pipeline,
            command_buffer_allocator,
        }
    }
}
