use core::engine_settings;
use render;

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano;

use std::sync::{Arc, Mutex, MutexGuard};







///Handles the frame attachment and attachment recreation based on settings. Can start a new
/// frame and end it.
///Also store the render pass and decides what images and attachments to add.
pub struct FrameSystem {
    engine_settings:  Arc<Mutex<engine_settings::EngineSettings>>,
    //list of the available passes
    pub passes: Arc<Mutex<render::render_passes::RenderPasses>>,

    /*TODO:
    * It would be nice to be able to configure the dynamic state. Things like "reversed" depth
    (from 1.0 - 0.0) or something like configuring wireframe line width. But that would be a nice
    to have.
    */
    dynamic_state: vulkano::command_buffer::DynamicState,

    //a copy of the device
    device: Arc<vulkano::device::Device>,
    //a copy of the queue
    queue: Arc<vulkano::device::Queue>,
}

impl FrameSystem{
    ///Creates a new frame system with a buffer etc.
    pub fn new(
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        device: Arc<vulkano::device::Device>,
        passes: Arc<Mutex<render::render_passes::RenderPasses>>,
        target_queue: Arc<vulkano::device::Queue>,
    ) -> Self{

        let current_dimensions = {
            settings
            .lock()
            .expect("failed to lock settings for frame creation")
            .get_dimensions()
        };

        //At this point we build the state, now we have to create the configuration for it as well
        //to be used, dynmaicly while drawing
        let dynamic_state = vulkano::command_buffer::DynamicState{
            line_width: None,
            viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                origin: [0.0, 0.0],
                dimensions: [current_dimensions[0] as f32, current_dimensions[1] as f32],
                depth_range: 0.0 .. 1.0,
            }]),
            scissors: None,
        };

        FrameSystem{

            dynamic_state: dynamic_state,

            engine_settings: settings,
            passes: passes,

            device: device,
            queue: target_queue,
        }
    }

    ///Recreates all attachments with the right size
    pub fn recreate_attachments(&mut self){

        //Dont have to recreate shadow images since they are not dependnt on the surface.

        self.get_passes().rebuild_images();

        let new_dimensions = {
            self.engine_settings
            .lock()
            .expect("failed to lock settings for frame creation")
            .get_dimensions()
        };

        //After all, create the frame dynamic states
        self.dynamic_state = vulkano::command_buffer::DynamicState{
            line_width: None,
            viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                origin: [0.0, 0.0],
                dimensions: [new_dimensions[0] as f32, new_dimensions[1] as f32],
                depth_range: 0.0 .. 1.0,
            }]),
            scissors: None,
        };
    }


    ///Starts a new frame by taking a target image and starting a command buffer for it.
    pub fn new_frame(&mut self) -> AutoCommandBufferBuilder{

        //start the command buffer for this frame
        let command_buffer: AutoCommandBufferBuilder =
            vulkano::command_buffer::AutoCommandBufferBuilder::new(
                self.device.clone(),
                self.queue.family()
            )
            .expect("failed to create tmp buffer!");
        command_buffer
    }

    ///Returns the current, up to date dynamic state. Should be used for every onscreen rendering.
    pub fn get_dynamic_state(&self) -> &vulkano::command_buffer::DynamicState{
        &self.dynamic_state
    }

    ///Returns the current unlocked passes of this system
    pub fn get_passes<'a>(&'a self) -> MutexGuard<'a, render::render_passes::RenderPasses>{
        self.passes.lock().expect("failed to lock renderpasses")
    }

}
