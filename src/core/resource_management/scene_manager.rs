use core::simple_scene_system::node;

use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;

///has a list of all available scenes
pub struct SceneManager {
    scenes: BTreeMap<String, Arc<Mutex<node::GenericNode>>>,
}

impl SceneManager {
    pub fn new() -> Self{
        SceneManager{
            scenes: BTreeMap::new(),
        }
    }

    //Adds a scene to the scene manager
    pub fn add_scene(&mut self, mut scene: node::GenericNode){
        match self.scenes.contains_key(&scene.name.clone()){
            true => {
                println!("This scene({}) already exists, adding it as {}_1", scene.name.clone(), scene.name.clone());
                let new_name = String::from(scene.name.clone()) + "_1";
                //change the internal name of this scene
                scene.name = new_name.clone();
                self.scenes.insert(new_name, Arc::new(Mutex::new(scene)));
            },
            false =>{
                self.scenes.insert(scene.name.clone(), Arc::new(Mutex::new(scene)));
            },
        }
    }

    ///Returns Some(scene) by name from the `scenes` Vector
    pub fn get_scene(&mut self, name: &str) -> Option<Arc<Mutex<node::GenericNode>>>{

        let has = self.scenes.get(&String::from(name));
        match has{
            None => None,
            Some(scene) => Some(scene.clone()),
        }

    }

    ///Returns the scenes vector as a copy
    pub fn get_scenes_copy(&self) -> Vec<Arc<Mutex<node::GenericNode>>>{
        let mut return_vec = Vec::new();
        for (_,i) in self.scenes.iter(){
            return_vec.push(i.clone())
        }
        return_vec
    }

    ///Returns true if a scene with `name` as name exists in the `self.scenes` vector
    pub fn has_scene(&self, name: &str) -> bool{

        self.scenes.contains_key(&String::from(name))
    }
}
