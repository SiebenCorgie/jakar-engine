use core::resources::mesh;
use core::resources::camera;
use core::resources::camera::Camera;
use jakar_tree::*;
use core::next_tree::*;
use render::frame_system;
use render::pipeline_manager;
use render::pipeline_builder;
use render::shader_impls;
use render::uniform_manager;

use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::collections::BTreeMap;

use cgmath::*;
use vulkano;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

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

///Function used to draw a line of bounds
pub fn add_bound_draw(
    command_buffer: frame_system::FrameStage,
    pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,
    object_node: &node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>,
    device: Arc<vulkano::device::Device>,
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
    dimensions: &[u32; 2]
) -> frame_system::FrameStage{

    //get the current command buffer stage id
    let id = command_buffer.get_id();

    match command_buffer{
        frame_system::FrameStage::Forward(cb) => {

            //Create a vertex buffer for the bound
            let mut pipeline_needed = pipeline_builder::PipelineConfig::default();
            //Setup the wireframe shader and the topology type
            pipeline_needed.topology_type = vulkano::pipeline::input_assembly::PrimitiveTopology::LineList;
            pipeline_needed.shader_set = shader_impls::ShaderTypes::Wireframe;

            let mut pipeline_lck = pipeline_manager.lock().expect("failed to lock pipeline manager");
            let pipeline = pipeline_lck.get_pipeline_by_config(pipeline_needed, device.clone(), id as u32);
            //now we get out self the points of the bound and create a vertex buffer form it

            let mut min = object_node.attributes.transform.transform_point(object_node.attributes.get_value_bound().min);
            let mut max = object_node.attributes.transform.transform_point(object_node.attributes.get_value_bound().max);
            //Now we transform them to match the object scale and location
            let value_vertices = create_vertex_buffer_for_bound(min, max, [1.0, 1.0, 0.0, 1.0]);
            let node_vertices = create_vertex_buffer_for_bound(
                object_node.attributes.bound.min, object_node.attributes.bound.max,
                [0.0, 1.0, 1.0, 1.0]
            );

            let indices: Vec<u32> = vec![
                0,1, //lower quad
                1,2,
                2,3,
                3,0,
                0,5,//columns
                1,6,
                2,7,
                3,4,
                5,6,//upper quad
                6,7,
                7,4,
                4,5
            ];

            //Now make an indice buffer
            let index_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
                ::from_iter(device.clone(), vulkano::buffer::BufferUsage::all(), indices.iter().cloned())
                .expect("failed to create index buffer 02");

            //and an vertex buffer
            let value_vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
                                        ::from_iter(
                                            device.clone(),
                                            vulkano::buffer::BufferUsage::all(),
                                            value_vertices.iter().cloned())
                                        .expect("failed to create buffer");

            //and an vertex buffer
            let node_vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
                                        ::from_iter(
                                            device.clone(),
                                            vulkano::buffer::BufferUsage::all(),
                                            node_vertices.iter().cloned())
                                        .expect("failed to create buffer");

            //We also have to create a descriptor set for the MVP stuff
            let mvp_data = (uniform_manager.lock().expect("failed to lock uniform manager"))
            .get_subbuffer_data(Matrix4::identity()); //We transformed the points our selfs
                                                                      //Thats why we use no model matrix
            //now create the set for the value
            let attachments_ds_value = PersistentDescriptorSet::start(pipeline.get_pipeline_ref(), 0)
                .add_buffer(mvp_data.clone())
                .expect("failed to add depth image")
                .build()
                .expect("failed to build postprogress cb");

            //now create the set for the node
            let attachments_ds_node = PersistentDescriptorSet::start(pipeline.get_pipeline_ref(), 0)
                .add_buffer(mvp_data)
                .expect("failed to add depth image")
                .build()
                .expect("failed to build postprogress cb");

            //draw the value bound
            let mut new_cb = cb.draw_indexed(
                pipeline.get_pipeline_ref(),
                vulkano::command_buffer::DynamicState{
                    line_width: None,
                    viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                        depth_range: 0.0 .. 1.0,
                    }]),
                    scissors: None,
                },
                vec![value_vertex_buffer],
                index_buffer.clone(),
                (attachments_ds_value),  //now descriptor sets for now
                () //also no constants
            ).expect("failed to draw bounds!");

            //draw the node bound
            new_cb = new_cb.draw_indexed(
                pipeline.get_pipeline_ref(),
                vulkano::command_buffer::DynamicState{
                    line_width: None,
                    viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                        depth_range: 0.0 .. 1.0,
                    }]),
                    scissors: None,
                },
                vec![node_vertex_buffer],
                index_buffer,
                (attachments_ds_node),  //now descriptor sets for now
                () //also no constants
            ).expect("failed to draw bounds!");


            frame_system::FrameStage::Forward(new_cb)
        },
        _ => { command_buffer }
    }
}

///A helper function to create the vertex_buffer for a bound
fn create_vertex_buffer_for_bound(min: Point3<f32>, max: Point3<f32>, color: [f32; 4]) -> Vec<mesh::Vertex> {
    let vertices = vec![
        //Point 1
        mesh::Vertex::new(                            //one could configure the bound color at this point
            [min.x, min.y, min.z], [0.0; 2], [0.0; 3], [0.0; 4], color
        ),
        //Point 2
        mesh::Vertex::new(
            [max.x, min.y, min.z] , [0.0; 2], [0.0; 3], [0.0; 4], color
        ),
        //Point 3
        mesh::Vertex::new(                            //one could configure the bound color at this point
            [max.x, max.y, min.z] , [0.0; 2], [0.0; 3], [0.0; 4], color
        ),
        //Point 4
        mesh::Vertex::new(
            [min.x, max.y, min.z] , [0.0; 2], [0.0; 3], [0.0; 4], color
        ),
        //END LOWER
        //Point 5
        mesh::Vertex::new(                            //one could configure the bound color at this point
            [min.x, max.y, max.z] , [0.0; 2], [0.0; 3], [0.0; 4], color
        ),
        //Point 6
        mesh::Vertex::new(
            [min.x, min.y, max.z] , [0.0; 2], [0.0; 3], [0.0; 4], color
        ),
        //Point 7
        mesh::Vertex::new(                            //one could configure the bound color at this point
            [max.x, min.y, max.z] , [0.0; 2], [0.0; 3], [0.0; 4], color
        ),
        //Point 8
        mesh::Vertex::new(
            [max.x, max.y, max.z] , [0.0; 2], [0.0; 3], [0.0; 4], color
        ),

    ];

    vertices
}
