use std::collections::BTreeMap;

use rt_error;
use render::pipeline;
use render::render_passes::{RenderPasses, RenderPassConf};
use render::pipeline_builder;
use render::shader_manager::ShaderManager;

use std::sync::{Arc, Mutex, MutexGuard};

use vulkano;

///Contains the requirements an material can have for a pipeline.
/// Can be used to search for an pipeline which has thoose requirements or, if there is non, create one.
#[derive(Clone)]
pub struct PipelineRequirements {
    ///Describes how the blending should be done
    pub blend_type: pipeline_builder::BlendTypes,
    ///Describes which side of an polygone should be discarded
    pub culling: pipeline_builder::CullMode,
    ///Describes the needed render pass type
    pub render_pass: RenderPassConf,
    ///Describes the shader set that should be used
    pub shader_set: String,
}

impl PipelineRequirements{
    ///Returns true if other has the same configuration as self
    pub fn compare(&self, other: &Self) -> bool{
        if self.blend_type != other.blend_type{
            return false;
        }

        if self.culling != other.culling{
            return false;
        }
        if self.render_pass != other.render_pass{
            return false;
        }
        if self.shader_set != other.shader_set{
            return false;
        }

        true
    }
}


///Manages all available pipeline
pub struct PipelineManager {
    //stores all the pipelines
    pipelines: BTreeMap<String, Arc<pipeline::Pipeline>>,
    //Manages the useable shader sets. Loads them when needed onec and can provide copys of the sets.
    shader_manager: ShaderManager,
    //strores the device this pipeline is based on
    device: Arc<vulkano::device::Device>,
    //A copy the available render passes. They will be used to translate the pass used in the
    //pipeline config
    passes: Arc<Mutex<RenderPasses>>,
}



impl PipelineManager{

    ///Creates a pipeline Manager without any pipeline, they have to be loaded from a config.
    pub fn new(
        device: Arc<vulkano::device::Device>,
        passes: Arc<Mutex<RenderPasses>>,
    ) -> Self
    {
        PipelineManager{
            pipelines: BTreeMap::new(),
            shader_manager: ShaderManager::new(device.clone()),
            device: device,
            passes: passes,
        }
    }


    ///Returns true if there is a pipeline with this name
    #[inline]
    pub fn has_pipeline(&self, name: &str) -> bool{
        if self.pipelines.contains_key(&String::from(name)){
            return true
        }
        false
    }

    ///Returns a pipeline by name, if not existend, returns the default pipeline
    pub fn get_pipeline_by_name(&mut self, name: &str) -> Arc<pipeline::Pipeline>{
        //println!("SEARCHING FOR PIPELINE: {}", name.clone() );
        match self.pipelines.get_mut(&String::from(name)){
            Some(ref mut pipe) => return pipe.clone(),
            None => rt_error("PIPELINE_MANAGER","Could not find pipe"),
        }
        panic!("Paniced because we could not find the correct pipeline!")
    }

    ///Adds a pipeline based on a name, configuration. Returns the created pipeline
    pub fn add_pipeline(&mut self, name: &str, config: pipeline_builder::PipelineConfig)
     -> Arc<pipeline::Pipeline>{
        //first of all make a unique name
        let unique_name = {
            if self.pipelines.contains_key(&name.to_string()){
                //cycle through the names
                let mut index = 0;
                while self.pipelines.contains_key(&(name.to_string() + "_" + &index.to_string())){
                    index += 1;
                }
                name.to_string() + "_" + &index.to_string()
            }else{
                name.to_string()
            }


        };

        let (pass, shader_set) = (config.render_pass.clone(), config.shader_set.clone());
        let (ren_pass, subpass_id) = self.get_passes().conf_to_pass(pass);

        let pipe = pipeline::Pipeline::new(
            self.device.clone(),
            config,
            ren_pass,
            subpass_id,
            self.shader_manager.get_shader_set(shader_set)
            .expect("failed to get correct shader set for pipeline... set a right one!")
        );

        let arc_pipe = Arc::new(pipe);
        println!("Adding pipeline with name: {} ... ", unique_name);
        self.pipelines.insert(unique_name, arc_pipe.clone());

        arc_pipe
    }

