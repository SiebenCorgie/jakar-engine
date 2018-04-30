use render::frame_system::{FrameStage, FrameSystem};
use render::light_system::LightSystem;

use cgmath::*;
use collision::*;

///Every object that implements this trait is able to to be rendered in the forward pass.
///This are mostly wireframe models or other primitives like voxels.
/// NOTE voxels and voxel clouds are not actually implemented
pub trait ForwardRenderAble {
    ///takes a framestage, checks for the correct path, adds its own draw command and returns
    ///the same stage again (mostly the forward stage).
    fn draw(
        &self,
        frame_stage: FrameStage,
        frame_system: &FrameSystem,
        light_system: &LightSystem,
        transform: Matrix4<f32>,
    ) -> FrameStage;

    ///Returns the bound of this object
    fn get_bound(&self) -> Aabb3<f32>;

    ///Returns the name of this object
    fn get_name(&self) -> String;
}
