use std::sync::{Mutex,Arc};
use std::collections::BTreeMap;
use core::resources::material;
use render;
use render::pipeline;
use render::pipeline_manager;
use core::resources::texture::Texture;
use vulkano;
///Handles all available materials
pub struct MaterialManager {
    //TODO comapare if a Vec<material> + search algorith would be faster
    material_vault: BTreeMap<String, Arc<Mutex<material::Material>>>,
    renderer_inst: Arc<Mutex<render::renderer::Renderer>>,
}

impl MaterialManager {
    ///Creates the manager with a default `fallback` material.
    ///The fallback textures have to be passed to the material though
    ///for a good performance, the none texture should be as small as possible.
    ///For instance a black 1x1 pixel.
    pub fn new(
        render: Arc<Mutex<render::renderer::Renderer>>,
        albedo_texture: Arc<Texture>,
        normal_texture: Arc<Texture>,
        physical_texture: Arc<Texture>,
        none_texture: Arc<Texture>,

    )->Self{
        //We'll have to check for a default pipeline, otherwise the Manager creation could fail
        {
            let mut render_lck = render.lock().expect("Failed to lock renderer");

            let pipeline_copy = (*render_lck).get_pipeline_manager().clone();
            {
                if !(*pipeline_copy).lock()
                    .expect("Failed to lock pipeline manager in material manager creation")
                    .has_pipeline("DefaultPipeline")
                {
                    //println!("STATUS: MATERIAL_MANAGER: Oups, this programm has no default pipeline, PANIC!", );
                    panic!("this engine has no default pipeline :(");
                }
            }
        }


        //println!("STATUS: MATERIAL_MANAGER: Checked pipeline for default pipeline in material manager creation", );
        //Creates a fallback material to which the programm can fallback in case of a "materal not found"

        let (pipe, uni_man, device) ={
            let mut render_lck = render.lock().expect("Failed to lock renderer");
            (*render_lck).get_material_instances()
        };

        let engine_settings = {
            let mut render_lck = render.lock().expect("Failed to lock renderer");
            (*render_lck).get_engine_settings()
        };

        //finally create the material from the textures
        //this will serve as fallback for any unsuccessful `get_material()`
        let fallback_material = Arc::new(
            Mutex::new(
                material::MaterialBuilder::new(
                    Some(albedo_texture),
                    Some(normal_texture),
                    Some(physical_texture),
                    None, //currently no occlusion texture
                    None,
                    none_texture
                )
                .build(
                    "fallback",
                    pipe,
                    uni_man,
                    device,
                )
            )
        );

        let mut tmp_map = BTreeMap::new();
        //and finnaly insert
        tmp_map.insert(String::from("fallback"), fallback_material);

        MaterialManager{
            material_vault: tmp_map,
            renderer_inst: render.clone(),
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
    fn get_default_material(&mut self) -> Arc<Mutex<material::Material>>{
        self.material_vault.get(&String::from("fallback"))
        .expect("Could not find fallback material, this shouldn't happen, please report this bug")
        .clone()
    }

    ///Returns a metarial-option with this name
    pub fn get_material_by_name(&mut self, name: &str)-> Option<Arc<Mutex<material::Material>>>{
        let mut getter = self.material_vault.get(&String::from(name.clone()));
        match getter{
            Some(material) => return Some(material.clone()),
            None => {
                //println!("STATUS: MATERIAL_MANAGER: Could not find material: {}", name.clone());
                return None
            }
        }
        None
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

    ///Adds a material to this manager, returns an error if the material already exists
    pub fn add_material(&mut self, material: material::Material) -> Result<(), &'static str>{
        //check for the key TODO might be faster with a vector containing all keys
        if self.material_vault.contains_key(&material.get_name()){
            return Err("error, the material is already present in the material manager");
        }else{
            self.material_vault.insert(material.get_name(), Arc::new(Mutex::new(material)));
            return Ok({});
        }


    }
    ///Checks for a material
    pub fn is_available(&self, name: &str) -> bool{
        self.material_vault.contains_key(&String::from(name))
    }

    ///A debuging fuction to see all materials
    pub fn print_all_materials(&mut self){
        for (k,i) in self.material_vault.iter(){
            println!("Material: {}", k.clone());
        }
    }

}
