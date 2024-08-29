use std::sync::Arc;
use vulkano::format::Format;
use vulkano::render_pass::RenderPass;
use vulkano::device::Device;

pub fn get_render_pass(device: &Arc<Device>) -> Arc<RenderPass> {
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: Format::R8G8B8A8_UNORM,
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


