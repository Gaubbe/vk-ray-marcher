use std::sync::Arc;
use vulkano::command_buffer::allocator::{
    StandardCommandBufferAllocator,
    StandardCommandBufferAllocatorCreateInfo
};
use vulkano::instance::Instance;
use vulkano::device::{Device, Queue};
use vulkano::memory::allocator::StandardMemoryAllocator;

mod instance;
mod device;
mod pipeline;

pub struct VulkanContext {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue_family_index: u32,
    pub queues: Vec<Arc<Queue>>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: StandardCommandBufferAllocator,
}

impl VulkanContext {
    pub fn new() -> VulkanContext {
        let instance = instance::create_vulkan_instance();

        let (device, queue_family_index, mut queues) = device::create_device(&instance);

        let memory_allocator = Arc::new(
            StandardMemoryAllocator::new_default(device.clone())
        );

        let command_buffer_allocator = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        );

        VulkanContext {
            instance,
            device,
            queue_family_index,
            queues,
            memory_allocator,
            command_buffer_allocator,
        }
    }
}

#[cfg(test)]
mod tests {
    use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
    use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo};
    use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
    use vulkano::sync::{self, GpuFuture};
    use super::VulkanContext;

    #[test]
    fn copy_buffer_to_other() {
        let context = VulkanContext::new();

        let source = Buffer::from_iter(
            context.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST |
                    MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            0..64
        ).expect("Could not create source buffer.");

        let destination = Buffer::from_iter(
            context.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST |
                    MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            (0..64).map(|_| 0)
        ).expect("Could not create destination buffer.");

        let mut builder = AutoCommandBufferBuilder::primary(
            &context.command_buffer_allocator,
            context.queue_family_index,
            CommandBufferUsage::OneTimeSubmit
        ).unwrap();

        builder.copy_buffer(CopyBufferInfo::buffers(
                    source.clone(),
                    destination.clone()
                ))
            .unwrap();

        let command_buffer = builder.build().unwrap();

        let future = sync::now(context.device.clone())
            .then_execute(context.queues[0].clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        let src_content = source.read().unwrap();
        let dst_content = destination.read().unwrap();

        assert_eq!(&*src_content, &*dst_content);
    }
}
