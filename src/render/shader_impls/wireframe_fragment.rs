use vulkano_shader_derive;


#[derive(VulkanoShader)]
#[ty = "fragment"]
#[path = "data/shader/wireframe_simple.fs"]
struct Dummy;
