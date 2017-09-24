use vulkano_shader_derive;

#[derive(VulkanoShader)]
#[ty = "vertex"]
#[path = "data/shader/pbr_opaque.vs"]
struct Dummy;
