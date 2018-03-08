use std::sync::{Mutex, Arc, MutexGuard};
use std::thread;
use std::time::Instant;

use jakar_tree::*;
use jakar_tree::node::Attribute;
use core::next_tree::content;
use core::next_tree::attributes;
use core::next_tree::jobs;
use core::next_tree;
use core::next_tree::SceneTree;


use core::resource_management::texture_manager;
use core::resource_management::material_manager;
use core::resource_management::mesh_manager;
use tools::gltf_importer;
use core::resource_management::scene_manager;
use core::resources::camera::Camera;
use core::resources::camera::DefaultCamera;
use core::engine_settings;
use core::resources::texture;
use core::resources::material;
use core::resources::empty;
use render;

use rt_error;

use render::uniform_manager;
use render::pipeline_manager;
use render::shader::shader_inputs::default_data;

use input::KeyMap;

use cgmath::*;
use vulkano;

///The main struct for the scene manager
///It is responible for handling the materials and scenes as well as the assets
#[derive(Clone)]
pub struct AssetManager {
    //Holds the current active scene
    active_main_scene: tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>,

    //holds all textures
    texture_manager: Arc<Mutex<texture_manager::TextureManager>>,

    //Holds the current material manager
    material_manager: Arc<Mutex<material_manager::MaterialManager>>,
    //hold all meshes
    mesh_manager: Arc<Mutex<mesh_manager::MeshManager>>,
    //hoolds all scenes
    scene_manager: Arc<Mutex<scene_manager::SceneManager>>,

    ///Holds a reference to the renderer
    //renderer: Arc<Mutex<renderer::Renderer>>,
    //things needed to create vulkano dependend data like textures and materials
    pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,


    ///A Debug camera, will be removed in favor of a camera_managemant system
    camera: DefaultCamera,

    settings: Arc<Mutex<engine_settings::EngineSettings>>,

    /// a copy of the keymap to be used for passing to everything gameplay related
    key_map: Arc<Mutex<KeyMap>>,

}

impl AssetManager {

    ///Creates a new idependend scene manager
    pub fn new(
        pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
        uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        key_map: Arc<Mutex<KeyMap>>,
    )->Self{


        //The camera will be moved to a camera manager
        let camera = DefaultCamera::new(settings.clone(), key_map.clone());

        //Start up the texture manager
        let mut tmp_texture_manager = texture_manager::TextureManager::new(
            device.clone(), queue.clone(), settings.clone()
        );
        //add the fallback textures
        let (fallback_alb, fallback_nrm, fallback_phy) = tmp_texture_manager.get_fallback_textures();
        let none_texture = tmp_texture_manager.get_none();
        //create a fallback material
        let tmp_material_manager = material_manager::MaterialManager::new(
            &pipeline_manager,
            &device,
            &uniform_manager,
            fallback_alb,
            fallback_nrm,
            fallback_phy,
            none_texture,
        );



        //create a empty scene manager
        let new_scene_manager = Arc::new(Mutex::new(scene_manager::SceneManager::new()));


        //create an empty main scene
        //the empty
        let empty = empty::Empty::new("main_root");
        let root_node = content::ContentType::Empty(empty);
        let main_scene = tree::Tree::new(root_node, attributes::NodeAttributes::default());

        AssetManager{
            active_main_scene: main_scene,
            texture_manager: Arc::new(Mutex::new(tmp_texture_manager)),
            material_manager: Arc::new(Mutex::new(tmp_material_manager)),
            mesh_manager: Arc::new(Mutex::new(mesh_manager::MeshManager::new())),
            scene_manager: new_scene_manager,

            pipeline_manager: pipeline_manager,
            device: device,
            queue: queue,
            uniform_manager: uniform_manager,

            camera: camera,

            settings: settings,
            key_map: key_map.clone(),
        }
    }


