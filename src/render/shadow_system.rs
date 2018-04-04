use vulkano;
use vulkano::image::AttachmentImage;
use vulkano::format::Format;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::pipeline::GraphicsPipelineAbstract;

use std::sync::{Arc, Mutex};

use collision::Frustum;

use core::resources::camera::Camera;
use core::resources::light;
use core::next_tree::{SceneTree, SceneComparer, ValueTypeBool};
use core::next_tree::SaveUnwrap;
use core::resource_management::asset_manager;
use core::engine_settings::EngineSettings;
use render::frame_system::FrameStage;
use render::frame_system::FrameSystem;
use render::pipeline::Pipeline;
use render::shader::shader_inputs::default_data::ty::LightData;


use jakar_tree::node::Node;
use core::next_tree::content::ContentType;
use core::next_tree::attributes::NodeAttributes;
use core::next_tree::jobs::SceneJobs;

//the shader infors we return
use render::shader::shader_inputs::lights::ty::{PointLight, SpotLight, DirectionalLight};


///Stores the data related to shadow creation.
pub struct ShadowSystem {
    engine_settings: Arc<Mutex<EngineSettings>>,

    shadow_pipeline: Arc<Pipeline>,
    data_buffer_pool: CpuBufferPool<LightData>,
    data_deciptor_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>
}

impl ShadowSystem{
    ///Creates a new shadow system. Currently only abel to create the depth map of one directional
    /// light.
    pub fn new(
        device: Arc<vulkano::device::Device>,
        engine_settings: Arc<Mutex<EngineSettings>>,
        shadow_pipeline: Arc<Pipeline>,
    ) -> Self{

        let pool = CpuBufferPool::uniform_buffer(device.clone());
        let descriptor_pool = FixedSizeDescriptorSetsPool::new(shadow_pipeline.get_pipeline_ref().clone(), 0);

        ShadowSystem{
            engine_settings: engine_settings,
            shadow_pipeline: shadow_pipeline,
            data_buffer_pool: pool,
            data_deciptor_pool: descriptor_pool,

        }
    }

    ///updates the information for which light which shadow is calculated
    pub fn set_shadow_atlases(
        &mut self,
        asset_manager: &mut asset_manager::AssetManager,
        point_lights: Vec<Node<ContentType, SceneJobs, NodeAttributes>>,
        spot_lights: Vec<Node<ContentType, SceneJobs, NodeAttributes>>,
        directional_lights: Vec<Node<ContentType, SceneJobs, NodeAttributes>>
    )-> (Vec<PointLight>, Vec<SpotLight>, Vec<DirectionalLight>){
        //First of all we calculate a list from nearest to furthest light.
        // We then take calculate which lights to use as shadow, argumetns for the consideration are:
        // - Volume / Impact on the scene
        // - Distance to camera
        // - should the light cast a shadow
        // - max shadow count

        //Finally we convert all of them to shader infos, count the lights with shadows and calculate
        // an optimal atlas for each.

        //TODO actually implement all the stuff above, currently only converting to infos and returning
        let current_camera = asset_manager.get_camera();


        let point_info = {
            let mut shader_vec = Vec::new();
            for p_light in point_lights.iter(){
                let light_location = &p_light.attributes.transform.disp;
                let light = {
                    match p_light.value{
                        ContentType::PointLight(ref light) => light,
                        _ => continue, //Is no pointlight, test next
                    }
                };
                shader_vec.push(light.as_shader_info(light_location));
            }
            shader_vec
        };

        let spot_info = {
            let mut shader_vec = Vec::new();
            for s_light in spot_lights.iter(){
                let light_location = &s_light.attributes.transform.disp;
                let light_rotation = &s_light.attributes.transform.rot;
                let light = {
                    match s_light.value{
                        ContentType::SpotLight(ref light) => light,
                        _ => continue, //Is no pointlight, test next
                    }
                };
                shader_vec.push(light.as_shader_info(light_rotation, light_location));
            }

            shader_vec
        };

        let directional_info = {

            let mut is_first_dir = true;

            let dir_settings = {
                let mut set_lck = self.engine_settings.lock().expect("Failed to lock settings");
                set_lck.get_render_settings().get_light_settings().directional_settings.clone()
            };

            let mut shader_vec = Vec::new();
            for d_light in directional_lights.iter(){
                let light_rotation = &d_light.attributes.transform.rot;
                let light = {
                    match d_light.value{
                        ContentType::DirectionalLight(ref light) => light,
                        _ => {
                            continue; //Is no pointlight, test next
                        }
                    }
                };

                let region = {
                    if is_first_dir {
                        is_first_dir = false;
                        [0.0, 0.0, 1.0, 1.0]
                    }else{
                        [0.0; 4]
                    }
                };

                shader_vec.push(light.as_shader_info(
                    light_rotation,
                    &current_camera,
                    dir_settings.pcf_samples,
                    region
                )); //shadow region will be set by the shadow system later if needed
            }

            shader_vec
        };

        (point_info, spot_info, directional_info)


    }

