use core::panic;
use std::{sync::Arc, usize};

use vulkano::{swapchain::{self, SwapchainPresentInfo}, sync::{self, future::FenceSignalFuture, GpuFuture}, Validated, VulkanError};
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

mod vulkan;

fn main() {
    let event_loop = EventLoop::new();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let mut context = vulkan::VulkanContext::new(&event_loop, &window);

    let mut window_resized = false;
    let mut recreate_swapchain = false;

    let frames_in_flight = context.framebuffers.len();
    let mut fences : Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            window_resized = true;
        }
        Event::MainEventsCleared => {
            if window_resized || recreate_swapchain {
                recreate_swapchain = false;
                context.recreate_swapchain(&window, window_resized);
                window_resized = false;
            }

            let (image_i, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(
                    context.swapchain.clone(),
                    None
                ).map_err(Validated::unwrap) {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image"),
                };

            if suboptimal {
                recreate_swapchain = true;
            }

            if let Some(image_fence) = &fences[image_i as usize] {
                image_fence.wait(None).unwrap();
            }

            let previous_future = match fences[previous_fence_i as usize].clone() {
                None => {
                    let mut now = sync::now(context.device.clone());
                    now.cleanup_finished();

                    now.boxed()
                }
                Some(fence) => fence.boxed(),
            };

            let future = previous_future
                .join(acquire_future)
                .then_execute(
                    context.queue.clone(),
                    context.command_buffers[image_i as usize].clone(),
                )
                .unwrap()
                .then_swapchain_present(
                    context.queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(
                        context.swapchain.clone(),
                        image_i,
                    )
                )
                .then_signal_fence_and_flush();

            fences[image_i as usize] = match future.map_err(Validated::unwrap) {
                Ok(value) => Some(Arc::new(value)),
                Err(VulkanError::OutOfDate) => {
                    recreate_swapchain = true;
                    None
                }
                Err(e) => {
                    println!("Failed to flush future: {e}");
                    None
                }
            };
        }
        _ => (),
    });
}
