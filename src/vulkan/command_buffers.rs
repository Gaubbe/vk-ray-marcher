use std::sync::Arc;

use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::device::Queue;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout};
use vulkano::render_pass::Framebuffer;

use super::shaders;
use super::vertex::Vertex;

pub fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    pipeline_layout: &Arc<PipelineLayout>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: &Subbuffer<[Vertex]>,
    push_constants: shaders::fs::constants,
) -> Vec<Arc<PrimaryAutoCommandBuffer>>{
    framebuffers
        .iter()
        .map(move |framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit,
            ).unwrap();

            builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([0.1, 0.1, 0.1, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                    },
                    SubpassBeginInfo {
                        contents: SubpassContents::Inline,
                        ..Default::default()
                    },
                ).unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .unwrap()
                .push_constants(pipeline_layout.clone(), 0, push_constants)
                .unwrap()
                .draw(vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass(SubpassEndInfo::default())
                .unwrap();

            builder.build().unwrap()
        })
        .collect()
}
