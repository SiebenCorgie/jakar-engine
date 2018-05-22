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


///Collects the two PostImages
pub struct PostImages {
    pub after_blur_h: Arc<ImageViewAccess  + Send + Sync>,
    pub after_blur_v: Arc<ImageViewAccess  + Send + Sync>,
    pub scaled_ldr_images: Vec<Arc<StorageImage<Format>>>,
}

impl PostImages{
    pub fn new(
        settings: Arc<Mutex<EngineSettings>>,
        hdr_msaa_format: Format,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
    ) -> Self{

        let current_dimensions = {
            settings
            .lock()
            .expect("failed to lock settings for frame creation")
            .get_dimensions()
        };

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

    ///Returns the framebuffer for writing the the horizontal blured images
    pub fn get_fb_blur_h(&self) -> Arc<FramebufferAbstract + Send + Sync>{
        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
            //Only writes to after h
            .add(self.images.after_blur_h.clone()).expect("failed to add after_blur_h image")
            .build()
            .expect("failed to build main framebuffer!")
        )
    }

    pub fn get_fb_blur_v(&self) -> Arc<FramebufferAbstract + Send + Sync> {
        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
            //Only writes to after v
            .add(self.images.after_blur_v.clone()).expect("failed to add after_blur_v image")
            .build()
            .expect("failed to build main framebuffer!")
        )
    }
}

///Takes currently:
/// - LDR Fragments
/// - Blured hdr fragments
///
/// Thoose are assembled to the final imaged and then tone mapping etc. is applied
#[derive(Clone)]
pub struct AssemblePass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
}

impl AssemblePass{
    pub fn new(device: Arc<Device>, swapchain_format: Format) -> Self{
        let render_pass = Arc::new(
            ordered_passes_renderpass!(device.clone(),
                attachments: {
                    final_image: {
                        load: Clear,
                        store: Store,
                        format: swapchain_format,
                        samples: 1,
                    }
                },
                passes:[
                    //The actual pass
                    {
                        color: [final_image],
                        depth_stencil: {},
                        input: []
                    }
                ]

            ).expect("failed to create main render_pass")
        );

        AssemblePass{
            render_pass: render_pass,
        }
    }
    ///Returns the framebuffer which will draw to a `sw_images` in your swapchain
    pub fn get_fb_assemble<I>(&self, sw_images: I) -> Arc<FramebufferAbstract + Send + Sync>
        where I: ImageAccess + ImageViewAccess + Clone + Send + Sync + 'static
    {
        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
            //Only writes to after v
            .add(sw_images).expect("failed to add assemble image")
            .build()
            .expect("failed to build assemble framebuffer!")
        )
    }
}

//TODO create pass

///A collection of the available render pass definitions.
#[derive(Clone)]
pub struct RenderPasses {

    //Local copy of the settings needed for fast rebuilding
    settings: Arc<Mutex<EngineSettings>>,
    device: Arc<Device>,
    queue: Arc<vulkano::device::Queue>,
    ///Is able to render objects and ouput the depth buffer
    pub shadow_pass: ShadowPass,

    ///Renderst the objects in a forward manor, in a second pass the msaa is resolved and the image
    /// is split in a hdr and ldr part.
    pub object_pass: ObjectPass,
    ///Blurs the first texture and writes it blured based on settings to the output
    pub blur_pass: BlurPass,
    ///Takes all the generated images and combines them to the final image
    pub assemble: AssemblePass,


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

        let msaa_factor = {
            let mut set_lck = settings.lock().expect("failed to lock settings");
            set_lck.get_render_settings().get_msaa_factor()
        };

        let shadow_pass = ShadowPass::new(settings.clone(), device.clone(), msaa_depth_format);
        let object_pass = ObjectPass::new(settings.clone(), device.clone(),  msaa_factor, hdr_msaa_format, msaa_depth_format);
        let blur_pass = BlurPass::new(settings.clone(), device.clone(), queue.clone(), hdr_msaa_format);
        let assemble = AssemblePass::new(device.clone(), swapchain_format);

        RenderPasses{
            settings,
            device,
            queue,
            shadow_pass: shadow_pass,
            object_pass: object_pass,
            blur_pass: blur_pass,
            assemble: assemble,

            image_hdr_msaa_format: hdr_msaa_format,
            image_msaa_depth_format: msaa_depth_format,
            swapchain_format: swapchain_format,
            static_msaa_factor: msaa_factor,
        }
    }

    ///Rebuilds the currently used images if needed. TODO, actually check what's needed, currently
    /// rebuilding all.
    pub fn rebuild_images(&mut self){
        self.object_pass.rebuild_images(
            self.settings.clone(),
            self.device.clone(),
            self.static_msaa_factor,
            self.image_hdr_msaa_format,
            self.image_msaa_depth_format,
        );

        self.blur_pass.rebuild_images(
            self.settings.clone(),
            self.device.clone(),
            self.queue.clone(),
            self.image_hdr_msaa_format
        );

        self.shadow_pass.rebuild_images(
            self.settings.clone(),
            self.device.clone(),
            self.image_msaa_depth_format,
        );
    }

    pub fn conf_to_pass(&self, conf: RenderPassConf) -> Arc<RenderPassAbstract + Send + Sync>{
        match conf{
            RenderPassConf::ShadowPass => self.shadow_pass.render_pass.clone(),
            RenderPassConf::ObjectPass => self.object_pass.render_pass.clone(),
            RenderPassConf::BlurPass => self.blur_pass.render_pass.clone(),
            RenderPassConf::AssemblePass => self.assemble.render_pass.clone(),
        }
    }

}

///Enum listing all the render passes availabe. They need to be known since we build pipelines against them
/// later.
#[derive(PartialEq, Clone)]
pub enum RenderPassConf{
    ///Currently renders everything, from the objects to the post progress.
    ShadowPass,
    ObjectPass,
    BlurPass,
    AssemblePass,
}
