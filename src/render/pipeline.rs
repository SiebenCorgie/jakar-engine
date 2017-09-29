use vulkano;
use vulkano::pipeline;
use vulkano_shaders;
use vulkano::pipeline::shader::GraphicsEntryPointAbstract;
use vulkano::pipeline::depth_stencil::DepthStencil;
use vulkano::pipeline::blend::AttachmentBlend;
use vulkano::pipeline::blend::LogicOp;
use vulkano::framebuffer::Subpass;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::RenderPassDesc;
use vulkano::pipeline::input_assembly::PrimitiveTopology;

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

///Descibes all possible blend types for a fragment
pub enum BlendTypes {
    ///Describes every tiny bit about how blending should be done
    BlendCollective(AttachmentBlend),
    ///The output gets directly written to the frame buffer (default)
    BlendPassThrough,
    ///Blends based on the alpha value
    BlendAlphaBlending,
    ///Blends based on a logic operator
    BlendLogicOp(LogicOp),
}

///Describes how the depth and stencil test should be handled
pub enum DepthStencilConfig {
    SimpleDepthNoStencil,
    NoDepthNoStencil,
    CustomDepthAndStencil(pipeline::depth_stencil::DepthStencil),
}


///A struct which can be used to configure the pipeline which can be build via the `new()` functions
///of the `Pipeline` struct.
///NOTE:
///
/// When specifying the shader sets via `shader_type` the engine will decide wether it has to
/// implement a tesselation and/or gemoetry shader or not.
pub struct PipelineConfig {

    ///Defines the shader type of the loaded pipeline.
    ///TODO this should be moved in a less static abroach, but I don't know how atm.
    pub shader_set: shader_impls::ShaderTypes,

    ///Describes how vertices must be grouped together to form primitives.
    pub topology_type: PrimitiveTopology,

    ///Describes the Vieport and scissors behavoir of the new pipeline
    pub viewport_scissors: ViewportScissorsBehavoir,

    ///True if the depth should be clamped between 0.0 and 1.0 for each vertice. Otherwise vertices out
    /// of the values between 0.0-1.0 will be discarded by vulkan.
    pub has_depth_clamp: bool,

    ///Should be true if the faces are oriented clockwise. The default is counter clockwise
    pub has_faces_clockwise: bool,

    ///Defines the cull mode of this pipeline
    pub cull_mode: CullMode,

    ///Defines how the polygones are drawn
    pub polygone_mode: PolygoneMode,

    ///Sets how depth and stencil should be handled, if you choose to write the depth buffer, you should
    /// also provied one.
    pub depth_stencil: DepthStencilConfig,

    ///Descibes how the fragment output should be blended into the frame buffer
    pub blending_operation: BlendTypes,

    ///Disables the logic operation when blending (default is true)
    pub disabled_logic_op: bool,

    ///Sets a blending constant (default is [0.0, 0.0, 0.0, 0.0]).
    ///If you want to set the constant dynamic per frame, choose `None` as value!
    pub blending_constant: Option<[f32; 4]>,

    ///Sets the render pass / subpass to use
    pub render_pass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,

    ///Sets the Id of the sub pass in this render pass to use. If you have only one pass (the main pass),
    ///use the `id: 0`.
    pub sub_pass_id: u32
}

impl PipelineConfig{
    ///Creates a default configuration with:
    ///
    /// - Single Vertex buffer
    /// - The vertices are a `TriangleList`
    /// - Vertex and fragment shader input
    /// - Opaque PBR shading
    /// - Filled polygones
    /// - Dynamic Viewport size and a scissors covering always the whole viewport
    /// - Writes to depth buffer but not stencil buffer
    /// - Creates subpass from index 0 of the provided `renderpass`
    ///
    /// This configuration is suitable for most simple drawing operations.
    #[inline]
    pub fn default(renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>) -> Self {

        PipelineConfig {
            shader_set: shader_impls::ShaderTypes::PbrOpaque,
            topology_type: PrimitiveTopology::TriangleList,
            viewport_scissors: ViewportScissorsBehavoir::DynamicViewportScissorsIrrelevant(1), //TODO find out what the 1 has to do
            has_depth_clamp: false,
            has_faces_clockwise: false,
            cull_mode: CullMode::Disabled,
            polygone_mode: PolygoneMode::Fill,
            depth_stencil: DepthStencilConfig::SimpleDepthNoStencil,
            blending_operation: BlendTypes::BlendPassThrough,
            disabled_logic_op: false,
            blending_constant: Some([0.0; 4]),
            render_pass: renderpass,
            sub_pass_id: 0
        }
    }

