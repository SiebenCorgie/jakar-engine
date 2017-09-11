use vulkano_shader_derive;


#[derive(VulkanoShader)]
#[ty = "fragment"]
#[path = "data/test.fs"]
struct Dummy;
