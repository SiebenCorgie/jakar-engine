use std::sync::Arc;
use vulkano;

use std::collections::HashMap;
use render::shader::DefaultShaderSets;
use render::shader::shader_inputs::DescriptorSetFamiliy;
use render::pipeline_builder::PipelineConfig;

use vulkano::pipeline::vertex::BufferlessDefinition;
use vulkano::pipeline::shader::EmptyEntryPointDummy as EEPD;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::GraphicsPipelineBuilder;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;

///A small manager which holds all the available shader set, ordered by name
pub struct ShaderManager {
    device: Arc<vulkano::device::Device>,
    shader_sets: HashMap<String, Arc<ToPipeline + Send + Sync>>
}

impl ShaderManager{
    ///Starts the manager
    pub fn new(device: Arc<vulkano::device::Device>) -> Self{
        ShaderManager{
            device: device,
            shader_sets: HashMap::new()
        }
    }

    ///Returns a list of available names
    pub fn get_all_shader_sets(&self) -> Vec<String>{
        let mut ret_vec = Vec::new();
        for (name, _) in self.shader_sets.iter(){
            ret_vec.push(name.clone());
        }
        ret_vec
    }

    ///Querrys all current shader sets for one with this name. If there is non, try to get one from the
    /// current librarys (TODO: add the library functions)
    pub fn get_shader_set(&mut self, name: String) -> Option<Arc<ToPipeline + Send + Sync>>{
        match self.shader_sets.get(&name){
            Some(e) => return Some(e.clone()),
            None => {},

        }
        //found non, trying to load one
        match DefaultShaderSets::new().get_shader_set(name.clone(), self.device.clone()){
            Some(shader) => {
                //add to manager and return reference
                self.shader_sets.insert(name, shader.clone());
                return Some(shader)
            },
            _ => return None,
        }

    }
}


///A lib of shader sets should implement this trait
pub trait ShaderLibrary {
    ///Returns true if the library has a shader set with this name
    fn has_shader_set(&self, name: String) -> bool;
    ///Returns the shader set with this name
    fn get_shader_set(&self, name: String, device: Arc<Device>) -> Option<Arc<ToPipeline + Send + Sync>>;
}


///Everything that implments that trait has to be able to take a pipeline builder and change that
/// into a jakar-pipeline object together with a Vector of the descriptoirsets used etc.
pub trait ToPipeline {
    ///Converts the builder to a real pipeline
    fn to_pipeline(&self,
        builder: GraphicsPipelineBuilder<BufferlessDefinition, EEPD, (), EEPD, (), EEPD, (), EEPD, (), EEPD, (), ()>,
        pipeline_settings: &PipelineConfig,
        render_pass: Arc<RenderPassAbstract + Send + Sync>,
        subpass_id: u32,
        device: Arc<Device>,
    ) -> (Arc<GraphicsPipelineAbstract + Send + Sync>, Vec<DescriptorSetFamiliy>);
}
