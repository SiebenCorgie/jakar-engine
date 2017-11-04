use vulkano;
use std::sync::Arc;

use render::pipeline_builder;



///The normal PBR fragment shader
pub mod pbr_fragment;

///The normal PBR vertex shader
pub mod pbr_vertex;

///A wireframe shader for debuging
pub mod wireframe_vertex;

///A wireframe shader for debuging
pub mod wireframe_fragment;

///Defines the default data struct for an shader containing:
/// - Camera Position (vec3)
/// - Model Transform (mat4)
/// - View Matrix (mat4)
/// - Projection Matrix (mat4)
pub mod default_data;

///Defines light types
pub mod lights;

///Defines the PBR Texture Factors
pub mod pbr_texture_factors;

///Defines the usage flags used to determin where to use textures as color base and where to use the linear factors.
pub mod pbr_texture_usage;

///Defines the texture sets usable in a pbr material
pub mod pbr_texture_sets;


///Holds a list of all available shader types which can be loaded
pub enum JakarShaders {
    ///Defines the default opaque shader
    PbrOpaque(
        (
            pbr_vertex::Shader,
            pbr_fragment::Shader,
            pipeline_builder::PipelineInput,
        )
    ),
    ///Defines the default Wireframe shader
    Wireframe(
        (
            wireframe_vertex::Shader,
            wireframe_fragment::Shader,
            pipeline_builder::PipelineInput,
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
            let inputs = pipeline_builder::PipelineInput::new_all();
            //now return them
            JakarShaders::PbrOpaque((vs, fs, inputs))
        }
        Wireframe => {
            let vs = wireframe_vertex::Shader::load(device.clone()).expect("failed to load vertex pbr shader");
            let fs = wireframe_fragment::Shader::load(device).expect("failed to load fragment pbr shader");
            let inputs = pipeline_builder::PipelineInput::with_none();

            JakarShaders::Wireframe((vs, fs, inputs))
        }
    }
}
