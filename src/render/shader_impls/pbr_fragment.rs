use vulkano_shader_derive;


#[derive(VulkanoShader)]
#[ty = "fragment"]
#[path = "data/shader/pbr_opaque.fs"]
struct Dummy;
