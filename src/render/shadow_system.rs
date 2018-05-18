use vulkano;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::command_buffer::AutoCommandBufferBuilder;

use std::sync::{Arc, Mutex};

use collision::Frustum;
use cgmath::*;

use core::resources::camera::Camera;
use core::next_tree::{SceneTree, SceneComparer, ValueTypeBool};
use jakar_tree::node::Node;

use core::resource_management::asset_manager;
use core::engine_settings::EngineSettings;
use render::frame_system::FrameStage;
use render::light_system::LightStore;
use render::frame_system::FrameSystem;
use render::pipeline::Pipeline;
use render::pipeline_manager::{PipelineManager};
use render::pipeline_builder;
use render::render_passes::RenderPassConf;
use render::shader::shaders::shadow_fragment::ty::MaskedInfo;
use render::shader::shader_inputs::default_data::ty::LightData;
use tools::node_tools;

use core::next_tree::content::ContentType;
use core::next_tree::attributes::NodeAttributes;
use core::next_tree::jobs::SceneJobs;

// the shader infors we return


/// Stores the data related to shadow creation.
pub struct ShadowSystem {
    engine_settings: Arc<Mutex<EngineSettings>>,

    shadow_pipeline_front_culled: Arc<Pipeline>,
    shadow_pipeline_none_culled: Arc<Pipeline>,

    data_buffer_pool: CpuBufferPool<LightData>,
    mask_buffer_pool: CpuBufferPool<MaskedInfo>,

    data_descriptor_pool_cull: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,
    data_descriptor_pool_no_cull: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>
}

impl ShadowSystem{
    /// Creates a new shadow system. Currently only abel to create the depth map of one directional
    /// light.
    pub fn new(
        device: Arc<vulkano::device::Device>,
        engine_settings: Arc<Mutex<EngineSettings>>,
        pipeline_manager: Arc<Mutex<PipelineManager>>
    ) -> Self{

        //first of all build the two pipelines
        let (default_pipe, no_cull_pipe) = {
            let mut manager_lck = pipeline_manager.lock()
            .expect("failed to lock pipeline manager for shadow pipeline creation");

            let mut config = pipeline_builder::PipelineConfig::default()
                .with_subpass_id(super::SubPassType::Shadow.get_id())
                .with_shader("Shadow".to_string())
                .with_render_pass(RenderPassConf::ShadowPass)
                .with_depth_and_stencil_settings(
                    pipeline_builder::DepthStencilConfig::SimpleDepthNoStencil
                )
                .with_cull_mode(pipeline_builder::CullMode::Front) //For better contact shadows
                .clamp_depth(true); //need for no leaking

            let default_pipe = manager_lck.get_pipeline_by_config(config.clone());

            //also one without culling for the masked materials
            config = config.with_cull_mode(pipeline_builder::CullMode::Disabled);

            let masked_pipe = manager_lck.get_pipeline_by_config(config);

            (default_pipe, masked_pipe)
        };


        let data_pool = CpuBufferPool::uniform_buffer(device.clone());
        let mask_pool = CpuBufferPool::uniform_buffer(device.clone());
        let descriptor_pool_cull = FixedSizeDescriptorSetsPool::new(default_pipe.get_pipeline_ref().clone(), 0);
        let descriptor_pool_none_cull = FixedSizeDescriptorSetsPool::new(no_cull_pipe.get_pipeline_ref().clone(), 0);

        ShadowSystem{
            engine_settings: engine_settings,

            shadow_pipeline_front_culled: default_pipe,
            shadow_pipeline_none_culled: no_cull_pipe,

            data_buffer_pool: data_pool,
            mask_buffer_pool: mask_pool,

            data_descriptor_pool_cull: descriptor_pool_cull,
            data_descriptor_pool_no_cull: descriptor_pool_none_cull,

        }
    }

