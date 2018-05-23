use render::frame_system::FrameSystem;
use render::light_system::LightSystem;

use core::next_tree::attributes::NodeAttributes;
use core::next_tree::content::ContentType;
use core::next_tree::jobs::SceneJobs;
use jakar_tree::node::NodeController;

use vulkano::command_buffer::AutoCommandBufferBuilder;

use cgmath::*;
use collision::*;

use std::sync::{Arc,Mutex};

///Every object that implements this trait is able to to be rendered in the forward pass.
///This are mostly wireframe models or other primitives like voxels.
/// NOTE voxels and voxel clouds are not actually implemented
pub trait ForwardRenderAble {
    ///takes a framestage, checks for the correct path, adds its own draw command and returns
    ///the same stage again (mostly the forward stage).
    fn draw(
        &self,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
        light_system: &LightSystem,
        transform: Matrix4<f32>,
    ) -> AutoCommandBufferBuilder;

    ///Returns the bound of this object
    fn get_bound(&self) -> Aabb3<f32>;

    ///Returns the name of this object
    fn get_name(&self) -> String;

    ///Returns a controller if there is one
    fn get_controller(&mut self) -> Arc<Mutex<NodeController<ContentType, SceneJobs, NodeAttributes>>>;

}
