use std::sync::{Mutex, Arc, MutexGuard};
use std::thread;
use std::time::Instant;

use jakar_tree::*;
use jakar_tree::node::Attribute;
use core::next_tree::content;
use core::next_tree::attributes;
use core::next_tree::jobs;
use core::next_tree::*;
use core::next_tree::{JakarNode, SceneTree};
use core::next_tree::content::ContentType;
use core::next_tree::node_controller::camera_controller::CameraController;

use tools::engine_state_machine::AssetUpdateState;

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


use render::uniform_manager;
use render::pipeline_manager;
use render::render_passes::{RenderPassConf, ObjectPassSubPasses};
use render::shader::shader_inputs::default_data;

use input::keymap::KeyMap;

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
    //things needed to create vulkano dependend data like textures and materials
    pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,


    ///Holds the current active camera, if non is set, falls back to a custom one
    active_camera: Option<String>,
    fall_back: JakarNode,

    settings: Arc<Mutex<engine_settings::EngineSettings>>,

    /// a copy of the keymap to be used for passing to everything gameplay related
    key_map: Arc<Mutex<KeyMap>>,

    ///Documents the current state of the asset manager
    state: Arc<Mutex<AssetUpdateState>>,


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
        let mut fallback_camera_node = node::Node::new(
            ContentType::Camera(camera), attributes::NodeAttributes::default()
        );

        fallback_camera_node.set_controller(CameraController::new(key_map.clone()));

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

            active_camera: None,
            fall_back: fallback_camera_node,

            settings: settings,
            key_map: key_map.clone(),

            state: Arc::new(Mutex::new(AssetUpdateState::wait())),
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
        //Show the other system that we are working
        self.set_working();

        if should_cap{
            println!(
                "\t \t AS: needed {}ms to update unform manager",
                time_stamp.elapsed().subsec_nanos() as f32 / 1_000_000.0
            );
            time_stamp = Instant::now()
        }

        self.fall_back.update(0.0, &Vec::new());
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

        //Show the other system that we are waiting again
        self.set_waiting();

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
    pub fn get_camera(&mut self) -> &mut DefaultCamera{

        if let Some(camera_name) = self.active_camera.clone(){
            let node = self.active_main_scene.get_node(&camera_name);
            if let Some(camera_node) = node{
                if let Some(camera) = camera_node.get_value_mut().as_camera(){
                    println!("Found active camera", );
                    return camera;
                }
            }
        }
        self.fall_back.get_value_mut().as_camera().expect("failed to get camera")
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
        &mut self, mesh_parameter: Option<SceneComparer>
    ) -> Vec<node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //add the mesh attributes to the sorter if needed
        let new_sorter = {
            match mesh_parameter{
                Some(para) => para.with_value_type(ValueTypeBool::none().with_mesh()),
                None => SceneComparer::new().with_value_type(ValueTypeBool::none().with_mesh())
            }
        };
        self.active_main_scene.copy_all_nodes(&Some(new_sorter))
    }

    ///Returns all meshes in the view frustum of the currently active camera
    #[inline]
    pub fn get_meshes_in_frustum(
        &mut self, sort_options: Option<SceneComparer>
    ) -> Vec<node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        let new_sorter = {
            match sort_options{
                Some(para) => para
                    .with_value_type(ValueTypeBool::none().with_mesh())
                    .with_frustum(self.get_camera().get_frustum_bound()),
                None => SceneComparer::new()
                    .with_value_type(ValueTypeBool::none().with_mesh())
                    .with_frustum(self.get_camera().get_frustum_bound())
            }
        };
        self.active_main_scene.copy_all_nodes(&Some(new_sorter))
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
            .with_render_pass(RenderPassConf::ObjectPass(ObjectPassSubPasses::ForwardRenderingPass));
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

    ///Changes to the state of the asset manager to working
    fn set_working(&mut self){
        let mut state_lck = self.state.lock().expect("failed to lock asset manager state");
        *state_lck = AssetUpdateState::working();
    }

    ///Changes to the state of the asset manager to waiting
    fn set_waiting(&mut self){
        let mut state_lck = self.state.lock().expect("failed to lock asset manager state");
        *state_lck = AssetUpdateState::wait();
    }

    ///Returns the asset manager state
    pub fn get_asset_manager_state(&self) -> Arc<Mutex<AssetUpdateState>>{
        self.state.clone()
    }

}