    /// updates the information for which light which shadow is calculated
    pub fn set_shadow_atlases(
        &mut self,
        asset_manager: &mut asset_manager::AssetManager
    )-> LightStore {
        // First of all we calculate a list from nearest to furthest light.
        // We then take calculate which lights to use as shadow, argumetns for the consideration are:
        // - Volume / Impact on the scene
        // - Distance to camera
        // - should the light cast a shadow
        // - max shadow count

        //This struct will we send back later containing a list of each light and its shader info
        let mut light_store = LightStore::new();

        // The frustum of the current camera. Since the bound of a light is always its influence
        // radius as well we can use this info to cull not usable spot and point lights
        let current_camera = asset_manager.get_camera().clone();
        let camera_loc = asset_manager.get_camera().get_position();
        let camera_mvp = asset_manager.get_camera().get_view_projection_matrix();

        let comparer = SceneComparer::new(); //We want to be able to render 360 deg pics with the same
        //lights... not culling per frustum .with_frustum(frustum);

        let point_lights = {
            asset_manager.get_active_scene().copy_all_nodes(
                &Some(
                    comparer.clone().with_value_type(
                        ValueTypeBool::none().with_point_light()
                    )
                    .with_cull_distance(0.05, camera_mvp)
                )
            )
        };
        // Order them now
        let point_recv = node_tools::order_by_distance(point_lights, camera_loc.clone());

        let spot_lights = {
            asset_manager.get_active_scene().copy_all_nodes(
                &Some(
                    comparer.clone().with_value_type(
                        ValueTypeBool::none().with_spot_light()
                    )
                    .with_cull_distance(0.05, camera_mvp)
                )
            )
        };

        let spot_recv = node_tools::order_by_distance(spot_lights, camera_loc.clone());


        // Since directional lights see everything we always use all of them
        let directional_lights: Vec<_> = {
            asset_manager.get_active_scene().copy_all_nodes(
                &Some(
                    SceneComparer::new().with_value_type(
                        ValueTypeBool::none().with_directional_light()
                    )
                )
            )
        };

        //Get some settings info we want TODO decide if this needs to be dynamic or not...
        let dir_settings = {
            let mut set_lck = self.engine_settings.lock().expect("Failed to lock settings");
            set_lck.get_render_settings().get_light_settings().directional_settings.clone()
        };

        // While we sort the point and spot lights, we calculate the space we can occupy per
        // directional light.
        let d_light_spaces = get_dir_light_areas(
            directional_lights.len() as u32, dir_settings.get_num_cascades()
        );
        //now, iterate through ech light/ lightspace on ther directional shadowmap and
        for (region, d_light) in d_light_spaces.into_iter().zip(directional_lights.into_iter()){
            let light_rotation = d_light.get_attrib().transform.rot;
            let light = {
                match d_light.get_value(){
                    ContentType::DirectionalLight(ref light) => light.clone(),
                    _ => {
                        continue; //Is no dir light, test next
                    }
                }
            };
            //currently have only one region
            let shader_info = light.as_shader_info(
                &light_rotation,
                &current_camera,
                dir_settings.get_pcf_samples(),
                dir_settings.get_poisson_spread(),
                region
            ); //shadow region will be set by the shadow system later if needed

            //now push into the store
            light_store.directional_lights.push((d_light, shader_info));
        }


        // Finally we convert all of them to shader infos, count the lights with shadows and calculate
        // an optimal atlas for each.

        //Since the directional lights are processed, try to get the point lights
        let new_points = {
            point_recv.recv().expect("Failed to get ordered point lights!")
        };

        for p_light in new_points.into_iter(){
            let light_location = p_light.get_attrib().transform.disp;
            let light = {
                match p_light.get_value(){
                    ContentType::PointLight(ref light) => light.clone(),
                    _ => continue, //Is no pointlight, test next
                }
            };
            let shader_info = light.as_shader_info(&light_location);
            light_store.point_lights.push((p_light, shader_info));
        }
        //Same with the spot lights
        let new_spot_lights = {
            spot_recv.recv().expect("Failed to recive spot_lights")
        };

        for s_light in new_spot_lights.into_iter(){
            let light_location = s_light.get_attrib().transform.disp;
            let light_rotation = s_light.get_attrib().transform.rot;
            let light = {
                match s_light.get_value(){
                    ContentType::SpotLight(ref light) => light.clone(),
                    _ => continue, //Is no pointlight, test next
                }
            };
            let shader_info = light.as_shader_info(&light_rotation, &light_location);
            light_store.spot_lights.push((s_light, shader_info));
        }

        light_store
    }

    //TODO configure shadow pass by light which was send here.

