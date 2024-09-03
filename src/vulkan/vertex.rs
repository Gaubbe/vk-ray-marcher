use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::vertex_input;

#[derive(BufferContents, vertex_input::Vertex)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2]
}

pub fn create_vertex_buffer(memory_allocator: &Arc<StandardMemoryAllocator>) -> Subbuffer<[Vertex]> {
    Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vec![
            Vertex { position: [-0.5, -0.5] },
            Vertex { position: [ 0.0,  0.5] },
            Vertex { position: [ 0.5, -0.25] },
        ]
        ).expect("Could not create the vertex buffer.")
}
