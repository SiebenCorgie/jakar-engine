use vulkano;
use std::sync::Arc;

use render;
use render::pipeline_builder;
use core::resources::mesh::Vertex;
use render::post_progress;


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

///The fragment shader for the pre-depths pass, used in conjunktion the the pbr-vertex shader
///to generate a depth image for the following compute shader
//pub mod pre_depth_fragment;
///The vertex shader stage for the pre-depths pass.
//pub mod pre_depth_vertex;

///Defines the PBR Texture Factors
pub mod pbr_texture_factors;

///Defines the usage flags used to determin where to use textures as color base and where to use the linear factors.
pub mod pbr_texture_usage;

///Defines the texture sets usable in a pbr material
pub mod pbr_texture_sets;

///The default post progress vertex shader
pub mod default_pstprg_vertex;
///The default post progress fragment shader (does all the work)
pub mod default_pstprg_fragment;


///Holds a list of all available shader types which can be loaded
pub enum JakarShaders {
    ///Defines the default opaque shader
    PbrOpaque(
        (
            pbr_vertex::Shader,
            pbr_fragment::Shader,
            pipeline_builder::PipelineInput,
            vulkano::pipeline::vertex::SingleBufferDefinition::<Vertex>
        )
    ),
    ///Defines the default Wireframe shader
    Wireframe(
        (
            wireframe_vertex::Shader,
            wireframe_fragment::Shader,
            pipeline_builder::PipelineInput,
            vulkano::pipeline::vertex::SingleBufferDefinition::<Vertex>
        )
    ),
    PostProgress(
        (
            default_pstprg_vertex::Shader,
            default_pstprg_fragment::Shader,
            vulkano::pipeline::vertex::SingleBufferDefinition::<post_progress::PostProgressVertex>
        )
    )
}

///A list of loadable shader types.
///
///NOTE: This type is only used for the configuration in the `PipelineConfiguration` struct.
/// The final shader and its properties will be stored in an `JakarShaders` enum.
#[derive(PartialEq)]
pub enum ShaderTypes {
    PbrOpaque,
    Wireframe,
    PostProgress
}

impl ShaderTypes{
    ///Returns the subpass this shader set is aimed at. Be aware that a wrong return values
    /// Crashes the renderer at runtime
    pub fn get_subpass_id(&self) -> u32{
        match self{
            &ShaderTypes::PbrOpaque => {
                render::SubPassType::Forward.get_id()
            }
            &ShaderTypes::Wireframe => {
                render::SubPassType::Forward.get_id()
            }
            &ShaderTypes::PostProgress => {
                render::SubPassType::PostProgress.get_id()
            }
        }
    }
}


///Loads an shader from specified type and returns the shaders as well as an `PipelineInputs` struct
/// to define the needed Inputs and the `ShaderSetTypes` for the pipeline creation.
pub fn load_shader(device: Arc<vulkano::device::Device>, shader_type: ShaderTypes) ->
    JakarShaders
{
    match shader_type{
        ShaderTypes::PbrOpaque => {
            println!("Loading PbrOpaque shader ...", );
            //load shader
            let vs = pbr_vertex::Shader::load(device.clone()).expect("failed to load vertex pbr shader");
            let fs = pbr_fragment::Shader::load(device).expect("failed to load fragment pbr shader");

            //Create the vertex buffer definition
            let vbd = vulkano::pipeline::vertex::SingleBufferDefinition::<Vertex>::new();

            //Create needed inputs
            let inputs = pipeline_builder::PipelineInput::new_all();
            //now return them
            JakarShaders::PbrOpaque((vs, fs, inputs, vbd))
        }
        ShaderTypes::Wireframe => {
            println!("Loading Post Wireframe shader ...", );
            let vs = wireframe_vertex::Shader::load(device.clone()).expect("failed to load vertex pbr shader");
            let fs = wireframe_fragment::Shader::load(device).expect("failed to load fragment pbr shader");
            let inputs = pipeline_builder::PipelineInput::with_none();
            //Create the vertex buffer definition
            let vbd = vulkano::pipeline::vertex::SingleBufferDefinition::<Vertex>::new();


            JakarShaders::Wireframe((vs, fs, inputs, vbd))
        }
        ShaderTypes::PostProgress => {
            println!("Loading Post progress shader ...", );
            let vs = default_pstprg_vertex::Shader::load(device.clone()).expect("failed to load vertex pbr shader");
            let fs = default_pstprg_fragment::Shader::load(device).expect("failed to load fragment pbr shader");
            //Create the vertex buffer definition
            let vbd = vulkano::pipeline::vertex::SingleBufferDefinition::<post_progress::PostProgressVertex>::new();
            JakarShaders::PostProgress((vs, fs, vbd))
        }
    }
}
