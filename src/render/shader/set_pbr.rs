use super::shaders::pbr_fragment;
use super::shaders::pbr_vertex;
use render::shader_set::*;
use super::shader_inputs::DescriptorSetFamiliy;
use core::resources::mesh::Vertex;
use render::pipeline_builder::PipelineConfig;

use vulkano;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::vertex::BufferlessDefinition;
use vulkano::pipeline::shader::EmptyEntryPointDummy as EEPD;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::GraphicsPipelineBuilder;
use vulkano::device::Device;
use vulkano::framebuffer::RenderPassAbstract;
use std::sync::Arc;

///Provides the building methode for the shader library
pub struct PbrSet{
    pub vertex_shader: Arc<pbr_vertex::Shader>,
    pub fragment_shader: Arc<pbr_fragment::Shader>,

    pub vertex_layout: SingleBufferDefinition<Vertex>,

    pub descriptor_sets: Vec<DescriptorSetFamiliy>,
}

//Loads the shader set set configures needed inputs for the building
impl PbrSet{
    pub fn load(device: Arc<vulkano::device::Device>) -> Self{
        //Load the shaders
        let v_s = pbr_vertex::Shader::load(device.clone()).expect("failed to load vertex shader!");
        let f_s = pbr_fragment::Shader::load(device.clone()).expect("failed to load vertex shader!");

        //Configure the inputs
        let mut descriptors = Vec::new();
        descriptors.push(DescriptorSetFamiliy::CameraData);
        descriptors.push(DescriptorSetFamiliy::MaterialTextures);
        descriptors.push(DescriptorSetFamiliy::MaterialData);
        descriptors.push(DescriptorSetFamiliy::Lights);

        let vertex_buffer_def = SingleBufferDefinition::<Vertex>::new();

        PbrSet{
            vertex_shader: Arc::new(v_s),
            fragment_shader: Arc::new(f_s),
            vertex_layout: vertex_buffer_def,
            descriptor_sets: descriptors,
        }
    }
}

impl ToPipeline for PbrSet{
    ///Converts the builder to a real pipeline
    fn to_pipeline (&self,
        builder: GraphicsPipelineBuilder<BufferlessDefinition, EEPD, (), EEPD, (), EEPD, (), EEPD, (), EEPD, (), ()>,
        pipeline_settings: &PipelineConfig,
        render_pass: Arc<RenderPassAbstract + Send + Sync>,
        device: Arc<Device>,
    ) -> (Arc<GraphicsPipelineAbstract + Send + Sync>, Vec<DescriptorSetFamiliy>){
        println!("Building pipeline based on PbrOpaque shader and vertex ...", );
        //take the current pipeline builder
        let pipeline: Arc<GraphicsPipelineAbstract + Send + Sync> = Arc::new(
            builder
            .render_pass(
                vulkano::framebuffer::Subpass::from(
                    render_pass, pipeline_settings.sub_pass_id
                ).expect("failed to set renderpass for pbr shader")
            )
            .vertex_input(SingleBufferDefinition::<Vertex>::new())
            //now add the vertex and fragment shader, then return the new created pipeline and the inputs
            .vertex_shader(self.vertex_shader.main_entry_point(), ())
            .fragment_shader(self.fragment_shader.main_entry_point(), ()) //Gets as specialisation the max light count
            //now build
            .build(device)
            .expect("failed to build pipeline for PBR-Opaque shader set!")
        );

        //Finally put this in an arc and return along the inputs
        (Arc::new(pipeline), self.descriptor_sets.clone())
    }
}
