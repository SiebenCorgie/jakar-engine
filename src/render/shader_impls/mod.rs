use vulkano;
use std::sync::Arc;

///The normal PBR vertex shader
pub mod pbr_vertex;

///The normal PBR fragment shader
pub mod pbr_fragment;

///A wireframe shader for debuging
pub mod wireframe_vertex;

///A wireframe shader for debuging
pub mod wireframe_fragment;


use vulkano::pipeline::shader::GraphicsEntryPointAbstract;

///Defines some properties of an shader which are used at pipeline creation time to
///define the pipeline corectly.
pub struct Shader<V, F, G, TC, TE> {
    ///holds the vertex shader (always needed).
    pub vertex: V,
    ///holds the fragment shader (always needed).
    pub framgent: F,
    ///Can hold a geometry shader if provided
    pub geometry: Option<G>,
    ///Can hold an tesselation control and evaluation shader
    pub tesselation: Option<(TC,TE)>,
}

///Holds a list of all available shader types
pub enum JakarShaders {
    ///Defines the default opaque shader
    PbrOpaque,
    ///Defines the default Wireframe shader
    Wireframe,
}

///Returns the opaque pbr shader
fn get_pbr_opaque_shader(device: Arc<vulkano::device::Device>) -> (pbr_vertex::Shader, pbr_fragment::Shader){
    (
        pbr_vertex::Shader::load(device.clone()).expect("failed to load vertex pbr shader"),
        pbr_fragment::Shader::load(device).expect("failed to load fragment pbr shader")
    )
}

///Returns the wireframe shader set
fn get_wireframe_shader(device: Arc<vulkano::device::Device>) -> (wireframe_vertex::Shader, wireframe_fragment::Shader){
    (
        wireframe_vertex::Shader::load(device.clone()).expect("failed to load vertex pbr shader"),
        wireframe_fragment::Shader::load(device).expect("failed to load fragment pbr shader")
    )
}

/*
pub enum JakarShaders {
    ///Defines the default opaque shader
    PbrOpaque(Shader
        <
        pbr_vertex::Shader,
        pbr_fragment::Shader,
        (), (), ()
        >),
    ///Defines the default Wireframe shader
    Wireframe(Shader
        <
        wireframe_vertex::Shader,
        wireframe_fragment::Shader,
        (), (), ()
        >),
}
*/
/*
pub fn get_shader<V, F, G, TC, TE>(s_type: JakarShaders, device: Arc<vulkano::device::Device>) -> Shader<V, F, G, TC, TE> {
    match s_type{
        JakarShaders::PbrOpaque => {
            Shader{
                ///holds the vertex shader (always needed).
                vertex: pbr_vertex::Shader::load(device).expect("failed to load vertex pbr shader"),
                ///holds the fragment shader (always needed).
                framgent: pbr_fragment::Shader::load(device).expect("failed to load fragment pbr shader"),
                ///Can hold a geometry shader if provided
                geometry: None,
                ///Can hold an tesselation control and evaluation shader
                tesselation: None,
            }
        }
    }
}

TODO Currently no nice way to make this :/ I'll have to investigate

*/