    pub fn render_shadows(
        &mut self,
        command_buffer: FrameStage,
        frame_system: &FrameSystem,
        asset_manager: &mut asset_manager::AssetManager,
        light_store: &mut LightStore,
    ) -> FrameStage{

        match command_buffer{
            FrameStage::Shadow(cb) =>{

                let mut new_cb = cb;

                //Currently only rendering the first cascade of each directional light
                new_cb = self.render_directional_light_map(
                    new_cb,
                    light_store,
                    asset_manager,
                    frame_system
                );

                return FrameStage::Shadow(new_cb);
            }
            _ => println!("wrong frame stage, not shadow!", ),
        }

        command_buffer
    }
    //Renders all directional light in the light store to the several targets on the directional ligh
    // shadow map.
    fn render_directional_light_map(
        &mut self,
        command_buffer: AutoCommandBufferBuilder,
        light_store: &mut LightStore,
        asset_manager: &mut asset_manager::AssetManager,
        frame_system: &FrameSystem,
    ) -> AutoCommandBufferBuilder{
        //first of all get all directional lights
        let scene = asset_manager.get_active_scene();
        //declare a new cb object which will be updated per draw call
        let mut new_cb = command_buffer;
        //Find the current percentage a mesh must cover to be used
        let cover_bias = {
            let set_lck = self.engine_settings.lock().expect("failed to lock settings");
            set_lck.get_render_settings().get_light_settings().directional_settings.get_occupy_bias()
        };
        //Now for each light and its cascade, render the light
        for &mut (ref mut _light_node, ref light_info) in light_store.directional_lights.iter_mut(){
            //Get the mvp matrix of the current light from the used matrixes in the
            //light buffer
            //TODO check if thats the right indice
            let light_mvps = {
                let mut ret_vec = Vec::new();
                for idx in 0..4{
                    ret_vec.push(Matrix4::from(light_info.light_space[idx]));
                }
                ret_vec
            };
            //image dimensions
            let img_dim = {
                let tmp_dim = frame_system.shadow_images.directional_shadows.dimensions();

                [tmp_dim[0] as f32, tmp_dim[1] as f32]
            };
            //no cycle through the light cascades and render to the correct region on the image
            for (idx, cascade_mvp) in light_mvps.into_iter().enumerate(){
                let view_frustum = Frustum::from_matrix4(cascade_mvp).expect("failed to create ortho frustum");

                let meshes_in_light_frustum = scene
                .copy_all_nodes(&Some(
                    SceneComparer::new()
                    .with_frustum(view_frustum)
                    .with_value_type(ValueTypeBool::none().with_mesh())
                    .with_cull_distance(cover_bias, cascade_mvp)
                ));

                //find the current region in the directional light map to render to
                let origin = [
                    //upper corner
                    img_dim[0] * light_info.shadow_region[idx][0],
                    img_dim[1] * light_info.shadow_region[idx][1],
                ];
                //the pixels from origin to the target location
                let dim = [
                    img_dim[0] * light_info.shadow_region[idx][2] - origin[0],
                    img_dim[1] * light_info.shadow_region[idx][3] - origin[1],
                ];

                //TODO configure based on the current shadow map region
                let dynamic_state = vulkano::command_buffer::DynamicState{
                    line_width: None,
                    viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                        origin: origin,
                        dimensions: dim,
                        depth_range: 0.0 .. 1.0,
                    }]),
                    scissors: None,
                };

