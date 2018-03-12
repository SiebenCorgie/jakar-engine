use core::engine_settings;
use render;

use vulkano::image::traits::ImageViewAccess;
use vulkano::image::traits::ImageAccess;
use vulkano::image::attachment::AttachmentImage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::format::Format;
use vulkano::image::ImageUsage;
use vulkano::image::StorageImage;
use vulkano::image::Dimensions;
use vulkano;

use std::sync::{Arc, Mutex};

///Describes the current stage the command buffer is in
pub enum FrameStage {
    ///Is a stage between the first and the second render pass, it does the light culling etc.
    LightCompute(AutoCommandBufferBuilder),
    ///The first stage allows to add objects to an command buffer
    Forward(AutoCommandBufferBuilder),
    //Creates a image which only holds HDR fragments
    HdrSorting(AutoCommandBufferBuilder),
    ///Blurs Horizontal
    BlurH(AutoCommandBufferBuilder),
    ///Blurs vertical
    BlurV(AutoCommandBufferBuilder),
    ///Computes the current average lumiosity
    ComputeAverageLumiosity(AutoCommandBufferBuilder),
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
            &FrameStage::BlurH(_) =>{
                let id_type = render::SubPassType::Blur;
                id_type.get_id()
            },
            &FrameStage::BlurV(_) =>{
                let id_type = render::SubPassType::Blur;
                id_type.get_id()
            },
            &FrameStage::HdrSorting(_) =>{
                let id_type = render::SubPassType::HdrSorting;
                id_type.get_id()
            },
            &FrameStage::ComputeAverageLumiosity(_) =>{
                let id_type = render::SubPassType::ComputeAverageLumiosity;
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

///Collects the images from the MainRenderPass
pub struct ObjectPassImages {
    //The buffer to which the multi sampled depth gets written
    pub forward_hdr_depth: Arc<ImageViewAccess + Send + Sync>,
    //Holds the raw multisampled hdr colors
    pub forward_hdr_image: Arc<ImageViewAccess + Send + Sync>, //TODO reimplement
    //Adter sorting the hdr fragments (used for bluring)
    pub hdr_fragments: Arc<ImageViewAccess + Send + Sync>,
    //The ldr fragments
    //pub ldr_fragments: Arc<ImageViewAccess + Send + Sync>,
    pub ldr_fragments: Arc<AttachmentImage<Format>>,

    pub transfer_usage: ImageUsage,
}

impl ObjectPassImages{
    pub fn new(
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        passes: &render::render_passes::RenderPasses,
        device: Arc<vulkano::device::Device>
    ) -> Self{

        let current_dimensions = {
            settings
            .lock()
            .expect("failed to lock settings for frame creation")
            .get_dimensions()
        };

        let transfer_usage = ImageUsage{
            transfer_source: true,
            sampled: true,
            color_attachment: true,
            input_attachment: true,
            ..ImageUsage::none()
        };

        let static_msaa_factor = passes.static_msaa_factor;

        let hdr_msaa_format = passes.image_hdr_msaa_format;
        let msaa_depth_format = passes.image_msaa_depth_format;

        //Creates a buffer for the msaa image
        let forward_hdr_color = AttachmentImage::transient_multisampled_input_attachment(device.clone(),
        current_dimensions, static_msaa_factor,
        hdr_msaa_format).expect("failed to create raw_render_color buffer!");


        //Create a multisampled depth buffer depth buffer
        let forward_hdr_depth = AttachmentImage::transient_multisampled_input_attachment(
            device.clone(), current_dimensions, static_msaa_factor, msaa_depth_format)
            .expect("failed to create forward_hdr_depth buffer!");

        let hdr_fragments = AttachmentImage::sampled_input_attachment(device.clone(),
        current_dimensions,
        hdr_msaa_format).expect("failed to create hdr_fragments buffer!");

        //Uses a custom usage parameter becuase it is later blir to a 1pixel texture for adaptive eye corretction
        let ldr_fragments = AttachmentImage::with_usage(device.clone(),
        current_dimensions,
        hdr_msaa_format, transfer_usage.clone()).expect("failed to create ldr_fragments buffer!");

        ObjectPassImages{
            forward_hdr_depth: forward_hdr_depth,
            forward_hdr_image: forward_hdr_color,
            hdr_fragments: hdr_fragments,
            ldr_fragments: ldr_fragments,
            transfer_usage: transfer_usage,
        }
    }
}

///Collects the two PostImages
pub struct PostImages {
    pub after_blur_h: Arc<ImageViewAccess  + Send + Sync>,
    pub after_blur_v: Arc<ImageViewAccess  + Send + Sync>,
    pub scaled_ldr_images: Vec<Arc<StorageImage<Format>>>,
    //pub average_lumiosity: Arc<StorageImage<Format>>,
}

impl PostImages{
    pub fn new(
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        passes: &render::render_passes::RenderPasses,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
    ) -> Self{

        let current_dimensions = {
            settings
            .lock()
            .expect("failed to lock settings for frame creation")
            .get_dimensions()
        };

        let hdr_msaa_format = passes.image_hdr_msaa_format;

        //We need two sampled images
        let after_blur_h = AttachmentImage::sampled_input_attachment(device.clone(),
        current_dimensions,
        hdr_msaa_format).expect("failed to create after_blur_h buffer!");

        let after_blur_v = AttachmentImage::sampled_input_attachment(device.clone(),
        current_dimensions,
        hdr_msaa_format).expect("failed to create after_blur_v buffer!");

        let transfer_usage = ImageUsage{
            transfer_destination: true,
            transfer_source: true,
            color_attachment: true,
            sampled: true,
            ..ImageUsage::none()
        };

        //For several PostProgress Stuff we might need scaled images of the original image
        // we do this by storing several layers of scaled images in this vec.
        //The first one in half size comapred to the original, the last one is a 1x1 texture
        let mut scaled_ldr_images = Vec::new();

        let mut new_dimension_image = current_dimensions;
        //Always use the dimension, create image, then scale down
        'reduction_loop: loop {

            //calculate the half dimension
            let mut new_dim_x = ((new_dimension_image[0] as f32).floor() / 2.0) as u32;
            let mut new_dim_y = ((new_dimension_image[1] as f32).floor() / 2.0) as u32;
            //Check if we reached 1 for only one but not the other, if so, cap at one
            if new_dim_x < 1 && new_dim_y >= 1{
                new_dim_x = 1;
            }

            if new_dim_y < 1 && new_dim_x >= 1{
                new_dim_y = 1;
            }

            new_dimension_image = [
                new_dim_x,
                new_dim_y
            ];

            //create the half image
            let new_image = StorageImage::with_usage(
                device.clone(), Dimensions::Dim2d{
                    width: new_dimension_image[0],
                    height: new_dimension_image[1]
                },
                hdr_msaa_format, transfer_usage, vec![queue.family()].into_iter()
            ).expect("failed to create one pix image");
            //push it
            scaled_ldr_images.push(new_image);
            println!("Aspect_last_frame: [{}, {}]", new_dimension_image[0], new_dimension_image[1]);

            //break if reached the 1x1 pixel
            if new_dim_x <= 1 && new_dim_y <= 1{
                break;
            }
        }


        PostImages{
            after_blur_h: after_blur_h,
            after_blur_v: after_blur_v,
            scaled_ldr_images: scaled_ldr_images,
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

    //The current collection of the object pass images
    pub object_pass_images: ObjectPassImages,
    ///The current collection of blur images
    pub post_pass_images: PostImages,

    /*TODO:
    * It would be nice to be able to configure the dynamic state. Things like "reversed" depth
    (from 1.0 - 0.0) or something like configuring wireframe line width. But that would be a nice
    to have.
    */
    dynamic_state: vulkano::command_buffer::DynamicState,

    //Sometimes holds the newly build main framebuffer, but gets taken out when switching from
    // pre-depth -> compute -> forward pass
    object_pass_fb: Option<Arc<FramebufferAbstract + Send + Sync>>,
    blur_pass_h_fb: Option<Arc<FramebufferAbstract + Send + Sync>>,
    blur_pass_v_fb: Option<Arc<FramebufferAbstract + Send + Sync>>,
    assemble_pass_fb: Option<Arc<FramebufferAbstract + Send + Sync>>,



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

        let current_dimensions = {
            settings
            .lock()
            .expect("failed to lock settings for frame creation")
            .get_dimensions()
        };

        let object_pass_images = ObjectPassImages::new(settings.clone(), &passes, device.clone());
        let post_pass_images = PostImages::new(settings.clone(), &passes, device.clone(), target_queue.clone());

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

        FrameSystem{

            dynamic_state: dynamic_state,

            engine_settings: settings,
            passes: passes,
            //Get created when starting a frame for later use
            object_pass_fb: None,
            blur_pass_h_fb: None,
            blur_pass_v_fb: None,
            assemble_pass_fb: None,

            object_pass_images: object_pass_images,
            post_pass_images: post_pass_images,

            device: device,
            queue: target_queue,
        }
    }

    ///Recreates all attachments with the right size
    pub fn recreate_attachments(&mut self){
        self.object_pass_images = ObjectPassImages::new(
            self.engine_settings.clone(),
            &self.passes,
            self.device.clone()
        );

        self.post_pass_images = PostImages::new(
            self.engine_settings.clone(),
            &self.passes,
            self.device.clone(),
            self.queue.clone()
        );

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


    ///Starts a new frame by taking a target image and starting a command buffer for it
    pub fn new_frame<I>(&mut self, target_image: I) -> FrameStage
    where I: ImageAccess + ImageViewAccess + Clone + Send + Sync + 'static
    {

        //Create the object pass frame buffer
        self.object_pass_fb = Some(Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.passes.object_pass.render_pass.clone())
            //the msaa image
            .add(self.object_pass_images.forward_hdr_image.clone()).expect("failed to add msaa image")
            //the multi sampled depth image
            .add(self.object_pass_images.forward_hdr_depth.clone()).expect("failed to add msaa depth buffer")
            //The hdr format
            .add(self.object_pass_images.hdr_fragments.clone()).expect("failed to add hdr_fragments image")
            //The color pass
            .add(self.object_pass_images.ldr_fragments.clone()).expect("failed to add image to frame buffer!")

            .build()
            .expect("failed to build main framebuffer!")
        ));

        //Now the blur pass frame buffer
        self.blur_pass_h_fb = Some(Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.passes.blur_pass.render_pass.clone())
            //Only writes to after h
            .add(self.post_pass_images.after_blur_h.clone()).expect("failed to add after_blur_h image")
            .build()
            .expect("failed to build main framebuffer!")
        ));


