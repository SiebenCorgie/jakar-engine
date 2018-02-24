
use core::resources::mesh;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

///The structure containing all meshes and created scenes
pub struct MeshManager {
    meshes: Arc<Mutex<BTreeMap<String, Arc<Mutex<mesh::Mesh>>>>>,

}

impl MeshManager {
    pub fn new() -> Self{
        MeshManager{
            meshes: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    ///Adds a mesh to the manager
    pub fn add_mesh(&mut self, mut mesh: mesh::Mesh){


        //have a look for this mesh in self
        let b_contains = {
            let mut mesh_lck = self.meshes.lock().expect("Failed to hold while adding mesh to mesh manager");
            (*mesh_lck).contains_key(&String::from(mesh.name.clone()))
        };

        match b_contains{
            true => {
                //generate a unque mesh name
                let unique_name = self.get_unique_name(mesh.name.clone());
                println!("The name: {} is already in the mesh manager, will add as {}", mesh.name.clone(), unique_name);
                //change the name in mesh
                mesh.name = unique_name.clone();
                let mut mesh_lck = self.meshes.lock().expect("Failed to hold while adding mesh to mesh manager");
                (*mesh_lck).insert(unique_name, Arc::new(Mutex::new(mesh)));
            },
            false => {
                let mut mesh_lck = self.meshes.lock().expect("Failed to hold while adding mesh to mesh manager");
               (*mesh_lck).insert(mesh.name.clone(), Arc::new(Mutex::new(mesh)));
            }
        }

    }

    ///helper function to get a not taken name, returns the name + _id
    fn get_unique_name(&self, name: String) -> String{
        //lock the meshes
        let mesh_lck = self.meshes.lock().expect("failed to lock meshes for unique name");
        let mut unique_id = 0;

        while mesh_lck.contains_key(&(name.clone() + "_" + &unique_id.to_string())){
            unique_id += 1;
        }

        name + "_" + &unique_id.to_string()

    }

    ///Adds a mesh which is already an `Arc<Mutex<Mesh>>`
    pub fn add_arc_mesh(&mut self, mesh: Arc<Mutex<mesh::Mesh>>){
        //get the meshs name and check if its already in there, if, cahnge the meshs name
        let mesh_name ={
            let mesh_ref_lck = mesh.lock().expect("failed to lock mesh while adding to manager");
            (*mesh_ref_lck).name.clone()
        };

        //have a look for this mesh in self
        let b_contains = {
            //now test the name
            let mesh_lck = self.meshes.lock().expect("Failed to hold while adding mesh to mesh manager");
            (*mesh_lck).contains_key(&mesh_name)
        };

        match b_contains {
            true => {
                let unique_name = self.get_unique_name(mesh_name.clone());
                println!("The name: {} is already in the mesh manager, will add as {}", mesh_name, unique_name);
                //change the name in mesh
                {
                    let mut mesh_ref_lck = mesh.lock().expect("failed to lock mesh while adding to manager");
                    (*mesh_ref_lck).name = unique_name.clone();
                }
                let mut mesh_lck = self.meshes.lock().expect("Failed to hold while adding mesh to mesh manager");
                //and add it to the manager finally
                (*mesh_lck).insert(unique_name, mesh);
            }
            false => {
                let mut mesh_lck = self.meshes.lock().expect("Failed to hold while adding mesh to mesh manager");
                //if all is right just add it
                (*mesh_lck).insert(mesh_name, mesh);
            }
        }
    }

    ///Returns a mesh by name without locking it (as a Arc<T> clone)
    pub fn get_mesh(&mut self, name: &str) -> Option<Arc<Mutex<mesh::Mesh>>>{
        let meshes = self.meshes.lock().expect("faield to lock meshes");
        match meshes.get(&String::from(name)){
            Some(mesh) => Some(mesh.clone()),
            None => None,
        }
    }
}