    ///Changes the shader set to a custom value
    #[inline]
    pub fn with_shader(mut self, shader_set: shader_impls::ShaderTypes) -> Self{
        self.shader_set = shader_set;
        self
    }

    ///Changes the topology type.
    /// Have a look a the types in
    // [vulkano](https://docs.rs/vulkano/0.7.0/vulkano/pipeline/input_assembly/enum.PrimitiveTopology.html)
    // for available types!
    #[inline]
    pub fn with_primitve_topology(mut self, new_pri_top: PrimitiveTopology) -> Self{
        self.topology_type = new_pri_top;
        self
    }

    ///Sets the viewport and scissors behavoir to a custom type. Make sure to supply needed information
    /// in your command buffer if you set any value to dynamic!
    #[inline]
    pub fn with_viewport_and_scissors_behavoir(mut self, viewport_scissors: ViewportScissorsBehavoir) -> Self{
        self.viewport_scissors = viewport_scissors;
        self
    }

    ///Can be set to true if you want to clamp the depth value of any vertice between 0.0  and 1.0.
    /// otherwise the values out of this dpeth will be discarded which is usually the wanted behavoir.
    #[inline]
    pub fn clamp_depth(mut self, new_val: bool) -> Self{
        self.has_depth_clamp = new_val;
        self
    }

    ///Can be set to true if the winding order of the supplied vertices is clockwise.
    /// The default is counter clockwise.
    /// Use this setting if your vertices are only visible from the "inside" when backface culling
    /// is enabled.
    #[inline]
    pub fn with_clockwise_winding(mut self, is_clockwise: bool) -> Self{
        self.has_faces_clockwise = is_clockwise;
        self
    }

    ///Sets the cullmode of the vertices to a custom value.
    ///
    /// By default both sides of an vertice are rendered. However, you can archiev much better
    /// preformace if you render only one side for an object which has no vertice holes like an stone.
    ///
    /// However, it is much better render both sides for thin things like gras planes, glas and fabrics.
    #[inline]
    pub fn with_cull_mode(mut self, new_mode: CullMode) -> Self{
        self.cull_mode = new_mode;
        self
    }

    ///Sets how the vertice are drawn. Can be changed to draw lines and points.
    /// As specially lines are usually used to create wireframe images of an object.
    #[inline]
    pub fn with_polygone_mode(mut self, new_mode: PolygoneMode) -> Self{
        self.polygone_mode = new_mode;
        self
    }

    ///Can be used to changed how depth and stencil buffer are written.
    ///
    /// NOTE: If you are using depth or stencil buffer, be sure to supply a target buffer.
    #[inline]
    pub fn with_depth_and_stencil_settings(mut self, new_settings: DepthStencilConfig) -> Self{
        self.depth_stencil = new_settings;
        self
    }

    /// Can be used to blend several fragments. By default the old fragment always get overwritten
    /// by the new fragment. When rendering something transparent like glas this might not be
    /// intended. Therefore you should change it to some kind of blending.
    #[inline]
    pub fn with_blending(mut self, new_blend_mode: BlendTypes) -> Self {
        self.blending_operation = new_blend_mode;
        self
    }

    ///Can be set to true if logic operations should be disabled or false if they should be enabled.
    ///
    /// Default is `false`.
    #[inline]
    pub fn with_disabled_logical_op(mut self, new_val: bool) -> Self{
        self.disabled_logic_op = new_val;
        self
    }

    ///Sets another blending constant. Can also be set to `None` if you want to set the value per frame.
    #[inline]
    pub fn with_blend_constant(mut self, new_const: Option<[f32; 4]>) -> Self{
        self.blending_constant = new_const;
        self
    }

