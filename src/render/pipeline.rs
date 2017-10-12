use vulkano;
use vulkano::pipeline;

use std::sync::Arc;
use core::resources::mesh;


use render::shader_impls;
use render::pipeline_builder::*;

///Definition of a single pipeline together with its creation and deleting behavoir
///
///Besides the pipeline definition of the vulkan struct the jakar-pipeline is also responsible
///for creation of the descriptor sets, needed to render a material based on this pipeline
///The features are described by an `PipelineInput` struct.
pub struct Pipeline {
    ///The main pipeline hold by this struct
    //TODO make this dynamic, or implement a different pipeline struct per type... maybe one graphic, one computing? (<- will do this)
    //TODO change to graphics_pipeline and add a compute_pipeline
    pub pipeline: Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>,
    ///defines the inputs this pipeline needs to pass to the shader
    pub inputs: PipelineInput,
    ///Stores the config this pipeline was created from
    pub pipeline_config: PipelineConfig,

    //defines several optional descriptor set pools, they depend on the `inputs` parameter

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
        // the shader struct containg max. the following shaders:
        //     - vertex shader
        //     - fragment shader
        //     - geometry shader
        //     - tesselation control shader
        //     - tesselation evaluation shader
        // - shader inputs struct (describes which inputs are needed for this pipeline later)
        // - shader sets (describes which shaders are used for pipeline creation)
        let shader = {
            //now return stuff depending on the loaded shader
            match pipeline_configuration.shader_set{
                shader_impls::ShaderTypes::PbrOpaque => {

                    //load the shader based on the type the use wants to load
                    let shader = shader_impls::load_shader(device.clone(), shader_impls::ShaderTypes::PbrOpaque);
                    //extract some infos which doesnt need to be stored in an enum for the compiler

                    //build the return
                    shader
                }

                shader_impls::ShaderTypes::Wireframe => {
                    let shader = shader_impls::load_shader(device.clone(), shader_impls::ShaderTypes::Wireframe);
                    //build the return
                    shader
                }
            }
        };



        //Create a pipeline vertex buffer definition
        let vertex_buffer_definition = vulkano::pipeline::vertex::SingleBufferDefinition::<mesh::Vertex>::new();

        //Now start the pipeline and configure it based on the PipelineSettings
        //TODO make sure to get a: Option<Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>>
        let mut pipeline = Some(
            vulkano::pipeline::GraphicsPipeline::start()
        );

        //add the vertex buffer definition and create a new pipeline from it
        let mut vertex_def_pipeline = {
            Some(
                pipeline
                .take()
                .expect("failed to get pipeline #1")
                .vertex_input(vertex_buffer_definition)
            )
        };

        //set the topolgy type
        let mut topology_pipeline = {
            Some(
                vertex_def_pipeline
                .take()
                .expect("failed to get pipeline #1")
                .primitive_topology(pipeline_configuration.topology_type)
            )
        };

        //Set the viewport and scissors behavoir
        let mut view_scis_pipeline = {

            match pipeline_configuration.viewport_scissors{
                ViewportScissorsBehavoir::DefinedViewport(ref viewport)=> {
                    Some(
                        topology_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .viewports(vec![viewport.clone()])
                    )
                },
                ViewportScissorsBehavoir::DefinedViewportAndScissors((ref viewport, ref scissor))=> {
                    Some(
                        topology_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .viewports_scissors(vec![(viewport.clone(), scissor.clone())])
                    )
                },
                ViewportScissorsBehavoir::DynamicViewportFixedScissors(ref scissors)=> {
                    Some(
                        topology_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .viewports_dynamic_scissors_fixed(vec![scissors.clone()])
                    )
                },
                ViewportScissorsBehavoir::DynamicViewportScissorsIrrelevant(ref id)=> {
                    Some(
                        topology_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .viewports_dynamic_scissors_irrelevant(id.clone())
                    )
                },
                ViewportScissorsBehavoir::FixedViewportDynamicScissors(ref viewport)=> {
                    Some(
                        topology_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .viewports_fixed_scissors_dynamic(vec![viewport.clone()])
                    )
                },
                ViewportScissorsBehavoir::ViewportScissorsDynamic(ref id)=> {
                    Some(
                        topology_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .viewports_scissors_dynamic(id.clone())
                    )
                },
            }
        };


        //set depth clamp
        let mut depth_clamp_pipeline = {
            Some(
                view_scis_pipeline
                .take()
                .expect("failed to get pipeline #1")
                .depth_clamp(pipeline_configuration.has_depth_clamp)
            )
        };

        //Setup clockwise or counter clockwise faces
        let mut face_rot_pipeline = {
            if pipeline_configuration.has_faces_clockwise{
                Some(
                    depth_clamp_pipeline
                    .take()
                    .expect("failed to get pipeline #1")
                    .front_face_counter_clockwise()
                )
            }else{
                //if not inverted just go on with the old one
                depth_clamp_pipeline
            }
        };

        //setup cull mode of the vertices
        let mut cull_pipeline = {
            match pipeline_configuration.cull_mode{
                CullMode::Disabled =>{
                    Some(
                        face_rot_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .cull_mode_disabled()
                    )
                },
                CullMode::Back => {
                    Some(
                        face_rot_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .cull_mode_back()
                    )
                },
                CullMode::Front => {
                    Some(
                        face_rot_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .cull_mode_front()
                    )
                },
                CullMode::FrontAndBack => {
                    Some(
                        face_rot_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .cull_mode_front_and_back()
                    )
                }
            }
        };

