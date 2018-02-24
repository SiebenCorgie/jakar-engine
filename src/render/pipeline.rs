
use vulkano;
use vulkano::pipeline;
use vulkano::framebuffer::RenderPassAbstract;

use render::pipeline_builder::*;
use render::shader_manager;
use render::shader::shader_inputs::DescriptorSetFamiliy;

use std::sync::{Arc};

///Definition of a single pipeline together with its creation and deleting behavoir
///
///Besides the pipeline definition of the vulkan struct the jakar-pipeline is also responsible
///for creation of the descriptor sets, needed to render a material based on this pipeline
///The features are described by an `PipelineInput` struct.
pub struct Pipeline {
    ///The main pipeline hold by this struct
    pub pipeline: Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>,
    ///List the descriptorsets needed for this pipeline in their order.
    inputs: Vec<DescriptorSetFamiliy>,
    ///Stores the config this pipeline was created from
    pub pipeline_config: PipelineConfig,
}

impl Pipeline{
    ///Creates a pipeline for a 'shader_set' from a `pipeline_configuration`, at a `sub_pass` id of the `target_subpass`
    ///
    /// NOTE:
    ///
    /// Some things are not configurable like the vertex buffer definition. They are set for this engine but
    /// this might change in the future if needed.
    pub fn new(
        device: Arc<vulkano::device::Device>,
        pipeline_configuration: PipelineConfig,
        render_pass: Arc<RenderPassAbstract + Send + Sync>,
        shader_set: Arc<shader_manager::ToPipeline>,
    )
        -> Self

    {
        //Now start the pipeline and configure it based on the PipelineSettings
        //TODO make sure to get a: Option<Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>>
        let mut pipeline = Some(
            vulkano::pipeline::GraphicsPipeline::start()
        );

        //set the topolgy type
        let mut topology_pipeline = {
            Some(
                pipeline
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




        //Set vertex_shader, fragment shader, geometry shader and tesselation shader at once
        //and build the pipeline to an Arc<GraphicsPipelineAbstract> for easy storage
        let (final_pipeline, pipeline_inputs) = shader_set.to_pipeline(
            blending_pipeline.take().expect("failed to take final confgured pipeline"),
            &pipeline_configuration,
            render_pass,
            device
        );

        //Create the Struct
        Pipeline{
            pipeline: final_pipeline,
            inputs: pipeline_inputs,
            pipeline_config: pipeline_configuration,
        }
    }

    ///Returns the vulkano pipline definition
    pub fn get_pipeline_ref(&self) -> Arc<pipeline::GraphicsPipelineAbstract + Send + Sync>
    {
        self.pipeline.clone()
    }

    ///Returns the inputs needed to feed the pipeline correctly
    pub fn get_inputs(&self) -> Vec<DescriptorSetFamiliy>{
        self.inputs.clone()
    }

    ///Prints the shader type the pipeline is based on for debug reasons
    pub fn print_shader_name(&self){
        println!("ShaderSetUsed: {}", self.pipeline_config.shader_set);
    }
}
