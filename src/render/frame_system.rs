use core::render_settings;
use core::engine_settings;
use render;

use vulkano::image::traits::ImageViewAccess;
use vulkano::image::traits::ImageAccess;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::attachment::AttachmentImage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::format::Format;
use vulkano;
use vulkano::sync::GpuFuture;
use std::sync::{Arc, Mutex};

///Describes the current stage the command buffer is in
pub enum FrameStage {
    ///Is a stage between the first and the second render pass, it does the light culling etc.
    LightCompute(AutoCommandBufferBuilder),
    ///The first stage allows to add objects to an command buffer
    Forward(AutoCommandBufferBuilder),
    //Creates a image which only holds HDR fragments
    HdrSorting(AutoCommandBufferBuilder),
    ///Is used to take the image from the first buffer and preform tone mapping on it
    Postprogress(AutoCommandBufferBuilder),
    ///Is used when next_frame() is called on the last pass
    Finished(AutoCommandBufferBuilder),
}

impl FrameStage{
    ///Returns the id of this stage
    pub fn get_id(&self) -> u32{
        match self{

            &FrameStage::LightCompute(_)=> {
                let id_type = render::SubPassType::LightCompute;
                id_type.get_id()
            }

            &FrameStage::Forward(_) =>{
                let id_type = render::SubPassType::Forward;
                id_type.get_id()
            },
            &FrameStage::HdrSorting(_) =>{
                let id_type = render::SubPassType::HdrSorting;
                id_type.get_id()
            },
            &FrameStage::Postprogress(_) =>{
                let id_type = render::SubPassType::PostProgress;
                id_type.get_id()
            },
            &FrameStage::Finished(_) =>{
                let id_type = render::SubPassType::Finished;
                id_type.get_id()
            },

        }
    }
}



///Handles the frame attachment and attachment recreation based on settings. Can start a new
/// frame and end it.
///Also store the render pass and decides what images and attachments to add.
pub struct FrameSystem {
    engine_settings:  Arc<Mutex<engine_settings::EngineSettings>>,
    //list of the available passes
    passes: render::render_passes::RenderPasses,

    //Stores the dynamic render states used for this frame
    /*TODO:
    * It would be nice to be able to configure the dynamic state. Things like "reversed" depth
    (from 1.0 - 0.0) or something like configuring wireframe line width. But that would be a nice
    to have.
    */
    dynamic_state: vulkano::command_buffer::DynamicState,

    //Sometimes holds the newly build main framebuffer, but gets taken out when switching from
    // pre-depth -> compute -> forward pass
    current_main_frame_buffer: Option<Arc<FramebufferAbstract + Send + Sync>>,

    //The buffer to which the multi sampled depth gets written
    forward_hdr_depth: Arc<ImageViewAccess + Send + Sync>,
    //this holds a multi sampled image (later hdr)
    forward_hdr_image: Arc<ImageViewAccess  + Send + Sync>, //TODO reimplement

