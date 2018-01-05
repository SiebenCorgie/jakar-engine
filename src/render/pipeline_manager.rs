use std::collections::BTreeMap;

use rt_error;
use render::pipeline;
use render::pipeline_builder;
use render::shader_impls;

use core::engine_settings;

use std::sync::{Arc, Mutex};

use vulkano;
use vulkano::framebuffer;

///Contains the requirements an material can have for a pipeline.
/// Can be used to search for an pipeline which has thoose requirements or, if there is non, create one.
#[derive(Clone)]
pub struct PipelineRequirements {
    ///Describes how the blending should be done
    pub blend_type: pipeline_builder::BlendTypes,
    ///Describes which side of an polygone should be discarded
    pub culling: pipeline_builder::CullMode,
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

        true
    }
}


///Manages all available pipeline
pub struct PipelineManager {
    //A reference to the settings, used later to get specialistaion constants and other special settings
    engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
    //stores all the pipelines
    pipelines: BTreeMap<String, Arc<pipeline::Pipeline>>,
    //stores the renderpass used for the pipeline creation
    render_pass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
    //strores the device this pipeline is based on
    device: Arc<vulkano::device::Device>,
}

impl PipelineManager{

    ///Creates a pipeline Manager with a default pipeline, have a look at the code to see the pipeline type
    pub fn new(
        device: Arc<vulkano::device::Device>,
        l_engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
        renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
        default_subpass: u32,
    ) -> Self
    {
        let mut b_tree_map = BTreeMap::new();
        //Creates a default pipeline from a default shader

        //the default inputs (all for the best visual graphics)
        let default_pipeline = pipeline_builder::PipelineConfig::default();

        let default_pipeline = Arc::new(
            pipeline::Pipeline::new(
                device.clone(),
                default_pipeline,
                l_engine_settings.clone(),
                framebuffer::Subpass::from(renderpass.clone(), default_subpass).expect("failed to create subpass from renderpass"),
                default_subpass
            )
        );

        b_tree_map.insert(String::from("DefaultPipeline"), default_pipeline);


        PipelineManager{
            engine_settings: l_engine_settings,
            pipelines: b_tree_map,
            render_pass: renderpass,
            device: device,
        }
    }

    ///Should always return the normal PBR pipeline, if it panics, please file a bug report, this should not happen
    pub fn get_default_pipeline(&mut self) -> Arc<pipeline::Pipeline>{

        match self.pipelines.get_mut(&String::from("DefaultPipeline")){
            Some(pipe) => return pipe.clone(),

            None =>rt_error("PIPELINE_MANAGER", "PIPELINE MANAGER: Could not find default pipe this should not happen"),
        }
        panic!("Crash could not get default pipeline!")
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
        self.get_default_pipeline()
    }

    ///Adds a pipeline based on a name, configuration and target subpass. Returns the created pipeline
    pub fn add_pipeline(&mut self, name: &str, config: pipeline_builder::PipelineConfig, subpass_id: u32)
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

        let pipe = pipeline::Pipeline::new(
            self.device.clone(),
            config,
            self.engine_settings.clone(),
            framebuffer::Subpass::from(self.render_pass.clone(), subpass_id)
                .expect("failed to get subpass at pipeline creation"),
            subpass_id
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
        device: Arc<vulkano::device::Device>,
        needed_subpass_id: u32
    ) -> Arc<pipeline::Pipeline> {

        //first of all test the available pipelines for this config
        for (_, pipe) in self.pipelines.iter(){
            //Test for the configuration
            if pipe.pipeline_config.compare(&needed_configuration){
                //test the subpass
                if pipe.sub_pass == needed_subpass_id{
                    println!("Found the right pipeline based on the supplied config!", );
                    return pipe.clone()
                }
            }
        }

        //now we create a new pipeline
        //CREATING_PIPE==============================================

        let pipe_name = self.create_pipeline_name(
            &needed_configuration.blending_operation, &needed_configuration.cull_mode
        );

        //now build the new pipeline and put it in an arc for cloning
        let new_pipe = Arc::new(pipeline::Pipeline::new(
            device,
            needed_configuration,
            self.engine_settings.clone(),
            framebuffer::Subpass::from(self.render_pass.clone(), needed_subpass_id)
                .expect("failed to get subpass at pipeline creation"),
            needed_subpass_id
        ));

        self.pipelines.insert(pipe_name.clone(), new_pipe);
        //now return the new pipe
        self.get_pipeline_by_name(&pipe_name)


    }


    ///Returns a pipeline which fulfills the `requirements`. If there is none one will be created.
    /// if possible based on the `needed_configuration`. Anyways the pipeline will always be made for
    /// the `needed_subpass_id` in the renderpass stored in this pipeline_manager.
    ///You can provide an optional `configuration` for the new pipeline.
    pub fn get_pipeline_by_requirements(
        &mut self,
        requirements: PipelineRequirements,
        device: Arc<vulkano::device::Device>,
        needed_subpass_id: u32
    ) -> Arc<pipeline::Pipeline> {

        //cycle thorugh the pipeline and match the types of the requirements with the pipeline
        //return if they are fulfilled or create a new one if not

        //first test based on the requirements and the subpass id
        for (_, pipe) in self.pipelines.iter(){
            let current_self_req = PipelineRequirements{
                blend_type: pipe.pipeline_config.blending_operation.clone(),
                culling: pipe.pipeline_config.cull_mode.clone(),
            };

            if current_self_req.compare(&requirements){
                if pipe.sub_pass == needed_subpass_id{
                    println!("Found correct pipeline based on the requirements", );
                    return pipe.clone();
                }
            }
        }

        //We found no pipe with the required attributes. Thats why we'll create one with the required
        // atribs.
        let mut pipeline_conf = pipeline_builder::PipelineConfig::default();
        //get two variables to get the name and overwerite the default pipeline conf if needed
        let (blend_type, cull_mode) = {
            //overwrite with needed config
            pipeline_conf = pipeline_conf.with_blending(requirements.blend_type.clone());
            pipeline_conf = pipeline_conf.with_cull_mode(requirements.culling.clone());
            (requirements.blend_type, requirements.culling)
        };

        //CREATING_PIPE==============================================

        //now build the new pipeline and put it in an arc for cloning
        let new_pipe = Arc::new(pipeline::Pipeline::new(
            device,
            pipeline_conf,
            self.engine_settings.clone(),
            framebuffer::Subpass::from(self.render_pass.clone(), needed_subpass_id)
                .expect("failed to get subpass at pipeline creation"),
            needed_subpass_id
        ));

        let pipe_name = self.create_pipeline_name(&blend_type, &cull_mode);

        self.pipelines.insert(pipe_name.clone(), new_pipe);
        //now return the new pipe
        self.get_pipeline_by_name(&pipe_name)
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
}
