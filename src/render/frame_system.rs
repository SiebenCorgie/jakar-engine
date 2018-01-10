use core::render_settings;
use core::engine_settings;
use render;

use vulkano::image::traits::ImageViewAccess;
use vulkano::image::traits::ImageAccess;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::attachment::AttachmentImage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::format::Format;
use vulkano;
use vulkano::sync::GpuFuture;
use std::sync::{Arc, Mutex};
///Describes the current stage the command buffer is in
pub enum FrameStage {
    ///Generates the depth image used in a compute shader to generate a list of light indices used in
    /// the fotward pass to reduce calculation time. Hvae a look at "Forward+" for more information.
    PreDepth(AutoCommandBufferBuilder),
    ///The first stage allows to add objects to an command buffer
    Forward(AutoCommandBufferBuilder),
    ///Is used to take the image from the first buffer and preform tone mapping on it
    Postprogress(AutoCommandBufferBuilder),
    ///Is used when next_frame() is called on the last pass
    Finished(AutoCommandBufferBuilder),
}

impl FrameStage{
    ///Returns the id of this stage
    pub fn get_id(&self) -> u32{
        match self{
            &FrameStage::PreDepth(_)=>{
                let id_type = render::SubPassType::PreDepth;
                id_type.get_id()
            }
            &FrameStage::Forward(_) =>{
                let id_type = render::SubPassType::Forward;
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

    //Stores the dynamic render states used for this frame
    /*TODO:
    * It would be nice to be able to configure the dynamic state. Things like "reversed" depth
    (from 1.0 - 0.0) or something like configuring wireframe line width. But that would be a nice
    to have.
    */
    dynamic_state: vulkano::command_buffer::DynamicState,


    renderpass: Arc<RenderPassAbstract + Send + Sync>,

    ///the image used in the compute shader
    pre_depth_buffer: Arc<ImageViewAccess + Send + Sync>,
    pre_depth_color: Arc<ImageViewAccess + Send + Sync>,

    //The buffer to which the multi sampled depth gets written
    forward_hdr_depth: Arc<ImageViewAccess + Send + Sync>,
    //this holds a multi sampled image (later hdr)
    forward_hdr_image: Arc<ImageViewAccess  + Send + Sync>, //TODO reimplement

    static_msaa_factor: u32,

    //a copy of the device
    device: Arc<vulkano::device::Device>,
    //a copy of the queue
    queue: Arc<vulkano::device::Queue>,

    image_hdr_msaa_format: Format,
    image_msaa_depth_format: Format,
    swapchain_format: Format,


}

impl FrameSystem{
    ///Creates a new frame system with a buffer etc.
    pub fn new(
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        device: Arc<vulkano::device::Device>,
        target_queue: Arc<vulkano::device::Queue>,
        swapchain_format: Format
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

        let static_msaa_factor = render_settings.get_msaa_factor();

        let hdr_msaa_format = vulkano::format::Format::R16G16B16A16Sfloat;
        let msaa_depth_format = vulkano::format::Format::D16Unorm;

        //Create the first images
        //Image used in the compute shader
        let pre_depth_color = AttachmentImage::input_attachment(device.clone(),
        current_dimensions, //sadly we have to write this one in the same size
        swapchain_format).expect("failed to create pre_depth color image!");


        let pre_depth = AttachmentImage::transient_input_attachment(device.clone(),
        current_dimensions,
        msaa_depth_format).expect("failed to create pre_depth buffer!");

        //Creates a buffer for the msaa image
        let raw_render_color = AttachmentImage::transient_multisampled_input_attachment(device.clone(),
        current_dimensions, static_msaa_factor,
        hdr_msaa_format).expect("failed to create msaa buffer!");


        //Create a multisampled depth buffer depth buffer
        let forward_hdr_depth = AttachmentImage::transient_multisampled_input_attachment(
            device.clone(), current_dimensions, static_msaa_factor, msaa_depth_format)
            .expect("failed to create depth buffer!");


        println!("Created images the first time", );
        //Setup the render_pass layout
        let renderpass = Arc::new(
            ordered_passes_renderpass!(target_queue.device().clone(),
            attachments: {
                /*
                pre_depth_color: {
                    load: Clear,
                    store: DontCare,
                    format: swapchain_format, //works per vulkan definition
                    samples: 1,
                },
                */
                //the second one is the msaa depth buffer
                pre_depth: {
                    load: Clear,
                    store: DontCare,
                    format: msaa_depth_format, //works per vulkan definition
                    samples: 1,
                },

                // The first framebuffer attachment is the raw_render_color image.
                raw_render_color: {
                    load: Clear,
                    store: Store,
                    format: hdr_msaa_format, //Defined that it works by the vulkan implementation
                    samples: static_msaa_factor,     // This has to match the image definition.
                },

                //the second one is the msaa depth buffer
                raw_render_depth: {
                    load: Clear,
                    store: DontCare,
                    format: msaa_depth_format, //works per vulkan definition
                    samples: static_msaa_factor,
                },

                //the final image
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain_format, //this needs to have the format which is presentable to the window
                    samples: 1, //target image is not sampled
                }

            },
            passes:[
                {
                    color: [], //NO Color value
                    depth_stencil: {pre_depth},
                    input: []
                },

                {
                    color: [raw_render_color], //TODO change to msaa
                    depth_stencil: {raw_render_depth},
                    input: []
                },

                {
                    color: [color],
                    depth_stencil: {},
                    input: [raw_render_color]
                    //resolve: [color]
                }

            ]

        ).expect("failed to create render_pass")
        );
        println!("Created RenderPass", );
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


        println!("Finished renderpass", );

        FrameSystem{

            dynamic_state: dynamic_state,

            engine_settings: settings,

            renderpass: renderpass,
            ///The depth buffer for the compute shader
            pre_depth_color: pre_depth_color,
            pre_depth_buffer: pre_depth,

            //The buffer to which the depth gets written
            forward_hdr_depth: forward_hdr_depth,
            //this holds a multi sampled image in hdr format
            forward_hdr_image: raw_render_color,

            static_msaa_factor: static_msaa_factor,

            image_hdr_msaa_format: hdr_msaa_format,
            image_msaa_depth_format: msaa_depth_format,
            swapchain_format: swapchain_format,
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

        self.pre_depth_color = AttachmentImage::transient_input_attachment(self.device.clone(),
        new_dimensions,
        self.swapchain_format).expect("failed to create pre_depth color image!");


        self.pre_depth_buffer = AttachmentImage::transient_input_attachment(self.device.clone(),
        new_dimensions,
        self.image_msaa_depth_format).expect("failed to create pre_depth buffer!");


        self.forward_hdr_image = AttachmentImage::transient_multisampled_input_attachment(
            self.device.clone(),
            new_dimensions, self.static_msaa_factor, self.image_hdr_msaa_format).expect("failed to create msaa buffer!");

        //Create a multisampled depth buffer depth buffer
        self.forward_hdr_depth = AttachmentImage::transient_multisampled_input_attachment(
            self.device.clone(), new_dimensions, self.static_msaa_factor, self.image_msaa_depth_format)
            .expect("failed to create depth buffer!");

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


        //Create the frame buffer
        let frame_buffer = {
            Arc::new(vulkano::framebuffer::Framebuffer::start(self.renderpass.clone())
                //Add the pre depth color image
                //.add(self.pre_depth_color.clone()).expect("Failed to add pre depth color to framebuffer")
                //Add the pre depth image
                .add(self.pre_depth_buffer.clone()).expect("Failed to add pre depth buffer to framebuffer")
                //the msaa image
                .add(self.forward_hdr_image.clone()).expect("failed to add msaa image")
                //the multi sampled depth image
                .add(self.forward_hdr_depth.clone()).expect("failed to add msaa depth buffer")
                //The color pass
                .add(target_image.clone()).expect("failed to add image to frame buffer!")
                //and its depth pass
                //.add(self.depth_buffer.clone()).expect("failed to add depth to frame buffer!")

                .build().expect("failed to build framebuffer!"))
        };

        //For successfull clearing we generate a vector for all images.
        let clearing_values = vec![
            1f32.into(), //pre depth
            [0.1, 0.1, 0.1, 1.0].into(), //forward color hdr
            1f32.into(), //forward depth
            [0.1, 0.1, 0.1, 1.0].into(), //post progress / frame buffer image
            //1f32.into(), //
        ];



        //start the commadn buffer for this frame
        let command_buffer: AutoCommandBufferBuilder =
            vulkano::command_buffer::AutoCommandBufferBuilder::new(
                self.device.clone(),
                self.queue.family()
            )
            .expect("failed to create tmp buffer!")
            .begin_render_pass(
                frame_buffer, false,
                clearing_values
            ).expect("failed to clear")
            ;

            FrameStage::PreDepth(command_buffer)
    }

    ///changes to the next render pass, returns the same if already at the last pass
    pub fn next_pass(&self, command_buffer: FrameStage) -> FrameStage{
        match command_buffer{
            FrameStage::PreDepth(cb) =>{
                let next_stage = cb.next_subpass(false).expect("Failed to change to forward pass from pre depth");
                FrameStage::Forward(next_stage)
            }
            FrameStage::Forward(cb) => {
                //change to next subpass
                let next_stage = cb.next_subpass(false).expect("failed to change to next render pass");
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

    ///Returns the current pre_depth image
    pub fn get_pre_depth_image(&self) -> Arc<ImageViewAccess +Sync + Send>{
        self.pre_depth_buffer.clone()
    }

    ///Returns the currently used renderpass layout
    pub fn get_renderpass(&self) -> Arc<RenderPassAbstract + Send + Sync>{
        self.renderpass.clone()
    }

    ///Returns the current, up to date dynamic state. Should be used for every onscreen rendering.
    pub fn get_dynamic_state(&self) -> &vulkano::command_buffer::DynamicState{
        &self.dynamic_state
    }

    ///Returns the id of the pre_depth pass
    pub fn get_pre_depth_pass_id(&self) -> u32{
        let id_type = render::SubPassType::PreDepth;
        id_type.get_id()
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

}
