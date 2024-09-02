use std::sync::Arc;
use vulkano::format::Format;
use vulkano::render_pass::RenderPass;
use vulkano::device::Device;
use vulkano::swapchain::Swapchain;

pub fn get_render_pass(
    device: &Arc<Device>,
    swapchain: &Arc<Swapchain>
) -> Arc<RenderPass> {
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    ).unwrap();

    render_pass
}