    ///Updates all child components
    pub fn update(&mut self){

        let (mut time_stamp, start_time, should_cap) = {
            let set_lck = self.settings.lock().expect("failed to lock engine settings");
            let sh_cap = set_lck.capture_frame.clone();
            let time_step = Instant::now();

            (time_step, Instant::now(), sh_cap)
        };

        let (far, near) = {
            let set_lck = self.settings.lock().expect("Failed to settings");
            (set_lck.camera.far_plane.clone(), set_lck.camera.near_plane.clone())
        };

        //figure out the main uniform matrix to be sent to the shaders
        let mat_4: Matrix4<f32> = Matrix4::identity();
        let uniform_data = default_data::ty::Data {
            //Updating camera from camera transform
            camera_position: self.camera.position.clone().into(),
            _dummy0: [0; 4],
            //This is getting a dummy value which is updated right bevore set creation via the new
            //model provided transform matrix. There might be a better way though.
            model: mat_4.into(),
            view: self.get_camera().get_view_matrix().into(),
            proj: self.get_camera().get_perspective().into(),
            near: near,
            far: far,
        };

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to setup uniform DATA",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            time_stamp = Instant::now()
        }

        let light_comparer = next_tree::SceneComparer::new().with_frustum(self.camera.get_frustum_bound());


