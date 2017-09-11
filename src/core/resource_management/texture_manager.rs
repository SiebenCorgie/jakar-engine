use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;

use core::resources::texture;
use core::engine_settings;

use vulkano::device::{Device, Queue};
use vulkano;

///Holds all kinds of different textures which can be retrieved as a copy in for of a Arc<Texture<F>>
pub struct TextureManager {
    //rgba textures
    textures: BTreeMap<String, Arc<texture::Texture>>,

    //Some more copys of structures needed to create more textures
    device: Arc<Device>,
    queue: Arc<Queue>,
    engine_settings: Arc<Mutex<engine_settings::EngineSettings>>
}


impl TextureManager{
    ///Creates a new Texture manager with two textures:
    ///A `none` texture, containing a single black pixel, which can be use d for instance if
    ///you are only using the `material_factor_*` values in a material.
    ///
    ///And a `default_texture`, which usually is used if the engine can't find a material or texture
    /// the `default_texture` comes as `albedo`, `normal`, and `physical` variant
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        engine_settings: Arc<Mutex<engine_settings::EngineSettings>>
    ) -> Self{


        //The default texture
        let none_texture = texture::TextureBuilder::from_image(
            "/home/siebencorgie/Scripts/Rust/engine/ori-engine/data/nothing.png",
            device.clone(),
            queue.clone(),
            engine_settings.clone()
        )
        .build_with_name("none");

        //The fallback textures
        let fallback_albedo = texture::TextureBuilder::from_image(
            "/home/siebencorgie/Scripts/Rust/engine/ori-engine/data/fallback_alb.png",
            device.clone(),
            queue.clone(),
            engine_settings.clone()
        )
        .build_with_name("fallback_albedo");

        let fallback_normal = texture::TextureBuilder::from_image(
            "/home/siebencorgie/Scripts/Rust/engine/ori-engine/data/fallback_nrm.png",
            device.clone(),
            queue.clone(),
            engine_settings.clone()
        )
        .build_with_name("fallback_normal");

        let fallback_physical = texture::TextureBuilder::from_image(
            "/home/siebencorgie/Scripts/Rust/engine/ori-engine/data/fallback_physical.png",
            device.clone(),
            queue.clone(),
            engine_settings.clone()
        )
        .build_with_name("fallback_physical");

        //Now store all the textures
        let mut current_textures = BTreeMap::new();

        current_textures.insert(String::from("none"), none_texture);
        current_textures.insert(String::from("fallback_albedo"), fallback_albedo);
        current_textures.insert(String::from("fallback_normal"), fallback_normal);
        current_textures.insert(String::from("fallback_physical"), fallback_physical);

        //Create the struct and return it
        TextureManager{
            textures: current_textures,
            //Some more copys of structures needed to create more textures
            device: device.clone(),
            queue: queue.clone(),
            engine_settings: engine_settings.clone()
        }
    }

    ///Returns the set of fallback textures in teh order: (albedo, normal, physical)
    ///TODO This could be faster by using index 1-3
    pub fn get_fallback_textures(&mut self) -> (
        Arc<texture::Texture>,
        Arc<texture::Texture>,
        Arc<texture::Texture>
    ){
        let albedo_tex = self.get_texture("fallback_albedo");
        let normal_tex = self.get_texture("fallback_normal");
        let physical_tex = self.get_texture("fallback_physical");



        (albedo_tex, normal_tex, physical_tex)
    }

    ///Returns the nothing 1x1 pixel texture
    ///TODO this could be faster by using index 0
    pub fn get_none(&self) -> Arc<texture::Texture>{
        self.get_texture("none")
    }

    ///Returns a texture if this name, if not found, returns th fallback texture
    pub fn get_texture(&self, name: &str) -> Arc<texture::Texture>{

        match self.textures.get(&String::from(name)){
            Some(texture) => return texture.clone(),
            None => {
                //if no texture was found return the fallback albedo (this should always be 1)
                println!(
                    "WARNING: TEXTURE_MANAGER: Returning fallback albedo because: {} was not found",
                    name.clone()
                );

                self.textures.get("none").unwrap().clone()

            }
        }
    }

    ///Adds a new texture to the manager, this will return an error if the texture is already in
    ///the manager
    pub fn add_texture(&mut self, texture: Arc<texture::Texture>) -> Result<(), &'static str>{

        //get the texture name and test the already used textures
        let name = texture.name.clone();
        //check if already in
        if self.textures.contains_key(&name){
            return Err("This texture is already in th manager");
        }

        //If the texture passed the name test, we can add it
        self.textures.insert(name, texture);
        Ok({})
    }

    ///Removes the texture from the manager, keep in mind that any copy will live till it is droped
    ///by its holder.
    ///TODO verfiy that not index 0-3 is delted (used for the system)
    pub fn remove_texture(mut self, texture_name: &str) -> Result<(), &'static str>{


        match self.textures.remove(&String::from(texture_name)){
            Some(t) => return Ok({}),
            None => Err("the texture could not removed because it is not in the manager"),
        }
    }

    ///A small helper function which returns the used engine settings, good if you have to transport
    ///much data between function
    pub fn get_settings(&self) -> Arc<Mutex<engine_settings::EngineSettings>>{
        self.engine_settings.clone()
    }

    ///Returns the stored device
    pub fn get_device(&self) -> Arc<vulkano::device::Device>{
        self.device.clone()
    }

    ///Returns the stored queue
    pub fn get_queue(&self) -> Arc<vulkano::device::Queue>{
        self.queue.clone()
    }
}
