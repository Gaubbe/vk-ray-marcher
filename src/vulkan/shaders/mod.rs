pub mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "src/vulkan/shaders/ray-marcher-vert.glsl",
    }
}

pub mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "src/vulkan/shaders/ray-marcher-frag.glsl",
    }
}