    ///Set the id from which pass in the `render_pass` the pipeline should be created from.
    #[inline]
    pub fn from_subpass_id(mut self, new_id: u32) -> Self{
        self.sub_pass_id = new_id;
        self
    }


}



///Describes the input needed for the shaders in this pipeline to work.
///
/// #panics
///
///This could panic if the input is defined wrong, mostly the engine won't build though
#[derive(Copy, Clone)]
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

impl PipelineInput {
    ///Creates a `PipelineInput` which includes all inputs available
    pub fn new_all() -> Self{
        PipelineInput{
            data: true,
            has_textures: true,
            has_light: true,
        }
    }

    ///Creates a config where everything is turned of *except* data which is always needed
    pub fn with_none() -> Self{
        PipelineInput{
            data: true,
            has_textures: false,
            has_light: false,
        }
    }
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
    ///Creates a pipeline for a 'shader_set' from a `pipeline_configuration`.
    ///
    /// NOTE:
    ///
    /// Some things are not configurable like the vertex buffer definition. They are set for this engine but
    /// this might change in the future if needed.
    pub fn new(
        device: Arc<vulkano::device::Device>,
        pipeline_configuration: PipelineConfig,
    )
        -> Self
    {

        //load the shader from configuration
        // the tubel loads the following objects depending on type of the shader (in order):
        // - vertex shader
        // - fragment shader
        // - geometry shader
        // - tesselation control shader
        // - tesselation evaluation shader
        // - shader inputs struct (describes which inputs are needed for this pipeline later)
        // - shader sets (describes which shaders are used for pipeline creation)
        let (shader, shader_type, shader_inputs, used_shader_sets) = {
            //now return stuff depending on the loaded shader
            match pipeline_configuration.shader_set{
                shader_impls::ShaderTypes::PbrOpaque => {

                    //load the shader based on the type the use wants to load
                    let shader = shader_impls::load_shader(device.clone(), shader_impls::ShaderTypes::PbrOpaque);
                    //extract some infos which doesnt need to be stored in an enum for the compiler
                    let (inputs, sets) = match shader{
                        shader_impls::JakarShaders::PbrOpaque((_, _, inputs, sets)) =>{
                            (inputs, sets)
                        },
                        _ => panic!("could not match shaders for inputs and shader set type"),
                    };

                    //build the return tubel
                    (shader, shader_impls::ShaderTypes::PbrOpaque, inputs, sets)
                }

                shader_impls::ShaderTypes::Wireframe => {
                    let shader = shader_impls::load_shader(device.clone(), shader_impls::ShaderTypes::Wireframe);
                    let (inputs, sets) = match shader{
                        shader_impls::JakarShaders::Wireframe((_, _, inputs, sets)) =>{
                            (inputs, sets)
                        },
                        _ => panic!("could not match shaders for inputs and shader set type"),
                    };
                    //build the return tubel
                    (shader, shader_impls::ShaderTypes::Wireframe, inputs, sets)
                }
            }
        };



        //Currently using a static shader from /data/test.vs/fs
        let vs = pbr_vertex::Shader::load(device.clone()).expect("failed to create shader module");
        let fs = pbr_fragment::Shader::load(device.clone()).expect("failed to create shader module");


        //get the renderpass
        let render_pass = pipeline_configuration.render_pass;

        //Create a pipeline
        let vertex_buffer_definition = vulkano::pipeline::vertex::SingleBufferDefinition::<mesh::Vertex>::new();

        //Now start the pipeline and configure it based on the PipelineSettings

        let tmp_pipeline: Arc<pipeline::GraphicsPipelineAbstract + Send + Sync> = Arc::new(vulkano::pipeline::GraphicsPipeline::start()
            .vertex_input(vertex_buffer_definition)
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(vulkano::framebuffer::Subpass::from(render_pass, 0).expect("failed to set render pass at pipe 01!"))
            .build(device.clone())
            .expect("failed to make pipe 01!"));

        //Create the Struct
        Pipeline{
            pipeline: tmp_pipeline,
            inputs: shader_inputs
        }
    }

    ///Returns the vulkano pipline definition
    pub fn get_pipeline_ref(&self) -> Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>
    {
        self.pipeline.clone()
    }

}