                //After setting each element, render the different shadow mapps
                for node in meshes_in_light_frustum.into_iter(){
                    new_cb = self.render_depth_mesh(
                        new_cb,
                        &node,
                        cascade_mvp.clone(),
                        dynamic_state.clone()
                    );
                }
            }
        }
        new_cb
    }

    //Renders a single mesh to the current active image with a depth pipeline
    #[inline]
    fn render_depth_mesh(
        &mut self,
        command_buffer: AutoCommandBufferBuilder,
        node: &Node<
            ContentType,
            SceneJobs,
            NodeAttributes
        >,
        mvp_mat: Matrix4<f32>,
        dynamic_state: vulkano::command_buffer::DynamicState,
    ) -> AutoCommandBufferBuilder {
        //get the actual mesh as well as its pipeline an create the descriptor sets
        let mesh_locked = match node.get_value(){
            ContentType::Mesh(ref mesh) => mesh.clone(),
            _ => {
                println!("No Mesh!", );
                return command_buffer;
            }, //is no mesh :(
        };
        let mesh = mesh_locked.lock().expect("failed to lock mesh in cb creation");

        let mesh_transform = node.get_attrib().get_matrix();

        let data = LightData{
            model: mesh_transform.into(),
            viewproj: mvp_mat.into(),
        };
        //get the mask info as well as the texture with the alpha values
        let(mask_info, tex_with_alpha) ={
            let material = mesh.get_material();
            let material_lck = material.lock().expect("failed to lock material");
            material_lck.get_shadow_mask_info()
        };
        //check if we should render doublesided (only for masked materials)
        let should_be_double = {
            if mask_info.b_is_masked == 1{
                true
            }else{
                false
            }
        };
        let mask_buffer = self.mask_buffer_pool.next(mask_info).expect("failed to allocate mask buffer for shadow");
        let data_buffer = self.data_buffer_pool.next(data).expect("failed to allocate buffer");

        //depending on the masked type we create different descriptorsets

        let descriptor =
        {
            //find right descriptor pool and build
            if should_be_double{
                &mut self.data_descriptor_pool_no_cull
            }else{
                &mut self.data_descriptor_pool_cull
            }.next()
            .add_buffer(data_buffer).expect("failed to add data buffer")
            .add_sampled_image(
                tex_with_alpha.get_raw_texture(),
                tex_with_alpha.get_raw_sampler()
            ).expect("Failed to add the alpha texture for shadow pass")
            .add_buffer(
                mask_buffer
            ).expect("failed to add mask buffer to shadow descriptor")
            .build().expect("failed to build data descriptorset")

        };
        //checkfor the vertex and index buffer, if there are none we won't render at all
        if let Some(vertex_buffer) = mesh.get_vertex_buffer(){
            if let Some(index_buffer) = mesh.get_index_buffer(){
                let new_cb = command_buffer.draw_indexed(
                    if should_be_double { //find right pipeline fitting to the descriptor and execute
                        &mut self.shadow_pipeline_none_culled
                    }else{
                        &mut self.shadow_pipeline_front_culled
                    }.get_pipeline_ref(),
                    dynamic_state,
                    vertex_buffer,
                    index_buffer,
                    descriptor,
                    ()
                ).expect("failed to draw mesh in directional depth pass");

                //return cb
                return new_cb
            }else{
                println!("Found no depth mesh index buffer", );
                return command_buffer;
            }
        }else{
            println!("Found no depth mesh vertex buffer", );
            return command_buffer;
        }
    }
}

/// calculates spaces for a number of directional lights in uv coords (0.0 - 1.0)
fn get_dir_light_areas(num_lights: u32, num_cascades: u32) -> Vec<[[f32; 4];4]>{
    //Since we always need squares which are unweighted at the moment, we just check how often
    //we have to power 2 to get at least the number of tiles
    let tile_count = num_lights * num_cascades;
    //println!("Need {} tiles", tile_count);
    //Doing some nth root of stuff might be correct, but is slower then trying (at least for the
    // numbers we need here).
    let mut count = 1;
    while (count * count) < tile_count {
        if tile_count == 1 {
            break;
        }
        count +=1;
    }
    //println!("{} needs a count of {} each", tile_count, count);

    //Since we got the count we need, arrange the arrays with their uv_coords
    //we cycle though the cascades of each light, assigning a set of coordinates each.
    let split_distance = 1.0 / count as f32;
    let mut lights_vec = Vec::new();
    let mut current_cascades_vec = [[0.0;4];4];
    let mut current_cascade_count = 0;

    for u in 0..count{
        for v in 0..count{
            let current_coords = [
                u as f32 *split_distance,
                v as f32 *split_distance,
                (u+1) as f32  * split_distance,
                (v+1) as f32 * split_distance,
                ];
            //push to the current vec
            current_cascades_vec[current_cascade_count] = current_coords;
            //increment and check if this was the last cascade
            current_cascade_count += 1;
            if current_cascade_count as u32 == num_cascades{
                //push the current cascade vec and create a new one, then reset the cascade counter
                lights_vec.push(current_cascades_vec);
                current_cascades_vec = [[0.0;4];4];
                current_cascade_count = 0;
            }
        }
    }

    lights_vec
}
