use super::shaders::wireframe_fragment;
use super::shaders::wireframe_vertex;
use render::shader_manager::*;
use core::resources::mesh::Vertex;
use super::shader_inputs::DescriptorSetFamiliy;
use render::pipeline_builder::PipelineConfig;

use std::sync::Arc;

use vulkano;
use vulkano::pipeline::vertex::SingleBufferDefinition;
use vulkano::pipeline::vertex::BufferlessDefinition;
use vulkano::pipeline::shader::EmptyEntryPointDummy as EEPD;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::GraphicsPipelineBuilder;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;


pub struct SetWireframe {
    pub vertex_shader: Arc<wireframe_vertex::Shader>,
    pub fragment_shader: Arc<wireframe_fragment::Shader>,

    pub vertex_layout: SingleBufferDefinition<Vertex>,


    ///The Descriptor sets of this shader set.
    pub descriptor_sets: Vec<DescriptorSetFamiliy>,
}

impl SetWireframe{
    pub fn load(device: Arc<vulkano::device::Device>) -> Self{
        //Load the shaders
        let v_s = wireframe_vertex::Shader::load(device.clone()).expect("failed to load vertex shader!");
        let f_s = wireframe_fragment::Shader::load(device.clone()).expect("failed to load vertex shader!");

        //Configure the inputs
        let mut descriptors = Vec::new();
        descriptors.push(DescriptorSetFamiliy::CameraData);

        let vertex_buffer_def = SingleBufferDefinition::<Vertex>::new();

        SetWireframe{
            vertex_shader: Arc::new(v_s),
            fragment_shader: Arc::new(f_s),
            vertex_layout: vertex_buffer_def,
            descriptor_sets: descriptors,
        }
    }
}

impl ToPipeline for SetWireframe{
    ///Converts the builder to a real pipeline
    fn to_pipeline (&self,
        builder: GraphicsPipelineBuilder<BufferlessDefinition, EEPD, (), EEPD, (), EEPD, (), EEPD, (), EEPD, (), ()>,
        pipeline_settings: &PipelineConfig,
        render_pass: Arc<RenderPassAbstract + Send + Sync>,
        subpass_id: u32,
        device: Arc<Device>,
    ) -> (Arc<GraphicsPipelineAbstract + Send + Sync>, Vec<DescriptorSetFamiliy>){
        println!("Building pipeline based on Wireframe shader and vertex ...", );
        //take the current pipeline builder
        let pipeline: Arc<GraphicsPipelineAbstract + Send + Sync> = Arc::new(
            builder
            .render_pass(
                vulkano::framebuffer::Subpass::from(
                    render_pass, subpass_id
                ).expect("failed to set renderpass for Wireframe shader")
            )
            .vertex_input(SingleBufferDefinition::<Vertex>::new())
            //now add the vertex and fragment shader, then return the new created pipeline and the inputs
            .vertex_shader(self.vertex_shader.main_entry_point(), ())
            .fragment_shader(self.fragment_shader.main_entry_point(), ()) //Gets as specialisation the max light count
            //now build
            .build(device)
            .expect("failed to build pipeline for Wireframe shader set!")
        );

        //Finally put this in an arc and return along the inputs
        (Arc::new(pipeline), self.descriptor_sets.clone())
    }
}
