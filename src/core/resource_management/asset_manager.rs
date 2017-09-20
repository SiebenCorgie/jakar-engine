use std::sync::{Mutex, Arc, MutexGuard};
use std::thread;
use time;

use core::simple_scene_system::node;
use core::resource_management::texture_manager;
use core::resource_management::material_manager;
use core::resources::light;
use core::resources::mesh;
use core::resource_management::mesh_manager;
//use tools::assimp_importer;
use tools::Importer;
use tools::gltf_importer;
use core::resource_management::scene_manager;
use core::resources::camera::Camera;
use core::resources::camera::DefaultCamera;
use core::engine_settings;
use core::resources::texture;
use core::resources::material;

use rt_error;

use render::renderer;
use render::pipeline;
use render::pipeline_manager;
use render::shader_impls::pbr_vertex;
use render::shader_impls::pbr_fragment;

use input::KeyMap;

use cgmath::*;
use vulkano;

///The main struct for the scene manager
///It is responible for handling the materials and scenes as well as the assets
pub struct AssetManager {
    ///Holds the current active scene
    active_main_scene: node::GenericNode,

    ///holds all textures
    texture_manager: Arc<Mutex<texture_manager::TextureManager>>,

    ///Holds the current material manager
    material_manager: Arc<Mutex<material_manager::MaterialManager>>,

    mesh_manager: Arc<Mutex<mesh_manager::MeshManager>>,

    scene_manager: Arc<Mutex<scene_manager::SceneManager>>,

    ///Holds a reference to the renderer
    renderer: Arc<Mutex<renderer::Renderer>>,

    ///A Debug camera, will be removed in favor of a camera_managemant system
    camera: DefaultCamera,

    settings: Arc<Mutex<engine_settings::EngineSettings>>,

    /// a copy of the keymap to be used for passing to everything gameplay related
    key_map: Arc<Mutex<KeyMap>>,

}

impl AssetManager {
    ///Creates a new idependend scene manager
    pub fn new(
        renderer: Arc<Mutex<renderer::Renderer>>,
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        key_map: Arc<Mutex<KeyMap>>,
    )->Self{

        //The camera will be moved to a camera manager
        let camera = DefaultCamera::new(settings.clone(), key_map.clone());

        //Gt us needed instances
        let device = {
            let render_lck = renderer.lock().expect("failed to hold renderer lock");
            (*render_lck).get_device().clone()
        };

        let queue = {
            let render_lck = renderer.lock().expect("failed to hold renderer lock");
            (*render_lck).get_queue().clone()
        };

        let mut tmp_texture_manager = texture_manager::TextureManager::new(
            device, queue, settings.clone()
        );

        let (fallback_alb, fallback_nrm, fallback_phy) = tmp_texture_manager.get_fallback_textures();
        let none_texture = tmp_texture_manager.get_none();

        let tmp_material_manager = material_manager::MaterialManager::new(
            renderer.clone(),
            fallback_alb,
            fallback_nrm,
            fallback_phy,
            none_texture,
        );
        let new_scene_manager = Arc::new(Mutex::new(scene_manager::SceneManager::new()));

        AssetManager{
            active_main_scene: node::GenericNode::new_empty("Empty"),
            texture_manager: Arc::new(Mutex::new(tmp_texture_manager)),
            material_manager: Arc::new(Mutex::new(tmp_material_manager)),
            mesh_manager: Arc::new(Mutex::new(mesh_manager::MeshManager::new())),
            scene_manager: new_scene_manager,
            renderer: renderer,
            camera: camera,

            settings: settings,

            key_map: key_map.clone(),
        }
    }

