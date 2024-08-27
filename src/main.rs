use core::panic;
use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};


use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};

use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};

use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::{GraphicsPipelineCreateInfo};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::{Validated, VulkanError, VulkanLibrary};
use vulkano::instance::{ Instance, InstanceCreateInfo };
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::sync::{self, GpuFuture};


use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::WindowBuilder;

// The compute shader
mod cs {
    vulkano_shaders::shader!{
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
            layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

            void main() {
                vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
                vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);

                vec2 z = vec2(0.0, 0.0);
                float i;
                for(i = 0.0; i < 1.0; i += 0.005) {
                    z = vec2(
                        z.x * z.x - z.y * z.y + c.x,
                        z.y * z.x + z.x * z.y + c.y
                    );

                    if(length(z) > 4.0) {
                        break;
                    }
                }

                vec4 to_write = vec4(vec3(i), 1.0);
                imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
            }
            "
    }
}

// The vertex shader
mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
            "
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: r"
            #version 460

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
            "
    }
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct VertexData {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2]
}

fn select_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions
) -> (Arc<PhysicalDevice>, u32) {
    instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,

            _ => 4
        })
        .expect("no device available")
}

fn get_render_pass(device: Arc<Device>, swapchain: &Arc<Swapchain>) -> Arc<RenderPass> {
    vulkano::single_pass_renderpass!(
        device,
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap()
}

fn get_framebuffers(
    images: &[Arc<Image>],
    render_pass: &Arc<RenderPass>
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                }).unwrap()
        }).collect::<Vec<_>>()
}

fn get_pipeline(
    device: Arc<Device>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport
) -> Arc<GraphicsPipeline> {
    let vs = vs.entry_point("main").unwrap();
    let fs = fs.entry_point("main").unwrap();

    let vertex_input_state = VertexData::per_vertex()
        .definition(&vs.info().input_interface)
        .unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vs),
        PipelineShaderStageCreateInfo::new(fs)
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap()
    ).unwrap();

    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    GraphicsPipeline::new(
        device.clone(),
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState::default()),
            viewport_state: Some(ViewportState {
                viewports: [viewport].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default()
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
    }).unwrap()
}

fn get_command_buffers(
    command_buffer_allocator: &StandardCommandBufferAllocator,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>,
    vertex_buffer: &Subbuffer<[VertexData]>
) -> Vec<Arc<PrimaryAutoCommandBuffer>> {
    framebuffers
        .iter()
        .map(|framebuffer| {
            let mut builder = AutoCommandBufferBuilder::primary(
                command_buffer_allocator,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit
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
                    }
                )
                .unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .unwrap()
                .draw(vertex_buffer.len() as u32, 1, 0, 0)
                .unwrap()
                .end_render_pass(SubpassEndInfo::default())
                .unwrap();

            builder.build().unwrap()
        }).collect()
}

fn main() {
    // Create event loop
    let event_loop = EventLoop::new();

    // Setup library and Vulkan instance
    let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
    let required_extensions = Surface::required_extensions(&event_loop);
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        })
        .expect("failed to create instance");

    // Setup window
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    // Find physical device and queue index
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (physical_device, queue_family_index) = select_physical_device(
        &instance,
        &surface,
        &device_extensions
    );

    // Create the logical device
    let (device, mut queues) = Device::new(
        physical_device.clone(),
        DeviceCreateInfo {
            queue_create_infos : vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions,
            ..Default::default()
        })
        .expect("failed to create device");

    let queue = queues.next().unwrap();

    // Setup the swapchain
    let caps = physical_device
        .surface_capabilities(&surface, Default::default())
        .expect("failed to get surface capabilities");

    let dimensions = window.inner_size();
    let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
    let image_format = physical_device
        .surface_formats(&surface, Default::default())
        .unwrap()[0]
        .0;

    let (mut swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1,
            image_format,
            image_extent: dimensions.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT,
            composite_alpha,
            ..Default::default()
        }).unwrap();

    // Creating a renderpass
    let render_pass = get_render_pass(device.clone(), &swapchain);

    // Creating the vertex buffer and framebuffers
    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let vertex_data = vec![
        VertexData { position: [-0.5, -0.5] },
        VertexData { position: [ 0.0,  0.5] },
        VertexData { position: [ 0.5, -0.25] }
    ];

    let vertex_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vertex_data)
        .expect("failed to create vertex buffer");

    let framebuffers = get_framebuffers(&images, &render_pass);

    // Loading the shaders
    let vs = vs::load(device.clone()).expect("failed to create shader module");
    let fs = fs::load(device.clone()).expect("failed to create shader module");

    // Creating the graphics pipeline
    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: window.inner_size().into(),
        depth_range: 0.0..=1.0
    };

    let pipeline = get_pipeline(
        device.clone(),
        vs.clone(),
        fs.clone(),
        render_pass.clone(),
        viewport.clone()
    );

    let command_buffer_allocator = StandardCommandBufferAllocator::new(
        device.clone(),
        Default::default());

    let mut command_buffers = get_command_buffers(
        &command_buffer_allocator,
        &queue,
        &pipeline,
        &framebuffers,
        &vertex_buffer
    );

    let mut window_resized = false;
    let mut recreate_swapchain = false;

    let _run = event_loop.run(move |event, _, control_flow| match event {
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

                let new_dimensions = window.inner_size();

                let (new_swapchain, new_images) = swapchain
                    .recreate(SwapchainCreateInfo {
                        image_extent: new_dimensions.into(),
                        ..swapchain.create_info()
                    })
                    .expect("failed to recreate swapchain: {e}");

                swapchain = new_swapchain;
                let new_framebuffers = get_framebuffers(&new_images, &render_pass);

                if window_resized {
                    window_resized = false;

                    viewport.extent = new_dimensions.into();
                    let new_pipeline = get_pipeline(
                        device.clone(),
                        vs.clone(),
                        fs.clone(),
                        render_pass.clone(),
                        viewport.clone()
                    );

                    command_buffers = get_command_buffers(
                        &command_buffer_allocator,
                        &queue,
                        &new_pipeline,
                        &new_framebuffers,
                        &vertex_buffer
                    );
                }
            }

            let (image_i, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(swapchain.clone(), None)
                    .map_err(Validated::unwrap)
                {
                    Ok(r) => r,
                    Err(VulkanError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("failed to acquire next image: {e}")
                };

            if suboptimal {
                recreate_swapchain = true;
            }

            let execution = sync::now(device.clone())
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffers[image_i as usize].clone())
                .unwrap()
                .then_swapchain_present(
                    queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i)
                )
                .then_signal_fence_and_flush();

            match execution.map_err(Validated::unwrap) {
                Ok(future) => {
                    future.wait(None).unwrap();
                }
                Err(e) => {
                    println!("failed to flush future: {e}");
                }
            }
        }
        _ => ()
    });
}

