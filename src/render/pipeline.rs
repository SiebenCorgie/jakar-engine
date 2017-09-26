use vulkano;
use vulkano::pipeline;
use vulkano_shaders;
use vulkano::pipeline::shader::GraphicsEntryPointAbstract;

use std::sync::Arc;
use core::resources::mesh;

use render::shader_impls::pbr_vertex;
use render::shader_impls::pbr_fragment;
use render::shader_impls;

///Descibes all possible scissors / viewport behavoir
pub enum ViewportScissorsBehavoir {
    ///When used, the scissors will always cover the whole viewport
    DefinedViewport(pipeline::viewport::Viewport),
    ///Defines both types independently
    DefinedViewportAndScissors((pipeline::viewport::Viewport, pipeline::viewport::Scissor)),
    ///Defines the viewport and let the scissors fixed
    DefineViewportFixedScissors(pipeline::viewport::Viewport),
    ///Defines the viewport each frame (dynamic) and creates a scissors covering the whole viewport
    DynamicViewportScissorsIrrelevant(u32),
    ///Defines the viewport once, but the scissors can be changed each frame
    FixedViewportDynamicScissors(pipeline::viewport::Viewport),
    ///Defines both dynamic, both have to be set dynamic per frame, usually used for resizable views
    ViewportScissorsDynamic(u32),
}

///Describes the cullmode of this pipeline
pub enum CullMode {
    Disabled,
    Front,
    Back,
    FrontAndBack
}

///Describes how polygones are drawn
pub enum PolygoneMode {
    ///Polygones are drawn as filled faces (usually used)
    Fill,
    ///Are drawn as lines with an width defined by the u32. If the width is 0.0, the line width is set
    ///dynamicly at render time.
    Line(u32),
    ///Polygones are drawn as points (at each vertice)
    Point
}


///A struct which can be used to configure the pipeline which can be build via the `new()` functions
///of the `Pipeline` struct
pub struct PipelineConfig {
    ///Should be true if only one single vertex buffer is used
    pub single_vertex_bufffer: bool,

    ///Defines the shader type of the loaded pipeline.
    ///TODO this should be moved in a less static abroach, but I don't know how atm.
    pub shader_type: shader_impls::JakarShaders,

    ///Describes how vertices must be grouped together to form primitives.
    pub topology_type: pipeline::input_assembly::PrimitiveTopology,

    ///Describes the Vieport and scissors behavoir of the new pipeline
    pub viewport_scissors: ViewportScissorsBehavoir,

    ///True if the depth should be clamped between 0.0 and 1.0
    pub has_depth_clamp: bool,

    ///Should be true if the faces are oriented clockwise. The default is counter clockwise
    pub has_faces_clockwise: bool,

    ///Defines the cull mode of this pipeline
    pub cull_mode: CullMode,

    ///Defines how the polygones are drawn
    pub polygone_mode: PolygoneMode,

    //TODO
    //Set depth stencil
    //blend type
    //give renderpass


}


///Describes the input needed for the shaders in this pipeline to work.
///
/// #panics
///
///This could panic if the input is defined wrong, mostly the engine won't build though
pub struct PipelineInput {

    ///Describes the mostly used data of projection and view matrix as well as model transform and camera
    ///position.
    pub data: bool,

    ///True if any of the following textures is send and used from the material description:
    ///
    /// - Albedo
    /// - Normal
    /// - MetallicRoughness
    /// - Ao
    /// - Emissive
    pub has_textures: bool,

    ///Is true if the shader recives light information
    pub has_light: bool,

}

///Definition of a single pipeline together with its creation and deleting behavoir
///
///Besides the pipeline definition of the vulkan struct the jakar-pipeline is also responsible
///for creation of the descriptor sets, needed to render a material based on this pipeline
///The features are described by an `PipelineInput` struct.
pub struct Pipeline {
    ///The main pipeline hold by this struct
    //TODO make this dynamic, or implement a different pipeline struct per type... maybe one graphic, one computing? (<- will do this)
    //TODO change to graphics_pipeline and add a compute_pipeline
    pipeline: Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>,

    inputs: PipelineInput,
}

impl Pipeline{
    ///Creates a pipeline for a shader, TODO make it possible to create a custom pipeline easily
    pub fn new_opaque(
        device: Arc<vulkano::device::Device>,
        renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
        inputs: PipelineInput,
    )
        -> Self
    {
        //Currently using a static shader from /data/test.vs/fs
        let vs = pbr_vertex::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = pbr_fragment::Shader::load(device.clone()).expect("failed to create shader module");

        //Create a pipeline
        let vertex_buffer_definition = vulkano::pipeline::vertex::SingleBufferDefinition::<mesh::Vertex>::new();

        let tmp_pipeline: Arc<pipeline::GraphicsPipelineAbstract + Send + Sync> = Arc::new(vulkano::pipeline::GraphicsPipeline::start()
            .vertex_input(vertex_buffer_definition)
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(vulkano::framebuffer::Subpass::from(renderpass.clone(), 0).expect("failed to set render pass at pipe 01!"))
            .build(device.clone())
            .expect("failed to make pipe 01!"));

        //Create the Struct
        Pipeline{
            pipeline: tmp_pipeline,
            inputs: inputs
        }
    }

    ///Returns the vulkano pipline definition
    pub fn get_pipeline_ref(&self) -> Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>
    {
        self.pipeline.clone()
    }

}