    ///Updates all child components
    pub fn update(&mut self){

        //Debug stuff which will be handled by the application later
        //let rotation = Rotation3::from_axis_angle(&Vector3::unit_z(), time::precise_time_ns() as f32 * 0.000000001);
        let mat_4: Matrix4<f32> = Matrix4::identity();


        let uniform_data = pbr_fragment::ty::Data {
            //Updating camera from camera transform
            camera_position: self.camera.position.clone().into(),
            _dummy0: [0; 4],
            //This is getting a dummy value which is updated right bevore set cretion via the new
            //model provided transform matrix. There might be a better way though.
            model: mat_4.into(),
            view: self.get_camera().get_view_matrix().into(),
            proj: self.get_camera().get_perspective().into(),
        };

        //in scope to prevent dead lock while updating material manager
        //TODO get the lights from the light-pre-pass
        //light counter
        let (mut c_point, mut c_dir, mut c_spot): (u32, u32, u32) = (0,0,0);
        //after getting all lights, create the shader-usable shader infos
        let point_shader_info = {

            let all_point_lights = self.active_main_scene.get_all_point_lights();

            let mut return_vec = Vec::new();
            //transform into shader infos
            for light in all_point_lights.iter(){
                //let light_inst = light.clone();
                //let light_lck = light_inst.lock().expect("failed to lock light");
                return_vec.push(light.as_shader_info());
            }

            let empty_light = pbr_fragment::ty::PointLight{
                color: [0.0; 3],
                location: [0.0; 3],
                intensity: 0.0,
                _dummy0: [0; 4],
            };
            let mut add_array = [empty_light.clone(); 6];

            let mut index = 0;
            //Todo make the bound configurable
            //configure the array to hold the forst six lights
            while (index < 6) & (index < return_vec.len()) {
                add_array[index] = return_vec[index];
                index += 1;
                c_point += 1;
            }


            pbr_fragment::ty::point_lights{
                p_light: add_array,
            }

        };

        let directional_shader_info = {
            let all_directional_lights = self.active_main_scene.get_all_directional_lights();

            let mut return_vec = Vec::new();
            //transform into shader infos
            for light in all_directional_lights.iter(){
                //let light_inst = light.clone();
                //let light_lck = light_inst.lock().expect("failed to lock light");
                return_vec.push(light.as_shader_info());
            }

            let empty_light = pbr_fragment::ty::DirectionalLight{
                color: [0.0; 3],
                direction: [1.0; 3],
                location: [0.0; 3],
                intensity: 0.0,
                _dummy0: [0; 4],
                _dummy1: [0; 4],
            };
            let mut add_array = [empty_light.clone(); 6];

            let mut index = 0;
            //Todo make the bound configurable
            //configure the array to hold the forst six lights
            while (index < 6) & (index < return_vec.len()) {
                add_array[index] = return_vec[index];
                index += 1;
                c_dir += 1;
            }

            pbr_fragment::ty::directional_lights{
                d_light: add_array,
            }
        };

        let spot_shader_info = {
            let mut return_vec = Vec::new();
            let all_spot_lights = self.active_main_scene.get_all_spot_lights();

            //transform into shader infos
            for light in all_spot_lights.iter(){
                //let light_inst = light.clone();
                //let light_lck = light_inst.lock().expect("failed to lock light");
                return_vec.push(light.as_shader_info());
            }

            let empty_light = pbr_fragment::ty::SpotLight{
                color: [0.0; 3],
                direction: [1.0; 3],
                location: [0.0; 3],
                intensity: 0.0,
                outer_radius: 0.0,
                inner_radius: 0.0,
                _dummy0: [0; 4],
                _dummy1: [0; 4],
                _dummy2: [0; 8],
            };

            let mut add_array = [empty_light.clone(); 6];

            let mut index = 0;
            //Todo make the bound configurable
            //configure the array to hold the forst six lights
            while (index < 6) & (index < return_vec.len()) {
                add_array[index] = return_vec[index];
                index += 1;
                c_spot +=1;
            }

            pbr_fragment::ty::spot_lights{
                s_light: add_array,
            }
        };

        //Update the uniform manager with the latest infos about camera and light
        {
            let render_lck = self.renderer.lock().expect("failed to lock renderer");
            let uniform_manager = (*render_lck).get_uniform_manager();
            let mut uniform_manager_lck = uniform_manager.lock().expect("failed to lock uniform_man.");
            (*uniform_manager_lck).update(
                uniform_data, point_shader_info, directional_shader_info, spot_shader_info, c_point, c_dir, c_spot
            );
        }


        //println!("STATUS: ASSET_MANAGER: Now I'll update the materials", );
        //Update materials
        self.get_material_manager().update();
        //self.material_manager.update();
        //println!("STATUS: ASSET_MANAGER: Finished materials", );

        //Now update the camera
        self.camera.update_view();
    }

