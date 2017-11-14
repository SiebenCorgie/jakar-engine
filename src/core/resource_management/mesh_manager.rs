
use core::resources::mesh;
use jakar_tree::node;
use core::next_tree::*;
//use tools::assimp_importer;

use vulkano;

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

        let mut mesh_lck = self.meshes.lock().expect("Failed to hold while adding mesh to mesh manager");
        //have a look for this mesh in self
        let b_contains = (*mesh_lck).contains_key(&String::from(mesh.name.clone()));
        match b_contains{
            true => {
                println!("The name: {} is already in the mesh manager, will add as {}_1", mesh.name.clone(), mesh.name.clone());
                let new_name = mesh.name.clone() + "_1";
                //change the name in mesh
                mesh.name = new_name.clone();
                (*mesh_lck).insert(new_name, Arc::new(Mutex::new(mesh)));
            },
            false => {
               (*mesh_lck).insert(mesh.name.clone(), Arc::new(Mutex::new(mesh)));
            }
        }

    }

    ///Adds a mesh which is already an `Arc<Mutex<Mesh>>`
    pub fn add_arc_mesh(&mut self, mesh: Arc<Mutex<mesh::Mesh>>){
        //get the meshs name and check if its already in there, if, cahnge the meshs name
        let mesh_name ={
            let mesh_ref_lck = mesh.lock().expect("failed to lock mesh while adding to manager");
            (*mesh_ref_lck).name.clone()
        };
        //now test the name
        let mut mesh_lck = self.meshes.lock().expect("Failed to hold while adding mesh to mesh manager");
        //have a look for this mesh in self
        let b_contains = (*mesh_lck).contains_key(&mesh_name);

        match b_contains {
            true => {
                println!("The name: {} is already in the mesh manager, will add as {}_1", mesh_name, mesh_name);
                let new_name = mesh_name + "_1";
                //change the name in mesh
                {
                    let mut mesh_ref_lck = mesh.lock().expect("failed to lock mesh while adding to manager");
                    (*mesh_ref_lck).name = new_name.clone();
                }
                //and add it to the manager finally
                (*mesh_lck).insert(new_name, mesh);
            }
            false => {
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
