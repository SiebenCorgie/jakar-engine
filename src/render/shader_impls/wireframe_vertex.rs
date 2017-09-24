use vulkano_shader_derive;

#[derive(VulkanoShader)]
#[ty = "vertex"]
#[path = "data/shader/wireframe_simple.vs"]
struct Dummy;