    pub fn render_shadows(
        &mut self,
        command_buffer: FrameStage,
        frame_system: &FrameSystem,
        asset_manager: &mut asset_manager::AssetManager,
    ) -> FrameStage{

        match command_buffer{
            FrameStage::Shadow(cb) =>{

                let mut new_cb = cb;

                //first of all get all directional lights
                let camera_pos = asset_manager.get_camera().clone();

                let scene = asset_manager.get_active_scene();
                //only use the first directional light
                println!("Getting firectional light", );
                let directional_light_name = {
                    let vec = scene.get_all_names(
                        &Some(SceneComparer::new()
                        .with_value_type(
                            ValueTypeBool::none().with_directional_light()
                        )));
                    if vec.len() == 0{
                        println!("No shadow here lel", );
                        return FrameStage::Shadow(new_cb);
                    }
                    vec[0].clone()
                };
                //now extract the data set from it and pass it to the shader after getting all meshe in the lights
                // frustum.
                let mut light_node = scene.get_node(&directional_light_name).expect("didn't find the dir light").clone();

                let light = light_node.value.as_directional_light().expect("failed to unwrap directional light node");

                let light_mvp = light.get_mvp(&light_node.attributes.transform.rot, &camera_pos);
                let view_frustum = Frustum::from_matrix4(light_mvp).expect("failed to create ortho frustum");

                let meshes_in_light_frustum = scene
                .copy_all_nodes(&Some(
                    SceneComparer::new()
                    .with_frustum(view_frustum)
                    .with_value_type(ValueTypeBool::none().with_mesh()
                )));
                //now get the dynamic stuff for the shadows
                //After all, create the frame dynamic states
                let img_dim = frame_system.shadow_images.directional_shadows.dimensions();
                let dynamic_state = vulkano::command_buffer::DynamicState{
                    line_width: None,
                    viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [img_dim[0] as f32, img_dim[1] as f32],
                        depth_range: 0.0 .. 1.0,
                    }]),
                    scissors: None,
                };


                for node in meshes_in_light_frustum.into_iter(){

                    //get the actual mesh as well as its pipeline an create the descriptor sets
                    let mesh_locked = match node.value{
                        ContentType::Mesh(ref mesh) => mesh.clone(),
                        _ => {
                            println!("No Mesh!", );
                            continue
                        }, //is no mesh :(
                    };
                    let mesh = mesh_locked.lock().expect("failed to lock mesh in cb creation");

                    let mesh_transform = node.attributes.get_matrix();

                    let data = LightData{
                        model: mesh_transform.into(),
                        viewproj: light_mvp.into(),
                    };

                    let data_buffer = self.data_buffer_pool.next(data).expect("failed to allocate buffer");
                    let descriptor = self.data_deciptor_pool.next()
                    .add_buffer(data_buffer).expect("failed to add data buffer")
                    .build().expect("failed to build data descriptorset");

                    //println!("Drawing shadow mesh");
                    new_cb = new_cb.draw_indexed(
                        self.shadow_pipeline.get_pipeline_ref(),
                        dynamic_state.clone(),
                        mesh.get_vertex_buffer(),
                        mesh.get_index_buffer(),
                        (descriptor),
                        ()
                    ).expect("failed to draw mesh in directional depth pass");
                    //println!("Finished Draw");
                }



                return FrameStage::Shadow(new_cb);
            }
            _ => println!("wrong frame stage, not shadow!", ),
        }

        command_buffer
    }

    fn render_directional_light_map(&self, light: light::LightDirectional){
        //IMPLEMENT
    }
}
