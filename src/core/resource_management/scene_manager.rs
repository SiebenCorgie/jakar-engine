use core::next_tree::*;
use jakar_tree;

use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::BTreeMap;

///has a list of all available scenes
pub struct SceneManager {
    scenes: BTreeMap<String, Arc<Mutex<jakar_tree::tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>>>,
}

impl SceneManager {
    pub fn new() -> Self{
        SceneManager{
            scenes: BTreeMap::new(),
        }
    }

    //Adds a scene to the scene manager by
    pub fn add_scene(&mut self, mut scene: jakar_tree::tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>){
        match self.scenes.contains_key(&scene.name.clone()){
            true => {
                //the scene exist, going to generate an indice which doesnt exist
                let mut indice = 0;
                while self.scenes.contains_key(&(scene.name.clone() + "_" + &indice.to_string())){
                    indice +=1;
                }

                //printing out the debug message
                println!(
                    "The scene '{}' already exists, adding it as '{}_{}'",
                    scene.name.clone(), scene.name.clone(), indice
                );


                let new_name = String::from(scene.name.clone()) + "_" + &indice.to_string();
                //change the internal name of this scene
                scene.name = new_name.clone();
                self.scenes.insert(new_name, Arc::new(Mutex::new(scene)));
            },
            //All is fine, we can add it normaly to the manager
            false =>{
                self.scenes.insert(scene.name.clone(), Arc::new(Mutex::new(scene)));
            },
        }
    }

    ///Returns Some(scene) by name from the `scenes` Vector as a Mutex guard
    pub fn get_scene(&mut self, name: &str) -> Option<
        MutexGuard<jakar_tree::tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>
    >{
        let has = self.scenes.get(&String::from(name));
        match has{
            None => None,
            Some(scene) => Some(scene.lock().expect("failed to load scene")),
        }
    }

    ///Returns Some(scene) by name from the `scenes` Vector as an Arc<Mutex<T>>
    pub fn get_scene_arc(&mut self, name: &str) -> Option<
        Arc<Mutex<jakar_tree::tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>>
    >{
        let has = self.scenes.get(&String::from(name));
        match has{
            None => None,
            Some(scene) => Some(scene.clone()),
        }
    }

    ///Returns the scenes as a copy within a vector
    pub fn get_scenes_copy(&self) -> Vec<
        Arc<Mutex<jakar_tree::tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>>
    >{
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

    ///prints a list of all stored scenes
    pub fn print_all_scenes(&self){
        println!("Alls stored scenes: ", );
        for (k,_) in self.scenes.iter(){
            println!("\t {}", k);
        }
    }
}
