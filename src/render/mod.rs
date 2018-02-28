
///The main renderer responsible for the coordination of all render work in its own render loop
pub mod renderer;
///Contains some helper functions and types for the main renderer
pub mod render_helper;
///Creates a builder struct to configure the renderer. And creates a renderer object from it.
pub mod render_builder;
///A primitve which describes one frame. It handles the creation of the frame buffer images
/// needed to preform this frame, returns a commnad buffer which can be filled and finaly
/// executes the command buffer and returns the future of it.
pub mod frame_system;

///Manages all available pipeline, you'll mostly just need the default one
pub mod pipeline_manager;
///Defines the pipeline an renderable object can have, must be stored in the pipeline_manager
pub mod pipeline;
///Describes some comfort types to create a pipeline
pub mod pipeline_builder;
///Handles a window which was created for the renderer
pub mod window;
///manages all universal accesible uniforms, like lights and world info
pub mod uniform_manager;

///Provides some structs and methodes for the postprogressing of a frame
pub mod post_progress;

///Handles buffer creation and drawing of the pre depth rendering. It will calculate needed lights
/// in a Tiled manor later TODO: Implement.
pub mod light_culling_system;

///An module which collects all the shader implementations, these are usually derived from
///vulkano-shader-derive
pub mod shader;

///This modules defines how a shader set and its inputs work. The engine can create pipelines based on the
/// shader sets and creates the descriptorsets on runtime based on the input definitions defined in the shader set.
pub mod shader_manager;

///Defines the usable render passes for this engine
pub mod render_passes;


// A Helper enum to specify which supass id something want, manly used in material creation
pub enum SubPassType {
    LightCompute,
    Forward,
    HdrSorting,
    Blur,
    PostProgress,
    Finished
}

impl SubPassType{
    pub fn get_id(&self) -> u32{
        match self{
            //ComputePass
            &SubPassType::LightCompute => { //is in its own compute queue
                0
            },
            //MainPass
            &SubPassType::Forward =>{ //the first pass in the rendering
                0
            },
            &SubPassType::HdrSorting => { //the second
                1
            },
            //Blur pass
            &SubPassType::Blur => { //the second
                0
            },
            //AssemblePass
            &SubPassType::PostProgress => { //the second
                0
            },
            &SubPassType::Finished => { //no actual renderpass atm, alter maybe
                1
            }
        }
    }
}