        //same for v pass
        self.blur_pass_v_fb = Some(Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.passes.blur_pass.render_pass.clone())
            //Only writes to after v
            .add(self.post_pass_images.after_blur_v.clone()).expect("failed to add after_blur_v image")
            .build()
            .expect("failed to build main framebuffer!")
        ));

        //The assemble stage reads the other pictures from descriptor sets and writes to the
        //swapchain image
        self.assemble_pass_fb = Some(Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.passes.assemble.render_pass.clone())
            //Only writes to after v
            .add(target_image).expect("failed to add assemble image")
            .build()
            .expect("failed to build assemble framebuffer!")
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
                    match self.object_pass_fb.take(){
                        Some(fb) => fb,
                        None => panic!("Could not find main frame buffer!"),
                    }
                };

                //For successfull clearing we generate a vector for all images.
                let clearing_values = vec![
                    [0.0, 0.0, 0.0, 1.0].into(), //forward color hdr
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
                //Starting the first blur pass
                let blur_h_fb = self.blur_pass_h_fb.take().expect("there was no blur_h image :(");
                let clearings = vec![
                    [0.0, 0.0, 0.0, 0.0].into()
                ];

                let next = cb
                .end_render_pass().expect("failed to end object pass")
                .begin_render_pass(blur_h_fb, false, clearings).expect("failed to start blur_h pass");

                FrameStage::BlurH(next)
            }

            FrameStage::BlurH(cb) => {
                //Starting the second blur pass
                let blur_v_fb = self.blur_pass_v_fb.take().expect("there was no blur_v image :(");
                let clearings = vec![
                    [0.0, 0.0, 0.0, 0.0].into()
                ];

                let next = cb
                .end_render_pass().expect("failed to end blur_h pass")
                .begin_render_pass(blur_v_fb, false, clearings).expect("failed to start blur_h pass");

                FrameStage::BlurV(next)
            }

            FrameStage::BlurV(cb)=> {

                let next = cb
                .end_render_pass().expect("failed to end blur_v pass");

                //Start the compute pass
                FrameStage::ComputeAverageLumiosity(next)
            }

            FrameStage::ComputeAverageLumiosity(cb) => {
                let assemble_fb = self.assemble_pass_fb.take()
                .expect("there was no assemble image");

                let clearings = vec![
                    [0.0, 0.0, 0.0, 0.0].into()
                ];
                //Begin the assamble pass
                let new_cb = cb.begin_render_pass(assemble_fb, false, clearings)
                .expect("failed to start assemble pass");

                FrameStage::Postprogress(new_cb)
            }

            FrameStage::Postprogress(cb) => {
                //Finish this frame
                let new = cb
                .end_render_pass().expect("failed to end command buffer");


                FrameStage::Finished(new)
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
