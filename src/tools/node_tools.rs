
use core::resources::camera;
use core::resources::camera::Camera;
use jakar_tree::*;
use core::next_tree::*;


use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::collections::BTreeMap;

use cgmath::*;

///Returns a thread handle which, at some point returns a ordered vector of the provided
/// `nodes` based on their distance to the `location` (the furthest away is the last node).
pub fn order_by_distance(
    nodes: Vec<node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>,
    location: Vector3<f32>,
) -> mpsc::Receiver<Vec<node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>>{
    //Create the pipe
    let (sender, reciver) = mpsc::channel();
    //extract the position of the camera needed for the calculation

    //spawn the thread
    let _ = thread::spawn(move ||{

        //Silly ordering
        let mut ordered_meshes: BTreeMap<i64, node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>> = BTreeMap::new();

        for mesh in nodes.iter(){

            use cgmath::InnerSpace;

            let mesh_location = mesh.attributes.get_transform().disp;

            //get distance between camera and position
            let distance = mesh_location - location;
            //now transform to an int and multiply by 10 to have some comma for better sorting
            let i_distance = (distance.magnitude2().abs() * 10.0) as i64;

            //now add the mesh to the map based on it
            ordered_meshes.insert(i_distance, mesh.clone());

        }
        //Silly ordering end ==================================================================

        //now reorder the meshes reversed into a vec and send them to the render thread
        let mut return_vector = Vec::new();
        for (_, mesh) in ordered_meshes.into_iter(){
            return_vector.push(mesh);
        }
        match sender.send(return_vector){
            Ok(_) => {},
            Err(er) => panic!("failed to send ordered nodes! {:?}", er)
        }

    });

    //return the reciver for further working on the renderer
    reciver
}