        //Set the polyfone drawing mode
        let mut poly_mode_pipeline = {
            match pipeline_configuration.polygone_mode{
                PolygoneMode::Fill => {
                    Some(
                        cull_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .polygon_mode_fill()
                    )
                },
                PolygoneMode::Line(line_width) => {
                    Some(
                        cull_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .polygon_mode_line()
                        .line_width(line_width)
                    )
                },
                PolygoneMode::Point => {
                    Some(
                        cull_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .polygon_mode_point()
                    )
                }
            }
        };

        //Setup depth and stencil properties
        let mut depth_stencil_pipeline = {
            match pipeline_configuration.depth_stencil{
                DepthStencilConfig::SimpleDepthNoStencil => {
                    Some(
                        poly_mode_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .depth_stencil_simple_depth()
                    )
                },
                DepthStencilConfig::NoDepthNoStencil => {
                    Some(
                        poly_mode_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .depth_stencil_disabled()
                    )
                },
                DepthStencilConfig::CustomDepthAndStencil(ref config) => {
                    Some(
                        poly_mode_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .depth_stencil(config.clone())
                    )
                },
            }
        };

        //Setup the blending operation
        let mut blending_pipeline = {

            let mut tmp_bl_pipe = {
                match pipeline_configuration.blending_operation{
                    BlendTypes::BlendCollective(ref attachment) => {
                        depth_stencil_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .blend_collective(attachment.clone())
                    },
                    BlendTypes::BlendPassThrough => {
                        depth_stencil_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .blend_pass_through()
                    },
                    BlendTypes::BlendAlphaBlending => {
                        depth_stencil_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .blend_alpha_blending()
                    }
                    BlendTypes::BlendLogicOp(ref op) => {
                        depth_stencil_pipeline
                        .take()
                        .expect("failed to get pipeline #1")
                        .blend_logic_op(op.clone())
                    }
                }
            };

            //have a look if we have to diable the blend op and if we have to set the blend const
            if pipeline_configuration.disabled_logic_op{
                tmp_bl_pipe = tmp_bl_pipe.blend_logic_op_disabled();
            }

            match pipeline_configuration.blending_constant{
                Some(b_const) => {
                    tmp_bl_pipe = tmp_bl_pipe.blend_constants(b_const)
                },
                None => {}, //don't change
            }

            //now return the Option
            Some(tmp_bl_pipe)
        };

        //nearly done :)
        //setup renderpass from the stored renderpass and the id
        let mut renderpass_pipeline = {
            Some(
                blending_pipeline
                .take()
                .expect("failed to get pipeline #1")
                .render_pass(
                    vulkano::framebuffer::Subpass::from(
                        pipeline_configuration.render_pass.clone(), //extracted this one at the top of this function //TODO after deleting the old approach this can move here
                        pipeline_configuration.sub_pass_id
                    )
                    .expect("failed to set supass for renderpass ")
                )
            )
        };


        //Set vertex_shader, fragment shader, geometry shader and tesselation shader at once
        //and build the pipeline to an Arc<GraphicsPipelineAbstract> for easy storage
        let (final_pipeline, pipeline_inputs) = {
            //sort the shaders and return generated Arc<GraphicsPipelineAbstract>
            match shader{
                shader_impls::JakarShaders::PbrOpaque((vs, fs, inputs)) => {

                    //take the current pipeline builder
                    let pipeline = renderpass_pipeline
                    .take()
                    .expect("failed to get pipeline #1")
                    //now add the vertex and fragment shader, then return the new created pipeline and the inputs
                    .vertex_shader(vs.main_entry_point(), ())
                    .fragment_shader(fs.main_entry_point(), ())
                    //now build
                    .build(device)
                    .expect("failed to build pipeline for PBR-Opaque shader set!");

                    //Finally put this in an arc and return along the inputs
                    (Arc::new(pipeline), inputs)
                },
                shader_impls::JakarShaders::Wireframe((vs, fs, inputs)) => {
                    //take the current pipeline builder
                    let pipeline = renderpass_pipeline
                    .take()
                    .expect("failed to get pipeline #1")
                    //now add the vertex and fragment shader, then return the new created pipeline and the inputs
                    .vertex_shader(vs.main_entry_point(), ())
                    .fragment_shader(fs.main_entry_point(), ())
                    //now build
                    .build(device)
                    .expect("failed to build pipeline for PBR-Opaque shader set!");

                    //Finally put this in an arc and return along the inputs
                    (Arc::new(pipeline), inputs)
                },
            }
        };


        //Create the Struct
        Pipeline{
            pipeline: final_pipeline,
            inputs: pipeline_inputs,
            pipeline_config: pipeline_configuration
        }
    }

    ///Returns the vulkano pipline definition
    pub fn get_pipeline_ref(&self) -> Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>
    {
        self.pipeline.clone()
    }

    ///Returns the inputs needed to feed the pipeline correctly
    pub fn get_inputs(&self) -> PipelineInput{
        self.inputs
    }

}