    ///Returns the scene manager as a locked mutex, need to be returned
    pub fn get_scene_manager<'a>(&'a mut self) -> MutexGuard<'a, scene_manager::SceneManager>{
        //lock own manager to return borrow
        //let scene_inst = self.scene_manager.clone();
        let scene_lock = self.scene_manager.lock().expect("failed to hold lock for scene manager");
        scene_lock
    }

    ///Returns the camera in use TODO this will be managed by a independent camera manager in the future
    pub fn get_camera(&mut self) -> &mut DefaultCamera{
        &mut self.camera
    }

    ///Sets the root scene to a `new_scene_root`
    pub fn set_active_scene(&mut self, new_scene_root: node::GenericNode){
        self.active_main_scene = new_scene_root;
    }

    ///Returns a reference to the active scene
    pub fn get_active_scene(&mut self) -> &mut node::GenericNode{
        &mut self.active_main_scene
    }

    ///Starts the asset thread, responsible for managing all assets
    ///Might be removed because not neccessary
    pub fn start_asset_thread(&mut self){
        // NOTE has to be implemented
        return
    }

    //Returns a reference to the texture manager
    pub fn get_texture_manager(&mut self) -> MutexGuard<texture_manager::TextureManager>{
        self.texture_manager.lock().expect("failed to lock texture manager")
    }

    ///Returns a reference to the material manager
    pub fn get_material_manager(&mut self) -> MutexGuard<material_manager::MaterialManager>{
        self.material_manager.lock().expect("failed to hold material manager")
    }

    ///Returns the mesh manager
    pub fn get_mesh_manager(&mut self) -> MutexGuard<mesh_manager::MeshManager>{
        self.mesh_manager.lock().expect("failed to hold mesh manager")
    }

    //Returns a raw copy of the meshes in the current active scene tree
    pub fn get_all_meshes(&mut self) -> Vec<(Arc<Mutex<mesh::Mesh>>, Matrix4<f32>)>{
        self.active_main_scene.get_all_meshes()
    }

    ///Returns all meshes in the view frustum of the currently active camera
    pub fn get_meshes_in_frustum(&mut self) -> Vec<(Arc<Mutex<mesh::Mesh>>, Matrix4<f32>)>{
        self.active_main_scene.get_meshes_in_frustum(&self.camera)
    }

    ///Imports a new gltf scene file to a new scene with `name` as name from `path`
    pub fn import_gltf(&mut self, name: &str, path: &str){
        //Lock in scope to prevent dead lock while importing

        let device_inst = {
            self.renderer.lock().expect("failed to hold renderer lock").get_device().clone()
        };
        let queue_inst = {
            self.renderer.lock().expect("failed to hold renderer lock").get_queue().clone()
        };
        let uniform_manager_inst = {
            self.renderer.lock().expect("failed to hold renderer lock").get_uniform_manager().clone()
        };
        let pipeline_manager_inst = {
            self.renderer.lock().expect("failed to hold renderer lock").get_pipeline_manager().clone()
        };


        use core;
        let managers = Arc::new(Mutex::new(core::resource_management::ManagerAndRenderInfo{
            ///The current pipeline manager
            pipeline_manager: pipeline_manager_inst,
            ///The current uniform manager
            uniform_manager: uniform_manager_inst,
            ///The current device used for rendering
            device: device_inst,
            ///The currently used queues
            queue: queue_inst,
            ///The current texture manager
            texture_manager: self.texture_manager.clone(),
            ///The current material manager
            material_manager: self.material_manager.clone(),
            ///The current mesh manager
            mesh_manager: self.mesh_manager.clone(),
            ///The current scene manager
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

    ///Imports a new scene from a file at `path` and saves the scene as `name`
    ///The meshes are stored as Arc<Mutex<T>>'s in the mesh manager the scene Is stored in the scene manager
    pub fn import_scene(&mut self, name: &str, path: &str){
        //Lock in scope to prevent dead lock while importing

        let device_inst = {
            self.renderer.lock().expect("failed to hold renderer lock").get_device().clone()
        };
        let queue_inst = {
            self.renderer.lock().expect("failed to hold renderer lock").get_queue().clone()
        };

        //Create the topy scene via an empty which will be used to add all the meshe-nodes
        let new_scene = node::GenericNode::new_empty(name.clone());
        //Add the new scene to the manager
        self.get_scene_manager().add_scene(new_scene);
        //Now get the scene in the manager (as Arc<T>) and pass it for adding new meshes
        let scene_in_manager = self.get_scene_manager().get_scene_arc(
            name.clone()
        ).expect("could not find the just added scene, this should not happen");

        //Pass the import params an a scene manager instance to the mesh manager
        self.get_mesh_manager().import_mesh(
            name, path,
            device_inst, queue_inst,
            scene_in_manager
        );

    }

    ///Adds a scene from the local scene manager (based on `name`) to the local main scene
    pub fn add_scene_to_main_scene(&mut self, name: &str){

        //Get the scene
        let scene ={
            self.get_scene_manager().get_scene_arc(name).clone()
        };

        match scene{
            Some(sc) =>{
                //TODO make this to an Arc<GenericNode>
                let scene_lck = sc.lock().expect("failed to hold scene lock while adding");
                //Create a pass it to the main scene TODO make this reference the old scene
                self.active_main_scene.add_node((*scene_lck).clone());
            },
            None => rt_error("ASSET_MANAGER", &("Could not find scene with name".to_string() + name.clone()).to_string()),
        }

        //finally rebuild bounds
        self.get_active_scene().rebuild_bounds();
    }

    ///Returns true if a scene with `name` as name exists in the local scene manager
    pub fn has_scene(&mut self, name: &str) -> bool{
        let scene_manager = self.get_scene_manager();
        scene_manager.has_scene(name.clone())
    }

    ///Returns a texture builder for the specified image at `path`
    pub fn create_texture(&mut self, path: &str) -> texture::TextureBuilder{

        //lock the renderer
        let  render_lck = self.renderer.lock().expect("failed to hold renderer");

        //Create a second material
        //create new texture
        let new_texture = texture::TextureBuilder::from_image(
            path,
            (*render_lck).get_device(),
            (*render_lck).get_queue(),
            self.settings.clone()
        );
        new_texture
    }

    ///Takes a `texture::TextureBuilder` and adds the texture by `name` to the texture manager.
    ///builds the texture and adds it to the internal manager,
    /// returns an error if the texture already exists
    pub fn add_texture_to_manager(
        &mut self, texture_builder: texture::TextureBuilder, tex_name: &str
    ) -> Result<(), &'static str>
    {
        let final_texture = texture_builder.build_with_name(tex_name);
        self.get_texture_manager().add_texture(final_texture)
    }

    ///Takes an `material::MaterialBuilder` as well as the `name` for the new material
    ///and adds it to the internam manager
    pub fn add_material_to_manager(&mut self, material: material::MaterialBuilder, name: &str)
    -> Result<String, String>
    {
        //lock the renderer
        let render_inst = self.renderer.clone();
        let  render_lck = render_inst.lock().expect("failed to hold renderer");
        let (pipe, uni_man, device) = (*render_lck).get_material_instances();

        let final_material = material.build(
            name,
            pipe,
            uni_man,
            device,
        );

        self.get_material_manager().add_material(final_material)
    }

    ///A small helper function which returns the used engine settings, good if you have to transport
    ///much data between function
    pub fn get_settings(&self) -> Arc<Mutex<engine_settings::EngineSettings>>{
        self.settings.clone()
    }

}


//Created a mesh manager holding all meshes as Arc<Mutex<T>>
//a scenen manger who holdes sub scenes created form imports as well as user created scenes
//The asset manager holds only a currently active scene, know as the player level
