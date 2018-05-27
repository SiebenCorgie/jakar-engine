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
use render::render_passes::post_images::PostImages;




///Is able to blur fragments based on settings supplied with the first descriptor set
#[derive(Clone)]
pub struct BlurPass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
    images: Arc<PostImages>,
}

impl BlurPass {
    pub fn new(
        settings: Arc<Mutex<EngineSettings>>,
        device: Arc<Device>,
        queue: Arc<vulkano::device::Queue>,
        hdr_msaa_format: Format,
    ) -> Self{
        let render_pass = Arc::new(
            ordered_passes_renderpass!(device.clone(),
                attachments: {
                    //The blured fragments
                    out_hdr_fragments: {
                        load: Clear,
                        store: Store,
                        format: hdr_msaa_format,
                        samples: 1,
                    }
                },
                passes:[
                    //The actual pass
                    {
                        color: [out_hdr_fragments],
                        depth_stencil: {},
                        input: []
                    }
                ]

            ).expect("failed to create main render_pass")
        );

        let images = PostImages::new(
            settings,
            hdr_msaa_format,
            device,
            queue
        );

        BlurPass{
            render_pass: render_pass,
            images: Arc::new(images)
        }
    }

    ///Returns the current images
    pub fn get_images(&self) -> Arc<PostImages>{
        self.images.clone()
    }

    ///Rebuilds the current images
    pub fn rebuild_images(
        &mut self,
        settings: Arc<Mutex<EngineSettings>>,
        device: Arc<Device>,
        queue: Arc<vulkano::device::Queue>,
        hdr_msaa_format: Format,
    ){
        self.images = Arc::new(
            PostImages::new(
                settings,
                hdr_msaa_format,
                device,
                queue
            )
        );
    }

    ///Returns the framebuffer for writing the the horizontal blured images for the level at idx.
    /// Returns the last / samlest level if the idx is too big.
    pub fn get_fb_blur_h(&self, idx: usize) -> Arc<FramebufferAbstract + Send + Sync>{

        if self.images.bloom.is_empty(){
            panic!("The bloom images are empty, that should not happen");
        }

        let mut index = idx;
        if index > self.images.bloom.len() - 1{
            index = self.images.bloom.len() - 1;
        }

        let after_h_image = self.images.bloom[index].after_h_img.clone();

        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
            //Only writes to after h
            .add(after_h_image)
            .expect("failed to add after_blur_h image")
            .build()
            .expect("failed to build main framebuffer!")
        )
    }

    pub fn get_fb_to_final(&self, idx: usize) -> Arc<FramebufferAbstract + Send + Sync> {

        if self.images.bloom.is_empty(){
            panic!("The bloom images are empty, that should not happen");
        }

        let mut index = idx;
        if index > self.images.bloom.len() - 1{
            index = self.images.bloom.len() - 1;
        }

        let final_image = self.images.bloom[index].final_image.clone();

        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
            //Only writes to after v
            .add(final_image)
            .expect("failed to add final blured image")
            .build()
            .expect("failed to build main framebuffer!")
        )
    }
}
