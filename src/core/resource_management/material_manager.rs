use std::sync::{Mutex,Arc};
use std::collections::BTreeMap;
use core::resources::material;
use render::uniform_manager;
use render::pipeline_manager;
use render::render_passes::{RenderPassConf, ObjectPassSubPasses};
use core::resources::texture::Texture;

use vulkano;
use render;

///Handles all available materials
pub struct MaterialManager {
    //TODO comapare if a Vec<material> + search algorith would be faster
    material_vault: BTreeMap<String, Arc<Mutex<material::Material>>>,
    //renderer_inst: Arc<Mutex<renderer::Renderer>>,
}

impl MaterialManager {
    ///Creates the manager with a default `fallback` material.
    ///The fallback textures have to be passed to the material though
    ///for a good performance, the none texture should be as small as possible.
    ///For instance a black 1x1 pixel.
    pub fn new(
        pipeline_manager: &Arc<Mutex<pipeline_manager::PipelineManager>>,
        device: &Arc<vulkano::device::Device>,
        uniform_manager: &Arc<Mutex<uniform_manager::UniformManager>>,
        albedo_texture: Arc<Texture>,
        normal_texture: Arc<Texture>,
        physical_texture: Arc<Texture>,
        none_texture: Arc<Texture>,

    )->Self{

        let mut tmp_map = BTreeMap::new();
        let default_mat = material::MaterialBuilder::new(
            Some(albedo_texture),
            Some(normal_texture),
            Some(physical_texture),
            None,
            None,
            none_texture,
        );

        //materials can only be used in the object pass, there for we create a default pipeline
        // used for the material.
        let pipe = {
            let mut pipe_lock = pipeline_manager.lock().expect("failed to lock pipe man");
            let config = render::pipeline_builder::PipelineConfig::default()
            .with_shader("Pbr".to_string())
            .with_render_pass(RenderPassConf::ObjectPass(ObjectPassSubPasses::ForwardRenderingPass));
            pipe_lock.get_pipeline_by_config(config)
        };

        let fallback_mat = default_mat.build(
            "fallback",
            pipe,
            uniform_manager.clone(),
            device.clone());
        tmp_map.insert("fallback".to_string(), Arc::new(Mutex::new(fallback_mat)));


        MaterialManager{
            material_vault: tmp_map,
        }
    }

    ///Updates all materials
    pub fn update(&mut self){
        //println!("STATUS: MATERIAL_MANAGER: In material manager", );
        for (_ ,i) in self.material_vault.iter_mut(){
            let mut i_lck = i.lock().expect("failed to lock material for updating");
            //println!("STATUS: MATERIAL_MANAGER: Updating: {}", k);
            (*i_lck).update();
        }
    }

    ///Returns the default material of the engine
    pub fn get_default_material(&mut self) -> Arc<Mutex<material::Material>>{
        self.material_vault.get(&String::from("fallback"))
        .expect("Could not find fallback material, this shouldn't happen, please report this bug")
        .clone()
    }

    ///Returns a metarial-option with this name
    pub fn get_material_by_name(&mut self, name: &str)-> Option<Arc<Mutex<material::Material>>>{
        let getter = self.material_vault.get(&String::from(name.clone()));
        match getter{
            Some(material) => return Some(material.clone()),
            None => {
                //println!("STATUS: MATERIAL_MANAGER: Could not find material: {}", name.clone());
                return None
            }
        }

    }

    ///Returns a material with this name, or the fallback if it not exists
    pub fn get_material(&mut self, name: &str) -> Arc<Mutex<material::Material>>{
        if self.material_vault.contains_key(&String::from(name.clone())){
            return self.get_material_by_name(name.clone())
                .expect("The material is in the manager, but unwraping failed")
                .clone();
        }else{
            return self.get_default_material();
        }
    }

    ///Adds a material to this manager, returns the name this material was actually added under.
    pub fn add_material(&mut self, material: material::Material) -> String{
        //check for the key TODO might be faster with a vector containing all keys
        let mut material_name = material.get_name();

        //If there is already a material with that name
        if self.material_vault.contains_key(&material_name){
            let mut material_index = 0;
            while self.material_vault.contains_key(&(material_name.clone() + "_" + &material_index.to_string())){
                material_index += 1;
            }
            //change the name to use the index
            material_name = material_name + "_" + &material_index.to_string();
            self.material_vault.insert(
                material_name.clone(),
                Arc::new(Mutex::new(material))
            );
        }else{
            //We can just add it
            self.material_vault.insert(material_name.clone(), Arc::new(Mutex::new(material)));
        }

        material_name

    }
    ///Checks for a material
    pub fn is_available(&self, name: &str) -> bool{
        self.material_vault.contains_key(&String::from(name))
    }

    ///A debuging fuction to see all materials
    pub fn print_all_materials(&mut self){
        println!("All Materials:", );
        for (k,_) in self.material_vault.iter(){
            println!("\t{}", k.clone());
        }
    }

}
