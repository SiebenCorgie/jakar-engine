use std::sync::{Arc, Mutex};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
use vulkano::image::StorageImage;
use vulkano::format::Format;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano;

use core::engine_settings::EngineSettings;

///Handles the shadowmaps as well as the renderpass which renders the directional and spot/point
/// light images
pub mod shadow_pass;

///Handles the forward object rendering as well as the sorting of hdr elements into a second hdr fragments only buffer
pub mod object_pass;

///A pass which can blur an image, is used for the bloom images.
pub mod blur_pass;

///Collects all images needed for the post progress.
//pub mod post_images;

///Takes the postprogressed image and the hdr image of the object pass, does tonemapping and assembles the image into a
/// final image which will be written to the swapchain image and later displayed.
pub mod assemble_pass;
///Collects all images needed in one render pass.
pub mod gbuffer;


///A collection of the available render pass definitions.
#[derive(Clone)]
pub struct RenderPasses {

    //Local copy of the settings needed for fast rebuilding
    settings: Arc<Mutex<EngineSettings>>,
    device: Arc<Device>,
    queue: Arc<vulkano::device::Queue>,
    ///Is able to render objects and ouput the depth buffer
    pub shadow_pass: shadow_pass::ShadowPass,

    ///Renders the objects in a forward manor, in a second pass the msaa is resolved and the image
    /// is split in a hdr and ldr part.
    pub object_pass: object_pass::ObjectPass,
    ///Blurs the first texture and writes it blured based on settings to the output
    pub blur_pass: blur_pass::BlurPass,
    ///Takes all the generated images and combines them to the final image
    pub assemble: assemble_pass::AssemblePass,

    ///Collects all images used in the passes.
    pub gbuffer: gbuffer::GBuffer,

    //Holds all the used formats
    pub image_hdr_msaa_format: Format,
    pub image_msaa_depth_format: Format,
    pub swapchain_format: Format,
    pub static_msaa_factor: u32,
}



impl RenderPasses{
    pub fn new(
        device: Arc<Device>,
        queue: Arc<vulkano::device::Queue>,
        swapchain_format: Format,
        settings: Arc<Mutex<EngineSettings>>
    ) -> Self{

        let hdr_msaa_format = vulkano::format::Format::R16G16B16A16Sfloat;
        let msaa_depth_format = vulkano::format::Format::D16Unorm;
        let shadow_depth_format = vulkano::format::Format::D16Unorm;

        let msaa_factor = {
            let mut set_lck = settings.lock().expect("failed to lock settings");
            set_lck.get_render_settings().get_msaa_factor()
        };

        let shadow_pass = shadow_pass::ShadowPass::new(device.clone(), shadow_depth_format);
        let object_pass = object_pass::ObjectPass::new(device.clone(),  msaa_factor, hdr_msaa_format, msaa_depth_format);
        let blur_pass = blur_pass::BlurPass::new(device.clone(), hdr_msaa_format);
        let assemble = assemble_pass::AssemblePass::new(device.clone(), swapchain_format);

        let gbuffer = gbuffer::GBuffer::new(
            settings.clone(),
            device.clone(),
            queue.clone(),
            msaa_factor,
            hdr_msaa_format,
            msaa_depth_format,
            shadow_depth_format
        );


        RenderPasses{
            settings,
            device,
            queue,
            shadow_pass: shadow_pass,
            object_pass: object_pass,
            blur_pass: blur_pass,
            assemble: assemble,

            gbuffer,

            image_hdr_msaa_format: hdr_msaa_format,
            image_msaa_depth_format: msaa_depth_format,
            swapchain_format: swapchain_format,
            static_msaa_factor: msaa_factor,
        }
    }

    ///Rebuilds the currently used images if needed. TODO, actually check what's needed, currently
    /// rebuilding all.
    pub fn rebuild_images(&mut self){
        //TODO reimplement with checks
    }

    ///Returns the render pass and its subpass id for this configuratiuon
    pub fn conf_to_pass(&self, conf: RenderPassConf) -> (Arc<RenderPassAbstract + Send + Sync>, u32){
        match conf{
            RenderPassConf::ShadowPass => (self.shadow_pass.render_pass.clone(), 0),
            RenderPassConf::ObjectPass(subpass) => match subpass{
                ObjectPassSubPasses::ForwardRenderingPass => (self.object_pass.render_pass.clone(), 0),
                ObjectPassSubPasses::HdrSortingPass => (self.object_pass.render_pass.clone(), 1),
            },
            RenderPassConf::BlurPass => (self.blur_pass.render_pass.clone(), 0),
            RenderPassConf::AssemblePass =>(self.assemble.render_pass.clone(), 0),
        }
    }

    ///Returns the framebuffer for the forward pass
    pub fn get_forward_framebuff(&self) -> Arc<FramebufferAbstract + Send + Sync>{
        //Create the object pass frame buffer
        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.object_pass.render_pass.clone())
            //the msaa image
            .add(self.gbuffer.forward_diffuse.clone()).expect("failed to add msaa image")
            //the multi sampled depth image
            .add(self.gbuffer.forward_depth.clone()).expect("failed to add msaa depth buffer")
            //The hdr format
            .add(self.gbuffer.hdr_fragments.clone()).expect("failed to add hdr_fragments image")
            //The color pass
            .add(self.gbuffer.diffuse_ambient.clone()).expect("failed to add image to frame buffer!")

            .build()
            .expect("failed to build main framebuffer!")
        )
    }

    ///Returns the framebuffer for the directional light shadows
    pub fn get_framebuff_directional(&self) -> Arc<FramebufferAbstract + Send + Sync>{
        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.shadow_pass.render_pass.clone())
            //Currently only has this single shadow map
            .add(self.gbuffer.directional_shadow_map.clone()).expect("failed to add assemble image")
            .build()
            .expect("failed to build assemble framebuffer!")
        )
    }


    ///Returns the final blur image. *This could be not the first image in the bloom stack!*
    pub fn get_final_bloom_img(&self) -> Arc<StorageImage<Format>>{
        let biggest_blured_image = {
            self.settings
            .lock().expect("failed to lock settings")
            .get_render_settings().get_bloom().first_bloom_level as usize
        };
        self.gbuffer.scaled_hdr[biggest_blured_image].final_image.clone()
    }

    ///Returns the framebuffer to blur the input image at `idx` to the blur_h image
    pub fn get_framebuff_blur_h(&self, idx: usize) -> Arc<FramebufferAbstract + Send + Sync>{
        self.gbuffer.get_fb_blur_h(
            idx,
            self.blur_pass.render_pass.clone()
        )
    }

    ///Returns the framebuffer to blur the blur_h image at `idx` to the final image
    pub fn get_framebuff_blur_final(&self, idx: usize) -> Arc<FramebufferAbstract + Send + Sync>{
        self.gbuffer.get_fb_to_final(
            idx,
            self.blur_pass.render_pass.clone()
        )
    }

}

///Collection of all subpasses in the object pass
#[derive(PartialEq, Clone)]
pub enum ObjectPassSubPasses {
    ForwardRenderingPass,
    HdrSortingPass,
}

///Enum listing all the render passes availabe. They need to be known since we build pipelines against them
/// later.
#[derive(PartialEq, Clone)]
pub enum RenderPassConf{
    ///Currently renders everything, from the objects to the post progress.
    ShadowPass,
    ObjectPass(ObjectPassSubPasses),
    BlurPass,
    AssemblePass,
}
