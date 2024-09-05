use std::sync::Arc;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator,
    StandardCommandBufferAllocatorCreateInfo
};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::Instance;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
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
mod command_buffers;

pub struct VulkanContext {
    pub instance: Arc<Instance>,
    pub surface: Arc<Surface>,
    pub device_extensions: DeviceExtensions,
    pub physical_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub queue_family_index: u32,
    pub queue: Arc<Queue>,
    pub swapchain: Arc<Swapchain>,
    pub render_pass: Arc<RenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub vertex_buffer: Subbuffer<[Vertex]>,
    pub viewport: Viewport,
    pub vs: Arc<ShaderModule>,
    pub fs: Arc<ShaderModule>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
    pub command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
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

        let queue = queues.into_iter().next().unwrap();

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
            viewport.clone(),
            &render_pass,
        );

        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        );

        let command_buffers = command_buffers::get_command_buffers(
            &command_buffer_allocator,
            &queue,
            &pipeline,
            &framebuffers,
            &vertex_buffer,
        );

        VulkanContext {
            instance,
            surface,
            device_extensions,
            physical_device,
            device,
            queue_family_index,
            queue,
            swapchain,
            render_pass,
            framebuffers,
            memory_allocator,
            vertex_buffer,
            viewport,
            vs,
            fs,
            pipeline,
            command_buffer_allocator,
            command_buffers,
        }
    }

    pub fn recreate_swapchain(
        &mut self,
        window: &Arc<Window>,
        window_resized: bool,
    ) {
        let new_dimensions = window.inner_size();

        let (new_swapchain, new_images) = self.swapchain
            .recreate(
                SwapchainCreateInfo {
                    image_extent: new_dimensions.into(),
                    ..self.swapchain.create_info()
                }
            ).expect("Failed to recreate swapchain: {e}");
        self.swapchain = new_swapchain;
        let new_framebuffers = swapchain::get_framebuffers(
            &new_images,
            &self.render_pass,
        );

        if window_resized {
            self.viewport.extent = new_dimensions.into();
            let new_pipeline = pipeline::get_pipeline::<Vertex>(
                &self.device,
                &self.swapchain,
                &self.vs,
                &self.fs,
                self.viewport.clone(),
                &self.render_pass,
            );
            self.pipeline = new_pipeline;
            self.command_buffers = command_buffers::get_command_buffers(
                &self.command_buffer_allocator,
                &self.queue,
                &self.pipeline,
                &new_framebuffers,
                &self.vertex_buffer,
            );
        }
    }
}