        //TODO only update lights when scene changes
        let point_shader_infos = {
            let all_point_light_nodes = self.active_main_scene.copy_all_point_lights(&None);
            let mut shader_vec = Vec::new();
            for p_light in all_point_light_nodes.iter(){
                let light_location = &p_light.attributes.transform.disp;
                let light = {
                    match p_light.value{
                        next_tree::content::ContentType::PointLight(ref light) => light,
                        _ => {
                            continue; //Is no pointlight, test next
                        }
                    }
                };
                shader_vec.push(light.as_shader_info(light_location));
            }
            shader_vec
        };

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to get point lights in frustum",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            time_stamp = Instant::now()
        }

        let spot_shader_infos = {
            let all_spot_light_nodes = self.active_main_scene.copy_all_spot_lights(&Some(light_comparer));
            let mut shader_vec = Vec::new();
            for s_light in all_spot_light_nodes.iter(){
                let light_location = &s_light.attributes.transform.disp;
                let light_rotation = &s_light.attributes.transform.rot;
                let light = {
                    match s_light.value{
                        next_tree::content::ContentType::SpotLight(ref light) => light,
                        _ => {
                            continue; //Is no pointlight, test next
                        }
                    }
                };
                shader_vec.push(light.as_shader_info(light_rotation, light_location));
            }
            shader_vec
        };

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to get spot lights",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            time_stamp = Instant::now()
        }

        let dir_shader_infos = {
            let all_dir_light_nodes = self.active_main_scene.copy_all_directional_lights(&None);
            let mut shader_vec = Vec::new();
            for d_light in all_dir_light_nodes.iter(){
                let light_rotation = &d_light.attributes.transform.rot;
                let light = {
                    match d_light.value{
                        next_tree::content::ContentType::DirectionalLight(ref light) => light,
                        _ => {
                            continue; //Is no pointlight, test next
                        }
                    }
                };
                shader_vec.push(light.as_shader_info(light_rotation));
            }
            shader_vec
        };

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to get directional lights",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            time_stamp = Instant::now()
        }

        //Update the uniform manager with the latest infos about camera and light
        {
            let mut uniform_manager_lck = self.uniform_manager.lock().expect("failed to lock uniform_man.");
            uniform_manager_lck.set_point_lights(point_shader_infos);
            uniform_manager_lck.set_directional_lights(dir_shader_infos);
            uniform_manager_lck.set_spot_lights(spot_shader_infos);
            //Finally upadte the MVP data as well
            uniform_manager_lck.update(uniform_data);
        }

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to update unform manager",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            time_stamp = Instant::now()
        }


        //println!("STATUS: ASSET_MANAGER: Now I'll update the materials", );
        //Update materials
        self.get_material_manager().update();
        //self.material_manager.update();
        //println!("STATUS: ASSET_MANAGER: Finished materials", );

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to update material manager",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            time_stamp = Instant::now()
        }

        //Now update the camera
        self.camera.update_view();

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to update view in camera",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            time_stamp = Instant::now()
        }

        //and finally update the tree
        self.active_main_scene.update();

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to update active scene",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            time_stamp = Instant::now()
        }
        //also update the bounds for the current scene.
        self.active_main_scene.rebuild_bounds();

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to rebuild bounds of active scene",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            //time_stamp = Instant::now()
            println!(
                "\t \t AS: needed {}ms for asset manager update",
                start_time.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
        }

    }

    ///Returns the scene manager as a locked mutex, need to be returned
    #[inline]
    pub fn get_scene_manager<'a>(&'a mut self) -> MutexGuard<'a, scene_manager::SceneManager>{
        //lock own manager to return borrow
        //let scene_inst = self.scene_manager.clone();
        let scene_lock = self.scene_manager.lock().expect("failed to hold lock for scene manager");
        scene_lock
    }

    ///Returns the camera in use TODO this will be managed by a independent camera manager in the future
    #[inline]
    pub fn get_camera(&mut self) -> &mut DefaultCamera{
        &mut self.camera
    }

    ///Sets the root scene to a `new_scene_root`
    #[inline]
    pub fn set_active_scene(
        &mut self,
        new_scene_root: tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>
    ){
        self.active_main_scene = new_scene_root;
    }

    ///Returns a reference to the active scene
    #[inline]
    pub fn get_active_scene(&mut self)
     -> &mut tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>{
        &mut self.active_main_scene
    }

    //Returns a reference to the texture manager
    #[inline]
    pub fn get_texture_manager(&mut self) -> MutexGuard<texture_manager::TextureManager>{
        self.texture_manager.lock().expect("failed to lock texture manager")
    }

    ///Returns a reference to the material manager
    #[inline]
    pub fn get_material_manager(&mut self) -> MutexGuard<material_manager::MaterialManager>{
        self.material_manager.lock().expect("failed to hold material manager")
    }

    ///Returns the mesh manager
    #[inline]
    pub fn get_mesh_manager(&mut self) -> MutexGuard<mesh_manager::MeshManager>{
        self.mesh_manager.lock().expect("failed to hold mesh manager")
    }

    ///Returns a raw copy of the meshes in the current active scene tree. They can be sorted by
    /// `Some(attributes)` or if no sorting is needed by `None`.
    #[inline]
    pub fn copy_all_meshes(
        &mut self, mesh_parameter: Option<next_tree::SceneComparer>
    ) -> Vec<node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        self.active_main_scene.copy_all_meshes(&mesh_parameter)
    }

    ///Returns all meshes in the view frustum of the currently active camera
    #[inline]
    pub fn get_meshes_in_frustum(
        &mut self, sort_options: Option<next_tree::SceneComparer>
    ) -> Vec<node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        self.active_main_scene.copy_all_meshes_in_frustum(&self.camera, &sort_options)
    }

    ///Imports a new gltf scene file to a new scene with `name` as name from `path`
    pub fn import_gltf(&mut self, name: &str, path: &str){
        //Lock in scope to prevent dead lock while importing

        let device_inst = {
            self.device.clone()
        };
        let queue_inst = {
            self.queue.clone()
        };
        let uniform_manager_inst = {
            self.uniform_manager.clone()
        };
        let pipeline_manager_inst = {
            self.pipeline_manager.clone()
        };


        use core;
        let managers = Arc::new(Mutex::new(core::resource_management::ManagerAndRenderInfo{
            //The current pipeline manager
            pipeline_manager: pipeline_manager_inst,
            //The current uniform manager
            uniform_manager: uniform_manager_inst,
            //The current device used for rendering
            device: device_inst,
            //The currently used queues
            queue: queue_inst,
            //The current texture manager
            texture_manager: self.texture_manager.clone(),
            //The current material manager
            material_manager: self.material_manager.clone(),
            //The current mesh manager
            mesh_manager: self.mesh_manager.clone(),
            //The current scene manager
            scene_manager: self.scene_manager.clone(),
        }));

        let path_inst = path.to_owned();
        let name_inst = name.to_owned();
        //now spawn a thread to load the gltf model
        let _ = thread::spawn(move || {
            gltf_importer::import_gltf(
                &path_inst,
                &name_inst,
                managers,
            );
        });
    }


    ///Adds a scene from the local scene manager (based on `name`) to the local main scene
    /// at the `_root` node. If you want to add it at a specific node, do it like this:
    /// `get_active_scene().join(tree, node_name);`
    pub fn add_scene_to_main_scene(&mut self, name: &str)
     -> Result<(), tree::NodeErrors>
     {

        //Get the scene
        let scene ={
            self.get_scene_manager().get_scene_arc(name).clone()
        };

        match scene{
            Some(sc) =>{
                //TODO make this to an Arc<GenericNode>
                let scene_lck = sc.lock().expect("failed to hold scene lock while adding");
                //Create a pass it to the main scene TODO make this reference the old scene
                match self.active_main_scene.join_at_root(&(*scene_lck).clone()){
                    Ok(_) => {},
                    Err(r) => {
                        return Err(r);
                    },
                }
            },
            None => {
                return Err(tree::NodeErrors::NoNodeFound("Could not find the parent node".to_string()));
            },
        }

        //finally rebuild bounds
        self.get_active_scene().rebuild_bounds();
        Ok(())
    }

    ///Returns true if a scene with `name` as name exists in the local scene manager
    #[inline]
    pub fn has_scene(&mut self, name: &str) -> bool{
        let scene_manager = self.get_scene_manager();
        scene_manager.has_scene(name.clone())
    }

    ///Returns a texture builder for the specified image at `path`
    pub fn create_texture(&mut self, path: &str) -> texture::TextureBuilder{

        //lock the renderer

        //Create a second material
        //create new texture
        let new_texture = texture::TextureBuilder::from_image(
            path,
            self.device.clone(),
            self.queue.clone(),
        );
        new_texture
    }

    ///Takes a `texture::TextureBuilder` and adds the texture by `name` to the texture manager.
    ///builds the texture and adds it to the internal manager,
    /// returns an error if the texture already exists
    #[inline]
    pub fn add_texture_to_manager(
        &mut self, texture_builder: texture::TextureBuilder, tex_name: &str
    ) -> Result<(), &'static str>
    {
        let final_texture = texture_builder.build_with_name(tex_name);
        self.get_texture_manager().add_texture(final_texture)
    }

    ///Takes an `material::MaterialBuilder` as well as the `name` for the new material
    ///and adds it to the internal manager. It assumes that this material is used on a mesh in the
    /// object pass as well as that it is opaque. It returns the name this material was actually added under.
    pub fn add_material_to_manager(&mut self, material: material::MaterialBuilder, name: &str)
    -> String
    {
        let default_pipeline = {
            let mut pipe_lck = self.pipeline_manager.lock().expect("failed to lock pipeline manager");
            //Assume that we want a material for the object pass
            let config = render::pipeline_builder::PipelineConfig::default()
            .with_shader("Pbr".to_string())
            .with_render_pass(render::render_passes::RenderPassConf::ObjectPass);
            pipe_lck.get_pipeline_by_config(config)
        };

        let final_material = material.build(
            name,
            default_pipeline,
            self.uniform_manager.clone(),
            self.device.clone(),
        );

        self.get_material_manager().add_material(final_material)
    }

    ///A small helper function which returns the used engine settings, good if you have to transport
    ///much data between function
    #[inline]
    pub fn get_settings(&self) -> Arc<Mutex<engine_settings::EngineSettings>>{
        self.settings.clone()
    }

    ///Returns a copy of the current keymap
    #[inline]
    pub fn get_keymap(&self) -> KeyMap{
        self.key_map.lock().expect("failed to lock keymap").clone()
    }

}
