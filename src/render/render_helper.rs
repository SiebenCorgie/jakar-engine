use core::resources::mesh;
use core::resources::camera;
use core::resources::camera::Camera;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::collections::BTreeMap;
use jakar_tree::*;
use core::next_tree::*;


use cgmath::*;
///Returns a thread handle which, at some point returns a ordered vector of the provided
/// `meshes` based on their distance to the `camera` (the furthest away is the first mesh, the neares is the last).
pub fn order_by_distance(
    mehes: Vec<node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>,
    camera: &camera::DefaultCamera,
) -> mpsc::Receiver<Vec<node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>>{
    //Create the pipe
    let (sender, reciver) = mpsc::channel();
    //extract the position of the camera needed for the calculation
    let camera_location = camera.get_position();

    //spawn the thread
    let _ = thread::spawn(move ||{

        //Silly ordering
        let mut ordered_meshes: BTreeMap<i64, node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>> = BTreeMap::new();

        for mesh in mehes.iter(){

            use cgmath::InnerSpace;

            let mesh_location = mesh.attributes.get_transform().disp;

            //get distance between camera and position
            let distance = mesh_location - camera_location;
            //now transform to an int and multiply by 10_000 to have some comma for better sorting
            let i_distance = (distance.magnitude().abs() * 10_000.0) as i64;

            //now add the mesh to the map based on it
            ordered_meshes.insert(i_distance, mesh.clone());

        }
        //Silly ordering end ==================================================================

        //now reorder the meshes reversed into a vec and send them to the render thread
        let mut return_vector = Vec::new();
        for (id, mesh) in ordered_meshes.into_iter().rev(){
            return_vector.push(mesh);
        }
        sender.send(return_vector);

    });

    //return the reciver for further working on the renderer
    reciver

}
