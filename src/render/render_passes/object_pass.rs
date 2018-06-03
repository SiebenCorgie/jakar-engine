use std::sync::{Arc, Mutex};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
use vulkano::image::traits::ImageViewAccess;
use vulkano::image::attachment::AttachmentImage;
use vulkano::format::Format;
use vulkano::image::ImageUsage;
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
    pub hdr_fragments: Arc<AttachmentImage<Format>>,
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

        //Use custom usage parameter since both are at some time used for a bliting operation
        let hdr_fragments = AttachmentImage::with_usage(device.clone(),
            current_dimensions,
            hdr_msaa_format,
            transfer_usage.clone()
        ).expect("failed to create hdr_fragments buffer!");

        let ldr_fragments = AttachmentImage::with_usage(device.clone(),
            current_dimensions,
            hdr_msaa_format,
            transfer_usage.clone()
        ).expect("failed to create ldr_fragments buffer!");

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
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,}

impl ObjectPass{
    pub fn new(
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
        ObjectPass{
            render_pass: main_renderpass
        }
    }
}
