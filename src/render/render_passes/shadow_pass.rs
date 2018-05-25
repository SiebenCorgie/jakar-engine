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


///Collects the final shadow pass images
pub struct ShadowPassImages {
    pub directional_shadows: Arc<AttachmentImage<Format>>,
}

impl ShadowPassImages{
    pub fn new(
        settings: Arc<Mutex<EngineSettings>>,
        device: Arc<vulkano::device::Device>,
        depth_format: Format
    ) -> Self{

        let dimensions = {
            let mut settings_lck = settings.lock().expect("Failed to lock settings for shadow map size");
            let res = settings_lck
            .get_render_settings()
            .get_light_settings()
            .directional_settings.get_shadow_map_resolution();

            [res; 2]
        };

        let directional_image = AttachmentImage::sampled_input_attachment(
            device.clone(),
            dimensions,
            depth_format
        ).expect("failed to create hdr_fragments buffer!");


        ShadowPassImages{
            directional_shadows: directional_image,
        }
    }
}

#[derive(Clone)]
pub struct ShadowPass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
    images: Arc<ShadowPassImages>,
}

impl ShadowPass{
    pub fn new(
        settings: Arc<Mutex<EngineSettings>>,
        device: Arc<Device>,
        msaa_depth_format: Format
    ) -> Self{
        let render_pass = Arc::new(
            ordered_passes_renderpass!(device.clone(),
                attachments: {
                    //The depth image
                    out_depth: {
                        load: Clear,
                        store: Store,
                        format: msaa_depth_format,
                        samples: 1,
                    }
                },
                passes:[
                    //The actual pass
                    {
                        color: [],
                        depth_stencil: {out_depth},
                        input: []
                    }
                ]

            ).expect("failed to create main render_pass")
        );

        let images = Arc::new(
            ShadowPassImages::new(
                settings,
                device,
                msaa_depth_format
            )
        );

        ShadowPass{
            render_pass: render_pass,
            images,
        }
    }

    //returns the current images
    pub fn get_images(&self) -> Arc<ShadowPassImages>{
        self.images.clone()
    }

    //Recreates the images, only needed if the shadowmap resolution has changed
    pub fn rebuild_images(
        &mut self,
        settings: Arc<Mutex<EngineSettings>>,
        device: Arc<vulkano::device::Device>,
        depth_format: Format
    ){
        self.images = Arc::new(
            ShadowPassImages::new(
                settings,
                device,
                depth_format
            )
        );
    }

    ///Returns the framebuffer for the directional light shadows
    pub fn get_fb_directional(&self) -> Arc<FramebufferAbstract + Send + Sync>{
        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
            //Currently only has this single shadow map
            .add(self.images.directional_shadows.clone()).expect("failed to add assemble image")
            .build()
            .expect("failed to build assemble framebuffer!")
        )
    }
}
