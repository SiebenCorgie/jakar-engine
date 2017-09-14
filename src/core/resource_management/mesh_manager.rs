
use core::resources::mesh;
use core::simple_scene_system::node;
use tools::Importer;
//use tools::assimp_importer;

use vulkano;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;

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

    ///Imports a mesh in a seperate thread.
    ///This will do two things:
    ///
    /// 1st. Import all sub meshes of this file in seperate `Arc<Mutex<Mesh>>` objects
    ///
    /// 2nd. Create a scene with all meshes stack as children below the root node
    ///
    /// By doing this the sub.meshes can be reused to create new scene and a complex scene with
    /// different objects stays in one sub-scene

    //Deprecaed in favor of the gltf loader
    pub fn import_mesh(&mut self, name: &str, path: &str, device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
        scene_manager_scenes: Arc<Mutex<node::GenericNode>>
    )
    {

    /*
        let mut meshes_instance = self.meshes.clone();
        let mut scene_instance = scene_manager_scenes.clone();
        let device_instance = device.clone();
        let queue_instance = queue.clone();
        let name_instance = name.to_owned();
        let path_instance = path.to_owned();

        let thread = thread::spawn(move ||{

            //println!("STATUS: MESH_MANAGER: Spawned thread with id: {:?}", thread::current().id());

            let mut importer = assimp_importer::AssimpImporter::new();
            let new_meshes = importer.import(&path_instance, &name_instance, device_instance.clone(), queue_instance.clone());


            let mut arc_meshes: Vec<(String, Arc<Mutex<mesh::Mesh>>)> = Vec::new();
            for mesh in new_meshes.iter(){
                arc_meshes.push((String::from(mesh.name.clone()),Arc::new(Mutex::new(mesh.clone()))));
            }


            //Now add the mesh[s] to the meshes vector in self
            //after that build a scene from it and add the scene to
            //the scenes Vec
            {
                let mut meshes_editor = (*meshes_instance).lock().expect("failed to lock meshes vec");
                for mesh in arc_meshes.iter(){
                    meshes_editor.insert(mesh.0.clone() ,mesh.1.clone() );
                }
            }

            //now lock the scene and add all meshes to it
            //println!("STATUS: MESH_MANAGER: Adding scene with name: {}", &name_instance.clone());
            let mut root_node = scene_instance.lock().expect("faield to lock scene while adding mehes");
            for i in arc_meshes.iter(){
                //create a node
                let mesh_node = node::ContentType::Renderable(
                    node::RenderableContent::Mesh(
                        i.1.clone()
                    )
                );
                println!("Adding mesh: {} ==================", i.0.clone());
                //And add it to the scene
                root_node.add_child(mesh_node);
            }


            //println!("STATUS: MESH_MANAGER: Finshed importing {}", name_instance.clone());
        });
    */
    }
}
