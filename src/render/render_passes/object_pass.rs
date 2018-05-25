use std::sync::{Arc, Mutex};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
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

use core::engine_settings::EngineSettings;

///Collects the images from the MainRenderPass
#[derive(Clone)]
pub struct ObjectPassImages {
    //The buffer to which the multi sampled depth gets written
    pub forward_hdr_depth: Arc<ImageViewAccess + Send + Sync>,
    //Holds the raw multisampled hdr colors
    pub forward_hdr_image: Arc<ImageViewAccess + Send + Sync>,
    //Adter sorting the hdr fragments (used for bluring)
    pub hdr_fragments: Arc<ImageViewAccess + Send + Sync>,
    //The ldr fragments
    //pub ldr_fragments: Arc<ImageViewAccess + Send + Sync>,
    pub ldr_fragments: Arc<AttachmentImage<Format>>,

    pub transfer_usage: ImageUsage,
}

impl ObjectPassImages{
    pub fn new(
        settings: Arc<Mutex<EngineSettings>>,
        device: Arc<vulkano::device::Device>,
        msaa_factor: u32,
        hdr_msaa_format: Format,
        msaa_depth_format: Format
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

        //Creates a buffer for the msaa image
        let forward_hdr_color = AttachmentImage::transient_multisampled_input_attachment(device.clone(),
        current_dimensions, msaa_factor,
        hdr_msaa_format).expect("failed to create raw_render_color buffer!");


        //Create a multisampled depth buffer depth buffer
        let forward_hdr_depth = AttachmentImage::transient_multisampled_input_attachment(
            device.clone(), current_dimensions, msaa_factor, msaa_depth_format)
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

#[derive(Clone)]
pub struct ObjectPass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
    images: Arc<ObjectPassImages>,
}

impl ObjectPass{
    pub fn new(
        settings: Arc<Mutex<EngineSettings>>,
        device: Arc<Device>,
        msaa_factor: u32,
        hdr_msaa_format: Format,
        msaa_depth_format: Format
    ) -> Self{

        //Setup the render_pass layout for the forward pass
        let main_renderpass = Arc::new(
            ordered_passes_renderpass!(device.clone(),
                attachments: {
                    // The first framebuffer attachment is the raw_render_color image.
                    raw_render_color: {
                        load: Clear,
                        store: Store,
                        format: hdr_msaa_format, //Defined that it works by the vulkan implementation
                        samples: msaa_factor,     // This has to match the image definition.
                    },

                    //the second one is the msaa depth buffer
                    raw_render_depth: {
                        load: Clear,
                        store: DontCare,
                        format: msaa_depth_format, //works per vulkan definition
                        samples: msaa_factor,
                    },

                    //Holds only fragments with at leas one value over 1.0
                    hdr_fragments: {
                        load: Clear,
                        store: Store,
                        format: hdr_msaa_format,
                        samples: 1,
                    },
                    //The non hdr framgnets
                    ldr_fragments: {
                        load: Clear,
                        store: Store,
                        format: hdr_msaa_format,
                        samples: 1,
                    }
                },
                passes:[
                    {
                        color: [raw_render_color],
                        depth_stencil: {raw_render_depth},
                        input: []
                    },

                    //Resolves msaa and creates a HDR fragment buffer
                    {
                        color: [ldr_fragments, hdr_fragments],
                        depth_stencil: {},
                        input: [raw_render_color]
                    }
                ]

            ).expect("failed to create main render_pass")
        );

        let images = ObjectPassImages::new(
            settings,
            device,
            msaa_factor,
            hdr_msaa_format,
            msaa_depth_format
        );
        ObjectPass{
            render_pass: main_renderpass,
            images: Arc::new(images),
        }
    }

    ///Returns the current images of this render_pass
    pub fn get_images(&self) -> Arc<ObjectPassImages>{
        self.images.clone()
    }

    ///Rebuilds the current images
    pub fn rebuild_images(
        &mut self,
        settings: Arc<Mutex<EngineSettings>>,
        device: Arc<Device>,
        msaa_factor: u32,
        hdr_msaa_format: Format,
        msaa_depth_format: Format
    ){
        self.images = Arc::new(
            ObjectPassImages::new(
                settings,
                device,
                msaa_factor,
                hdr_msaa_format,
                msaa_depth_format
            )
        );
    }

    ///Returns a framebuffer which can be used for this renderpass
    pub fn get_framebuffer(&self) -> Arc<FramebufferAbstract + Send + Sync>{
        //Create the object pass frame buffer
        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
            //the msaa image
            .add(self.images.forward_hdr_image.clone()).expect("failed to add msaa image")
            //the multi sampled depth image
            .add(self.images.forward_hdr_depth.clone()).expect("failed to add msaa depth buffer")
            //The hdr format
            .add(self.images.hdr_fragments.clone()).expect("failed to add hdr_fragments image")
            //The color pass
            .add(self.images.ldr_fragments.clone()).expect("failed to add image to frame buffer!")

            .build()
            .expect("failed to build main framebuffer!")
        )
    }

}
