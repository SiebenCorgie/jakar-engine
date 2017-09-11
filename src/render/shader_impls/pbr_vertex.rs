use vulkano_shader_derive;

#[derive(VulkanoShader)]
#[ty = "vertex"]
#[path = "data/test.vs"]
struct Dummy;