    hdr_fragments: Arc<ImageViewAccess  + Send + Sync>,

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
        passes: render::render_passes::RenderPasses,
        target_queue: Arc<vulkano::device::Queue>,
    ) -> Self{

        //get our selfs a easy to read render_settings insance :)
        let render_settings = {
            settings
            .lock()
            .expect("failed to lock settings for frame creation")
            .get_render_settings()
            .clone()
        };

        let current_dimensions = {
            settings
            .lock()
            .expect("failed to lock settings for frame creation")
            .get_dimensions()
        };

        let static_msaa_factor = passes.object_pass.static_msaa_factor;

        let hdr_msaa_format = passes.object_pass.image_hdr_msaa_format;
        let msaa_depth_format = passes.object_pass.image_msaa_depth_format;

        //Creates a buffer for the msaa image
        let raw_render_color = AttachmentImage::transient_multisampled_input_attachment(device.clone(),
        current_dimensions, static_msaa_factor,
        hdr_msaa_format).expect("failed to create msaa buffer!");


        //Create a multisampled depth buffer depth buffer
        let forward_hdr_depth = AttachmentImage::transient_multisampled_input_attachment(
            device.clone(), current_dimensions, static_msaa_factor, msaa_depth_format)
            .expect("failed to create depth buffer!");

        let hdr_fragments = AttachmentImage::sampled_input_attachment(device.clone(),
        current_dimensions,
        hdr_msaa_format).expect("failed to create hdr buffer!");


        println!("Created images the first time", );


        println!("Created main_renderpass", );
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


        println!("Finished main_renderpass", );

        FrameSystem{

            dynamic_state: dynamic_state,

            engine_settings: settings,
            passes: passes,
            //Get created when starting a frame for later use
            current_main_frame_buffer: None,

            ///The depth buffer for the compute shader

            //The buffer to which the depth gets written
            forward_hdr_depth: forward_hdr_depth,
            //this holds a multi sampled image in hdr format
            forward_hdr_image: raw_render_color,

            hdr_fragments: hdr_fragments,

            device: device,
            queue: target_queue,
        }
    }

    ///Recreates all attachments with the right size
    pub fn recreate_attachments(&mut self){
        let new_dimensions = {
            self.engine_settings.lock()
            .expect("failed to get new dimenstions in frame system update")
            .get_dimensions()
        };

        let static_msaa_factor = self.passes.object_pass.static_msaa_factor;

        let hdr_msaa_format = self.passes.object_pass.image_hdr_msaa_format;
        let msaa_depth_format = self.passes.object_pass.image_msaa_depth_format;


        self.forward_hdr_image = AttachmentImage::transient_multisampled_input_attachment(
            self.device.clone(),
            new_dimensions, static_msaa_factor, hdr_msaa_format).expect("failed to create msaa buffer!");

        //Create a multisampled depth buffer depth buffer
        self.forward_hdr_depth = AttachmentImage::transient_multisampled_input_attachment(
            self.device.clone(), new_dimensions, static_msaa_factor, msaa_depth_format)
            .expect("failed to create depth buffer!");

        self.hdr_fragments = AttachmentImage::sampled_input_attachment(self.device.clone(),
        new_dimensions,
        hdr_msaa_format).expect("failed to create hdr buffer!");

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


    ///Starts a new frame by taking a target image and starting a command buffer for it
    pub fn new_frame<I>(&mut self, target_image: I) -> FrameStage
    where I: ImageAccess + ImageViewAccess + Clone + Send + Sync + 'static
    {
        //Recreate images if needed:
        //doing this now in the check_images() function of the renderer
        //check the frame dimensions, if changed (happens if the swapchain changes),
        //recreate all attachments

        //Create the main frame buffer
        self.current_main_frame_buffer = Some(Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.passes.object_pass.render_pass.clone())
            //Add the pre depth image
            //.add(self.pre_depth_buffer.clone()).expect("Failed to add pre depth buffer to framebuffer")
            //the msaa image
            .add(self.forward_hdr_image.clone()).expect("failed to add msaa image")
            //the multi sampled depth image
            .add(self.forward_hdr_depth.clone()).expect("failed to add msaa depth buffer")
            //The hdr format
            .add(self.hdr_fragments.clone()).expect("failed to add hdr_fragments image")
            //The color pass
            .add(target_image.clone()).expect("failed to add image to frame buffer!")
            //and its depth pass
            //.add(self.depth_buffer.clone()).expect("failed to add depth to frame buffer!")

            .build()
            .expect("failed to build main framebuffer!")
        ));

        //start the commadn buffer for this frame
        let command_buffer: AutoCommandBufferBuilder =
            vulkano::command_buffer::AutoCommandBufferBuilder::new(
                self.device.clone(),
                self.queue.family()
            )
            .expect("failed to create tmp buffer!");
        FrameStage::LightCompute(command_buffer)
    }

    ///changes to the next render pass, returns the same if already at the last pass
    pub fn next_pass(&mut self, command_buffer: FrameStage) -> FrameStage{

        match command_buffer{
            FrameStage::LightCompute(cb) => {
                //first of all try to get the main frame buffer, if not possible, panic
                let main_fb = {
                    match self.current_main_frame_buffer.take(){
                        Some(fb) => fb,
                        None => panic!("Could not find main frame buffer!"),
                    }
                };

                //For successfull clearing we generate a vector for all images.
                let clearing_values = vec![
                    [0.1, 0.1, 0.1, 1.0].into(), //forward color hdr
                    1f32.into(), //forward depth
                    [0.0, 0.0, 0.0, 1.0].into(),
                    [0.0, 0.0, 0.0, 1.0].into(), //post progress / frame buffer image
                    //1f32.into(), //
                ];
                let next = cb.begin_render_pass(main_fb, false, clearing_values)
                    .expect("failed to start main renderpass");

                FrameStage::Forward(next)
            }

            FrameStage::Forward(cb) => {
                //change to next subpass
                let next_stage = cb.next_subpass(false).expect("failed to change to Hdr Sorting render pass");
                FrameStage::HdrSorting(next_stage)
            }

            FrameStage::HdrSorting(cb) => {
                let next_stage = cb.next_subpass(false).expect("failed to change to PP render pass");
                FrameStage::Postprogress(next_stage)
            }

            FrameStage::Postprogress(cb) => {
                //println!("Is already at the last stage, end it!", );
                FrameStage::Finished(cb)
            }

            FrameStage::Finished(cb) => FrameStage::Finished(cb),
        }
    }

    ///Can extract the raw command buffer builder from a finished frame. Returns an error if
    /// the supplied stage is not in Finished stage
    pub fn finish_frame(&self, command_buffer: FrameStage) -> Result<AutoCommandBufferBuilder, String>{
        match command_buffer{
            FrameStage::Finished(cb) => Ok(cb),
            _ => Err("Could not end frame, wrong frame state!".to_string())
        }
    }

    ///Returns the msaa image
    pub fn get_forward_hdr_image(&self) -> Arc<ImageViewAccess +Sync + Send>{
        self.forward_hdr_image.clone()
    }

    ///Returns the msaa depth image
    pub fn get_forward_hdr_depth(&self) -> Arc<ImageViewAccess +Sync + Send>{
        self.forward_hdr_depth.clone()
    }

    ///Returns the hdr fragments image
    pub fn get_hdr_fragments(&self) -> Arc<ImageViewAccess +Sync + Send>{
        self.hdr_fragments.clone()
    }

    ///Returns the current, up to date dynamic state. Should be used for every onscreen rendering.
    pub fn get_dynamic_state(&self) -> &vulkano::command_buffer::DynamicState{
        &self.dynamic_state
    }


    ///Returns the id of the object pass
    pub fn get_object_pass_id(&self) -> u32{
        let id_type = render::SubPassType::Forward;
        id_type.get_id()
    }

    ///Returns the post progressing pass id
    pub fn get_post_progress_id(&self) -> u32{
        let id_type = render::SubPassType::PostProgress;
        id_type.get_id()
    }

    ///Resolveing step's ID
    pub fn get_resolve_id(&self) -> u32{
        let id_type = render::SubPassType::HdrSorting;
        id_type.get_id()
    }

}
