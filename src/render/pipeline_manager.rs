use std::collections::BTreeMap;

use rt_error;
use render::pipeline;
use render::pipeline_builder;

use std::sync::Arc;

use vulkano;

///Contains the requirements an material can have for a pipeline.
/// Can be used to search for an pipeline which has thoose requirements or, if there is non, create one.
pub struct PipelineRequirements {
    ///Describes how the blending should be done
    pub blend_type: pipeline_builder::BlendTypes,
    ///Describes which side of an polygone should be discarded
    pub culling: pipeline_builder::CullMode,
}


///Manages all available pipeline
pub struct PipelineManager {
    //stores all the pipelines
    pipelines: BTreeMap<String, Arc<pipeline::Pipeline>>,
    //stores the renderpass used for the pipeline creation
    render_pass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
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
        let default_pipeline = pipeline_builder::PipelineConfig::default(renderpass.clone());

        let default_pipeline = Arc::new(
            pipeline::Pipeline::new(device, default_pipeline)
        );

        b_tree_map.insert(String::from("DefaultPipeline"), default_pipeline);

        PipelineManager{
            pipelines: b_tree_map,
            render_pass: renderpass,
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

    ///Should always return the normal PBR pipeline, if it panics, please file a bug report, this should not happen
    pub fn get_default_pipeline(&mut self) -> Arc<pipeline::Pipeline>{

        match self.pipelines.get_mut(&String::from("DefaultPipeline")){
            Some(pipe) => return pipe.clone(),

            None =>rt_error("PIPELINE_MANAGER", "PIPELINE MANAGER: Could not find default pipe this should not happen"),
        }
        panic!("Crash could not get default pipeline!")
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

    ///Adds a pipeline made for the specified properties. Returns a the name under which this pipeline was actually
    ///created, as well as an Arc<T> clone of the vulkano-pipeline object
    ///TODO make the shader specified
    pub fn add_pipeline(&mut self, name: &str,device: Arc<vulkano::device::Device>,
        renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Send + Sync>,
    )
    {
        let pipeline_config = pipeline_builder::PipelineConfig::default(renderpass);

        let tmp_pipeline = Arc::new(
            pipeline::Pipeline::new(device, pipeline_config)
        );
        self.pipelines.insert(String::from(name), tmp_pipeline);
    }

    ///Returns a pipeline which fulfills the `requirements`. If there is none one will be created.
    ///You can provide an optional `configuration` for the new pipeline.
    pub fn get_pipeline_by_requirements(
        &mut self,
        requirements: PipelineRequirements,
        configuration: Option<pipeline_builder::PipelineConfig>,
        device: Arc<vulkano::device::Device>
    ) -> Arc<pipeline::Pipeline> {
        //cycle thorugh the pipeline and match the types of the requirements with the pipeline
        //return if they are fulfilled or create a new one if not

        for (_, pipeline) in self.pipelines.iter(){
            //there is currently only one requirement to consider, otherwise we would have
            //to add another branch.
            if pipeline.pipeline_config.blending_operation == requirements.blend_type{
                //test the culling parameter
                if pipeline.pipeline_config.cull_mode == requirements.culling{
                    return pipeline.clone()
                }
            }
        }

        //if we are here, there was no right pipeline, thats why we have to create a new one and return it
        //first of all, see if we got a config, if yes use this one and overwrite it with the requirements,
        // otherwise create a default config and overwrite it
        let mut pipeline_conf = {
            match configuration{
                Some(config) =>{
                    config
                },
                None =>{
                    pipeline_builder::PipelineConfig::default(self.render_pass.clone())
                },
            }
        };

        //clone two types needed for the name generation
        let blend_type = requirements.blend_type.clone();
        let poly_mode = pipeline_conf.polygone_mode.clone();

        //MAIN STAGE =======================================================
        //now overwrite with new values
        pipeline_conf = pipeline_conf.with_blending(requirements.blend_type);
        pipeline_conf = pipeline_conf.with_cull_mode(requirements.culling);
        //END ==============================================================

        //now build the new pipeline and put it in an arc for cloning
        let new_pipe = Arc::new(pipeline::Pipeline::new(device, pipeline_conf));
        //add it to the manager
        let pipe_name = {
            //create a nice name to indentify it later
            let blend_type_name = {
                match blend_type{
                    pipeline_builder::BlendTypes::BlendCollective(_) =>{
                        String::from("CustomBlending")
                    },
                    pipeline_builder::BlendTypes::BlendPassThrough =>{
                        String::from("PassThrough")
                    },
                    pipeline_builder::BlendTypes::BlendAlphaBlending =>{
                        String::from("AlphaBlending")
                    },
                    pipeline_builder::BlendTypes::BlendLogicOp(_) =>{
                        String::from("LogicBlending")
                    },
                }
            };

            let poly_mode = {
                match poly_mode{
                    pipeline_builder::PolygoneMode::Fill =>{
                        String::from("Filled")
                    },
                    pipeline_builder::PolygoneMode::Line(_) =>{
                        String::from("Wireframe")
                    },
                    pipeline_builder::PolygoneMode::Point =>{
                        String::from("Points")
                    },
                }
            };

            //Now create a tmp string and decide the indice via a for loop till we found a nice one
            let tmp_string = String::from("Pipeline_") + &poly_mode + "_" + &blend_type_name;

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

        self.pipelines.insert(pipe_name.clone(), new_pipe);
        //now return the new pipe
        self.get_pipeline_by_name(&pipe_name)
    }

}
