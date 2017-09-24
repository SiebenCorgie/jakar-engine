use vulkano;
use vulkano::pipeline;
use vulkano_shaders;


use std::sync::Arc;
use core::resources::mesh;



use render::shader_impls::pbr_vertex;
use render::shader_impls::pbr_fragment;

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
    pub fn new(
        device: Arc<vulkano::device::Device>,
        renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
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
            inputs: PipelineInput{
                data: true,
                has_textures: true,
                has_light: true,
            }
        }
    }

    ///Returns the vulkano pipline definition
    pub fn get_pipeline_ref(&self) -> Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>
    {
        self.pipeline.clone()
    }

}
