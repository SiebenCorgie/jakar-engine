use super::shaders::default_pstprg_fragment;
use super::shaders::default_pstprg_vertex;
use render::post_progress::PostProgressVertex;
use super::shader_inputs::DescriptorSetFamiliy;
use render::pipeline_builder::PipelineConfig;
use render::shader_set::ToPipeline;

use vulkano;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::vertex::BufferlessDefinition;
use vulkano::pipeline::shader::EmptyEntryPointDummy as EEPD;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::GraphicsPipelineBuilder;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;

use std::sync::Arc;


pub struct PostProgressSet {
    pub vertex_shader: Arc<default_pstprg_vertex::Shader>,
    pub fragment_shader: Arc<default_pstprg_fragment::Shader>,

    pub vertex_layout: SingleBufferDefinition<PostProgressVertex>,


    ///The Descriptor sets of this shader set.
    pub descriptor_sets: Vec<DescriptorSetFamiliy>,
}


impl PostProgressSet{
    pub fn load(device: Arc<vulkano::device::Device>) -> Self{
        //Load the shaders
        let v_s = default_pstprg_vertex::Shader::load(device.clone()).expect("failed to load vertex shader!");
        let f_s = default_pstprg_fragment::Shader::load(device.clone()).expect("failed to load vertex shader!");

        //Configure the inputs
        let mut descriptors = Vec::new();
        descriptors.push(DescriptorSetFamiliy::MultisampledColorAndDepth);
        descriptors.push(DescriptorSetFamiliy::PostProgressData);

        let vertex_buffer_def = SingleBufferDefinition::<PostProgressVertex>::new();

        PostProgressSet{
            vertex_shader: Arc::new(v_s),
            fragment_shader: Arc::new(f_s),
            vertex_layout: vertex_buffer_def,
            descriptor_sets: descriptors,
        }
    }
}


impl ToPipeline for PostProgressSet{
    ///Converts the builder to a real pipeline
    fn to_pipeline (&self,
        builder: GraphicsPipelineBuilder<BufferlessDefinition, EEPD, (), EEPD, (), EEPD, (), EEPD, (), EEPD, (), ()>,
        pipeline_settings: &PipelineConfig,
        render_pass: Arc<RenderPassAbstract + Send + Sync>,
        device: Arc<Device>,
    ) -> (Arc<GraphicsPipelineAbstract + Send + Sync>, Vec<DescriptorSetFamiliy>){
        println!("Building pipeline based on PostProgressSet shader and vertex ...", );
        //take the current pipeline builder
        let pipeline: Arc<GraphicsPipelineAbstract + Send + Sync> = Arc::new(
            builder
            .render_pass(
                vulkano::framebuffer::Subpass::from(
                    render_pass, pipeline_settings.sub_pass_id
                ).expect("failed to set renderpass for PostProgress shader")
            )
            .vertex_input(SingleBufferDefinition::<PostProgressVertex>::new())
            //now add the vertex and fragment shader, then return the new created pipeline and the inputs
            .vertex_shader(self.vertex_shader.main_entry_point(), ())
            .fragment_shader(self.fragment_shader.main_entry_point(), ()) //Gets as specialisation the max light count
            //now build
            .build(device)
            .expect("failed to build pipeline for PostProgress shader set!")
        );

        //Finally put this in an arc and return along the inputs
        (Arc::new(pipeline), self.descriptor_sets.clone())
    }
}
