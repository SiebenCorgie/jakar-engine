use std::collections::BTreeMap;

use rt_error;
use render::pipeline;

use std::sync::Arc;

use vulkano;
use vulkano::pipeline::GraphicsPipelineAbstract;


///Manages all available pipeline
pub struct PipelineManager {
    pipelines: BTreeMap<String, pipeline::Pipeline>,
}

impl PipelineManager{

    ///Creates a pipeline Manager with a default pipeline, have a look at the code to see the pipeline type
    pub fn new(
        device: Arc<vulkano::device::Device>,
        renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
    ) -> Self
    {
        let mut b_tree_map = BTreeMap::new();
        //Creates a default pipeline from a default shader

        //the default inputs (all for the best visual graphics)
        let inputs_default = pipeline::PipelineInput{
            data: true,
            has_textures: true,
            has_light: true,
        };

        let default_pipeline = pipeline::Pipeline::new_opaque(device, renderpass, inputs_default);
        b_tree_map.insert(String::from("DefaultPipeline"), default_pipeline);

        PipelineManager{
            pipelines: b_tree_map,
        }
    }

    ///Returns true if there is a pipeline with this name
    pub fn has_pipeline(&self, name: &str) -> bool{
        if self.pipelines.contains_key(&String::from(name)){
            return true
        }
        false
    }

    ///Should always return the normal PBR pipeline, if it panics, please file a bug report, this should not happen
    pub fn get_default_pipeline(&mut self) -> Arc<GraphicsPipelineAbstract + Send + Sync>{
        match self.pipelines.get_mut(&String::from("DefaultPipeline")){
            Some(ref mut pipe) => return pipe.get_pipeline_ref(),
            None =>rt_error("PIPELINE_MANAGER", "PIPELINE MANAGER: Could not find default pipe this should not happen"),
        }
        panic!("Crash could not get default pipeline!")
    }

    ///Returns a pipeline by name, if not existend, returns the default pipeline
    pub fn get_pipeline_by_name(&mut self, name: &str) -> Arc<GraphicsPipelineAbstract + Send + Sync>{
        //println!("SEARCHING FOR PIPELINE: {}", name.clone() );
        match self.pipelines.get_mut(&String::from(name)){
            Some(ref mut pipe) => return pipe.get_pipeline_ref(),
            None => rt_error("PIPELINE_MANAGER","Could not find pipe"),
        }
        self.get_default_pipeline()
    }

    ///Adds a pipeline made for the specified shader
    ///TODO make the shader specified
    pub fn add_pipeline(&mut self, name: &str,device: Arc<vulkano::device::Device>,
        renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
    )
    {
        //the default inputs (all for the best visual graphics)
        let inputs_default = pipeline::PipelineInput{
            data: true,
            has_textures: true,
            has_light: true,
        };

        let tmp_pipeline = pipeline::Pipeline::new_opaque(device,renderpass, inputs_default);
        self.pipelines.insert(String::from(name), tmp_pipeline);
    }

}
