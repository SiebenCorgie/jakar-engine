
///The main renderer responsible for the coordination of all render work in its own render loop
pub mod renderer;
///Contains some helper functions and types for the main renderer
pub mod render_helper;
///Creates a builder struct to configure the renderer. And creates a renderer object from it.
pub mod render_builder;
///Defines the usable render passes for this engine
pub mod render_passes;
///Contains all traits which must be implemented by resources if they want to be handled by the
///Renderer.
pub mod render_traits;

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

///Handles the rendering of everything forward rendering related, also emmits the sorted hdr values
/// of this frame used for bluring.
pub mod forward_system;

///Provides some structs and methodes for the postprogressing of a frame
pub mod post_progress;

///Culls all point and spotlights in world space and updates a device local indice buffer with that information.
pub mod light_system;

///An module which collects all the shader implementations, these are usually derived from
///vulkano-shader-derive
pub mod shader;

///This modules defines how a shader set and its inputs work. The engine can create pipelines based on the
/// shader sets and creates the descriptorsets on runtime based on the input definitions defined in the shader set.
pub mod shader_manager;

///Handels the rendering of all shadow maps
pub mod shadow_system;
