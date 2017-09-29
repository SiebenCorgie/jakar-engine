use vulkano;
use std::sync::Arc;

use render;

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

///Defines all possible shader constructs which are supported by the engine
#[derive(Copy, Clone)]
pub enum ShaderSetTypes {
    VertFrag,
    VertFragGeo,
    VertFragGeoTess,
    VertFragTess,
}

///Holds a list of all available shader types which can be loaded
pub enum JakarShaders {
    ///Defines the default opaque shader (3 dummys)
    PbrOpaque(
        (
            pbr_vertex::Shader,
            pbr_fragment::Shader,
            render::pipeline::PipelineInput,
            ShaderSetTypes
        )
    ),
    ///Defines the default Wireframe shader (3 dummys)
    Wireframe(
        (
            wireframe_vertex::Shader,
            wireframe_fragment::Shader,
            render::pipeline::PipelineInput,
            ShaderSetTypes
        )
    ),
}

///A list of loadable shader types.
///
///NOTE: This type is only used for the configuration in the `PipelineConfiguration` struct.
/// The final shader and its properties will be stored in an `JakarShaders` enum.
pub enum ShaderTypes {
    PbrOpaque,
    Wireframe,
}

///Loads an shader from specified type and returns the shaders as well as an `PipelineInputs` struct
/// to define the needed Inputs and the `ShaderSetTypes` for the pipeline creation.
pub fn load_shader(device: Arc<vulkano::device::Device>, shader_type: ShaderTypes) ->
    JakarShaders
{
    match shader_type{
        PbrOpaque => {
            //load shader
            let vs = pbr_vertex::Shader::load(device.clone()).expect("failed to load vertex pbr shader");
            let fs = pbr_fragment::Shader::load(device).expect("failed to load fragment pbr shader");

            //Create needed inputs
            let inputs = render::pipeline::PipelineInput::new_all();
            //Sets which sets are used for the shader
            let sets = ShaderSetTypes::VertFrag;
            //now return them
            JakarShaders::PbrOpaque((vs, fs, inputs, sets))
        }
        Wireframe => {
            let vs = wireframe_vertex::Shader::load(device.clone()).expect("failed to load vertex pbr shader");
            let fs = wireframe_fragment::Shader::load(device).expect("failed to load fragment pbr shader");
            let inputs = render::pipeline::PipelineInput::with_none();
            let sets = ShaderSetTypes::VertFrag;

            JakarShaders::Wireframe((vs, fs, inputs, sets))
        }
    }
}