    ///Returns a pipeline which has this configuration. Creates the pipeline if needed
    pub fn get_pipeline_by_config(
        &mut self,
        needed_configuration: pipeline_builder::PipelineConfig,
    ) -> Arc<pipeline::Pipeline> {

        //first of all test the available pipelines for this config
        for (_, pipe) in self.pipelines.iter(){
            //Test for the configuration
            if pipe.pipeline_config.compare(&needed_configuration){
                //If the config matches, return this one, else create new one
                return pipe.clone()
            }
        }

        //now we create a new pipeline
        //CREATING_PIPE==============================================

        let pipe_name = self.create_pipeline_name(
            &needed_configuration.blending_operation, &needed_configuration.cull_mode
        );


        let (pass, shader_set) = (needed_configuration.render_pass.clone(), needed_configuration.shader_set.clone());
        let (render_pass, subpass_id) = self.get_passes().conf_to_pass(pass);
        //now build the new pipeline and put it in an arc for cloning
        let new_pipe = Arc::new(pipeline::Pipeline::new(
            self.device.clone(),
            needed_configuration,
            render_pass,
            subpass_id,
            self.shader_manager.get_shader_set(shader_set)
            .expect("failed to get shader set for pipeline.. that shoudn't not happen")
        ));

        self.pipelines.insert(pipe_name.clone(), new_pipe);
        //now return the new pipe
        self.get_pipeline_by_name(&pipe_name)
    }


    ///Returns a pipeline which fulfills the `requirements`. If there is none one will be created.
    /// if possible based on the `needed_configuration`. Anyways the pipeline will always be made for
    /// the `needed_subpass_id` in the main_renderpass stored in this pipeline_manager.
    ///You can provide an optional `configuration` for the new pipeline.
    pub fn get_pipeline_by_requirements(
        &mut self,
        requirements: PipelineRequirements,
    ) -> Arc<pipeline::Pipeline> {

        //cycle thorugh the pipeline and match the types of the requirements with the pipeline
        //return if they are fulfilled or create a new one if not

        //first test based on the requirements and the subpass id
        for (_, pipe) in self.pipelines.iter(){
            let current_self_req = PipelineRequirements{
                blend_type: pipe.pipeline_config.blending_operation.clone(),
                culling: pipe.pipeline_config.cull_mode.clone(),
                render_pass: pipe.pipeline_config.render_pass.clone(),
                shader_set: pipe.pipeline_config.shader_set.clone()
            };

            if current_self_req.compare(&requirements){
                println!("Found correct pipeline based on the requirements", );
                return pipe.clone();
            }
        }

        //We found no pipe with the required attributes. Thats why we'll create one with the required
        // atribs.
        let pipeline_conf = pipeline_builder::PipelineConfig::default()
        //overwrite with needed config
        .with_blending(requirements.blend_type.clone())
        .with_cull_mode(requirements.culling.clone())
        //Set some additional info
        .with_render_pass(requirements.render_pass.clone())
        .with_shader(requirements.shader_set.clone());



        //CREATING_PIPE==============================================

        self.get_pipeline_by_config(pipeline_conf)
    }

    ///A helper function to create nice pipeline names
    fn create_pipeline_name(
        &self,
        blend_type: &pipeline_builder::BlendTypes,
        cull_mode: &pipeline_builder::CullMode
    ) -> String{

        //Create a name
        let pipe_name = {
            //Create a blend string
            let blend_type_name = {
                match blend_type{
                    &pipeline_builder::BlendTypes::BlendCollective(_) =>{
                        String::from("CustomBlending")
                    },
                    &pipeline_builder::BlendTypes::BlendPassThrough =>{
                        String::from("PassThrough")
                    },
                    &pipeline_builder::BlendTypes::BlendAlphaBlending =>{
                        String::from("AlphaBlending")
                    },
                    &pipeline_builder::BlendTypes::BlendLogicOp(_) =>{
                        String::from("LogicBlending")
                    },
                }
            };
            //create a poly mode string
            let cull_mode = {
                match cull_mode{
                    &pipeline_builder::CullMode::Disabled =>{
                        String::from("NoCulling")
                    },
                    &pipeline_builder::CullMode::Front =>{
                        String::from("FrontCulling")
                    },
                    &pipeline_builder::CullMode::Back =>{
                        String::from("BackCulling")
                    },
                    &pipeline_builder::CullMode::FrontAndBack =>{
                        String::from("FrontAndBackCulling")
                    },
                }
            };

            //Now create a tmp string and decide the indice via a for loop till we found a nice one
            let tmp_string = String::from("Pipeline_") + &cull_mode + "_" + &blend_type_name;

            //check if this name already exists
            if self.pipelines.contains_key(&tmp_string){
                //cycle through by checking if the name + indice exists, if not, use it
                let mut indice = 1;
                while self.pipelines.contains_key(&(tmp_string.clone() + "_" + &indice.to_string())) {
                    indice +=1;
                }
                //should be the right one now, create the final string
                tmp_string + "_" + &indice.to_string()
            }else{
                tmp_string
            }
        };

        //Return it
        pipe_name
    }

    ///Returns the locked passes struct of this manager.
    pub fn get_passes<'a>(&'a self) -> MutexGuard<'a, RenderPasses>{
        self.passes.lock().expect("Failed to lock renderpasses")
    }
}
