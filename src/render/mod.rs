
///The main renderer responsible for the coordination of all render work in its own render loop
pub mod renderer;
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

///An module which collects all the shader implementations, these are usually derived from
///vulkano-shader-derive
pub mod shader_impls;
///Contains some helper functions and types for the main renderer
pub mod render_helper;
