use std::sync::Arc;

use winit::{event_loop::EventLoop, window::WindowBuilder};

mod vulkan;

fn main() {
    let event_loop = EventLoop::new();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let _context = vulkan::VulkanContext::new(&event_loop, &window);
}
