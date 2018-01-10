use render::uniform_manager;
use render::frame_system;
use render::pipeline;
use jakar_tree;
use core::next_tree;

use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano;


use cgmath::*;

use std::sync::{Arc,Mutex};

pub struct PreDpethSystem {
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
    depth_pipeline: Arc<pipeline::Pipeline>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
}

impl PreDpethSystem{
    pub fn new(
        uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
        pipeline: Arc<pipeline::Pipeline>,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
    ) -> Self {
        PreDpethSystem{
            uniform_manager: uniform_manager,
            depth_pipeline: pipeline,
            device: device,
            queue: queue,
        }
    }

    pub fn draw_object(
        &mut self,
        command_buffer: frame_system::FrameStage,
        node: &jakar_tree::node::Node<
            next_tree::content::ContentType,
            next_tree::jobs::SceneJobs,
            next_tree::attributes::NodeAttributes>,
        dynamic_state: &vulkano::command_buffer::DynamicState,
    ) -> frame_system::FrameStage{
        //first match the command buffer state, then add the node
        match command_buffer{
            frame_system::FrameStage::PreDepth(cb) => {

                //get the actual mesh as well as its pipeline an create the descriptor sets
                let mesh_locked = match node.value{
                    next_tree::content::ContentType::Mesh(ref mesh) => mesh.clone(),
                    _ => return frame_system::FrameStage::PreDepth(cb), //is no mesh :(
                };

                let mesh = mesh_locked.lock().expect("failed to lock mesh in cb creation");

                let mesh_transform = node.attributes.get_matrix();

                let set_01 = self.get_descriptor(mesh_transform);

                //extend the current command buffer by this mesh
                let new_cb = cb
                    .draw_indexed(
                        self.depth_pipeline.get_pipeline_ref(),
                        dynamic_state.clone(),
                        mesh
                        .get_vertex_buffer(), //vertex buffer (static usually)
                        mesh
                        .get_index_buffer(
                            self.device.clone(), self.queue.clone()
                        ).clone(), //index buffer
                        (set_01),
                        ()
                    )
                    .expect("Failed to draw in command buffer!");

                frame_system::FrameStage::PreDepth(new_cb)
            },
            _ => {
                ::rt_error("Pre-Depth-Stage", "Could not render node, wrong stage.");
                command_buffer
            }
        }
    }

    ///Creates a descriptor set from a node
    fn get_descriptor(&self, transform_matrix: Matrix4<f32>) -> Arc<DescriptorSet + Send + Sync>{
        let mut uniform_manager_lck = self.uniform_manager.lock().expect("Failed to lock unfiorm_mng");
        let new_set = Arc::new(PersistentDescriptorSet::start(
                self.depth_pipeline.get_pipeline_ref().clone(), 0
            )
            .add_buffer(uniform_manager_lck.get_subbuffer_data(transform_matrix)).expect("Failed to create descriptor set")
            .build().expect("failed to build descriptor \"Depth\"")
        );
        new_set
    }

}
