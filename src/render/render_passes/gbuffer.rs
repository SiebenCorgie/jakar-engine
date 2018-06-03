use std::sync::{Arc, Mutex};

use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
use vulkano::image::traits::ImageViewAccess;
use vulkano::image::attachment::AttachmentImage;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::format::Format;
use vulkano::image::ImageUsage;
use vulkano::image::StorageImage;
use vulkano::image::Dimensions;
use vulkano;

use core::engine_settings::EngineSettings;

//Helper struct which collects the images needed for a single blur stage
#[derive(Clone)]
pub struct BlurStage {
    ///The source image which gets blured, inital color comes from a resize operation
    pub input_image: Arc<StorageImage<Format>>,
    ///After bluring the horizontal
    pub after_h_img: Arc<StorageImage<Format>>,
    ///After also bluring vertical and possibly adding a image from another stage
    pub final_image: Arc<StorageImage<Format>>
}

impl BlurStage{
    pub fn new(
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
        hdr_msaa_format: Format,
        dimensions: [u32; 2]
    ) -> Self{

        let dims = Dimensions::Dim2d{
            width: dimensions[0],
            height: dimensions[1],
        };

        let transfer_usage = ImageUsage{
            transfer_destination: true,
            transfer_source: true,
            color_attachment: true,
            sampled: true,
            storage: true,
            ..ImageUsage::none()
        };

        let input_image = StorageImage::with_usage(
            device.clone(),
            dims,
            hdr_msaa_format,
            transfer_usage,
            vec![queue.family()].into_iter()
        ).expect("failed to creat input image for blur stage");

        let after_h_img = StorageImage::with_usage(
            device.clone(),
            dims,
            hdr_msaa_format,
            transfer_usage,
            vec![queue.family()].into_iter()
        ).expect("failed to create after_h image for blur stage!");

        let final_image = StorageImage::with_usage(
            device.clone(),
            dims,
            hdr_msaa_format,
            transfer_usage,
            vec![queue.family()].into_iter()
        ).expect("failed to create final image for blur stage!");

        BlurStage{
            input_image: input_image,
            after_h_img: after_h_img,
            final_image: final_image,
        }
    }
}

///Collects all the images we render. If the engine settings or resolution of an image changes it will
/// Recreate the attachment.
#[derive(Clone)]
pub struct GBuffer {
    ///Recreation infos
    settings: Arc<Mutex<EngineSettings>>,
    device: Arc<Device>,
    queue: Arc<vulkano::device::Queue>,
    msaa_factor: u32,
    hdr_msaa_format: Format,
    msaa_depth_format: Format,
    shadow_depth_format: Format,


    //Shadows
    pub directional_shadow_map: Arc<AttachmentImage<Format>>,
    //Non else atm

    ///The actual gbuffer images

    //The buffer to which the multi sampled depth gets written
    pub forward_depth: Arc<ImageViewAccess + Send + Sync>,
    ///Holds the raw multisampled hdr diffuse buffer. The alpha is the amount of ambient occlusion
    pub forward_diffuse: Arc<ImageViewAccess + Send + Sync>,
    ///Holds only the hdr fragments used for the bloom. Those are scaled down by the bloom scale.
    pub hdr_fragments: Arc<AttachmentImage<Format>>,
    ///The resolved diffuse/ambient buffer
    pub diffuse_ambient: Arc<AttachmentImage<Format>>,
    ///TODO add normal+metallic as well as subsurface+roughness1 buffer later.


    ///PostProgressImages
    pub scaled_hdr: Vec<BlurStage>,
    pub scaled_ldr: Vec<Arc<StorageImage<Format>>>,
}


impl GBuffer{
    ///Creates a new GBuffer from the supplied settings. When settings change the GBuffer can be
    /// updated from the renderpass via `update_x()`.
    pub fn new(
        settings: Arc<Mutex<EngineSettings>>,
        device: Arc<Device>,
        queue: Arc<vulkano::device::Queue>,
        msaa_factor: u32,
        hdr_msaa_format: Format,
        msaa_depth_format: Format,
        shadow_depth_format: Format,
    ) -> Self{

        let current_dimensions = {
            settings
            .lock()
            .expect("failed to lock settings for frame creation")
            .get_dimensions()
        };

        let hdr_ldr_blit_usage = ImageUsage{
            transfer_source: true,
            sampled: true,
            color_attachment: true,
            input_attachment: true,
            ..ImageUsage::none()
        };


        let directional_shadow_map = create_shadow_maps(
            settings.clone(),
            device.clone(),
            shadow_depth_format
        );

        //multisampled depth
        let forward_depth = AttachmentImage::transient_multisampled_input_attachment(
            device.clone(),
            current_dimensions,
            msaa_factor,
            msaa_depth_format
        ).expect("failed to create forward_hdr_depth buffer!");
        //mutlisampled difffuse/ao
        let forward_diffuse = AttachmentImage::transient_multisampled_input_attachment(device.clone(),
            current_dimensions,
            msaa_factor,
            hdr_msaa_format
        ).expect("failed to create raw_render_color buffer!");
        //resolved hdr-only fragments
        let hdr_fragments = AttachmentImage::with_usage(device.clone(),
            current_dimensions,
            hdr_msaa_format,
            hdr_ldr_blit_usage.clone()
        ).expect("failed to create hdr_fragments buffer!");
        //resolved diffuse/ao
        let diffuse_ambient = AttachmentImage::with_usage(device.clone(),
            current_dimensions,
            hdr_msaa_format,
            hdr_ldr_blit_usage.clone()
        ).expect("failed to create ldr_fragments buffer!");

        let scaled_hdr = create_blur_level(
            settings.clone(),
            device.clone(),
            queue.clone(),
            hdr_msaa_format
        );

        let scaled_ldr = create_ldr_level(
            settings.clone(),
            device.clone(),
            queue.clone(),
            hdr_msaa_format
        );

        GBuffer {
            ///Recreation infos
            settings,
            device,
            queue,
            msaa_factor,
            hdr_msaa_format,
            msaa_depth_format,
            shadow_depth_format,

            //Shadows
            directional_shadow_map,
            //Non else atm

            ///The actual gbuffer images

            //The buffer to which the multi sampled depth gets written
            forward_depth,
            ///Holds the raw multisampled hdr diffuse buffer. The alpha is the amount of ambient occlusion
            forward_diffuse,
            ///Holds only the hdr fragments used for the bloom. Those are scaled down by the bloom scale.
            hdr_fragments,
            ///The resolved diffuse/ambient buffer
            diffuse_ambient,
            ///TODO add normal+metallic as well as subsurface+roughness1 buffer later.


            ///PostProgressImages
            scaled_hdr,
            scaled_ldr,
        }
    }
    ///Returns the framebuffer for writing the the horizontal blured images for the level at idx.
    /// Returns the last / samlest level if the idx is too big.
    pub fn get_fb_blur_h(
        &self,
        idx: usize,
        renderpass: Arc<RenderPassAbstract + Send + Sync>
    ) -> Arc<FramebufferAbstract + Send + Sync>{

        if self.scaled_hdr.is_empty(){
            panic!("The bloom images are empty, that should not happen");
        }

        let mut index = idx;
        if index > self.scaled_hdr.len() - 1{
            index = self.scaled_hdr.len() - 1;
        }

        let after_h_image = self.scaled_hdr[index].after_h_img.clone();

        Arc::new(
            vulkano::framebuffer::Framebuffer::start(renderpass)
            //Only writes to after h
            .add(after_h_image)
            .expect("failed to add after_blur_h image")
            .build()
            .expect("failed to build main framebuffer!")
        )
    }
    ///Retuirns the framebuffer to blur the h_blured image to the final stage
    pub fn get_fb_to_final(
        &self,
        idx: usize,
        renderpass: Arc<RenderPassAbstract + Send + Sync>
    ) -> Arc<FramebufferAbstract + Send + Sync> {

        if self.scaled_hdr.is_empty(){
            panic!("The bloom images are empty, that should not happen");
        }

        let mut index = idx;
        if index > self.scaled_hdr.len() - 1{
            index = self.scaled_hdr.len() - 1;
        }

        let final_image = self.scaled_hdr[index].final_image.clone();

        Arc::new(
            vulkano::framebuffer::Framebuffer::start(renderpass)
            //Only writes to after v
            .add(final_image)
            .expect("failed to add final blured image")
            .build()
            .expect("failed to build main framebuffer!")
        )
    }
}

///A helper function to not pollude the new function too much
fn create_blur_level(
    settings: Arc<Mutex<EngineSettings>>,
    device: Arc<Device>,
    queue: Arc<vulkano::device::Queue>,
    hdr_msaa_format: Format,
) -> Vec<BlurStage>{
    let (current_dimensions, mut bloom_level) = {
        let set_lck = settings
        .lock()
        .expect("failed to lock settings for frame creation");

        let dur_dim = set_lck
        .get_dimensions();

        let bloom_lvl = set_lck.get_render_settings().get_bloom().levels;


        (dur_dim, bloom_lvl)
    };

    //Always do at least one bloom level if activated
    if bloom_level <= 0{
        bloom_level = 1;
    }

    //For the bloom we want to scale the hdr_frags only image some levels down, blur each of them,
    // and add them back together.
    let mut blur_level_dim = [
        current_dimensions[0] / 2,
        current_dimensions[1] / 2
    ];
    let mut bloom_images = Vec::new();

    //Create new image for each level
    for _ in  0..bloom_level{
        //Thoose will eb created in a loop as well
        let blur_image = BlurStage::new(
            device.clone(),
            queue.clone(),
            hdr_msaa_format,
            blur_level_dim
        );
        //Store
        bloom_images.push(blur_image);
        //Update the dims and exit if too small
        blur_level_dim = [blur_level_dim[0] / 2, blur_level_dim[1] / 2];

        if blur_level_dim[0] < 1 || blur_level_dim[1] < 1 {
            //Dims too small, break
            break;
        }
    }

    println!("Created {:?} blur images", bloom_level);

    bloom_images
}

///A helperfunction to create enough images to blit the diffuse buffer from the current resolution to a 1x1 image.
fn create_ldr_level(
    settings: Arc<Mutex<EngineSettings>>,
    device: Arc<Device>,
    queue: Arc<vulkano::device::Queue>,
    hdr_msaa_format: Format,
) -> Vec<Arc<StorageImage<Format>>>{
    let current_dimensions = {
        let set_lck = settings
        .lock()
        .expect("failed to lock settings for frame creation");

        let dur_dim = set_lck
        .get_dimensions();

        dur_dim
    };

    //Now generate the targets for the average lumiosity pass.
    let transfer_usage = ImageUsage{
        transfer_destination: true,
        transfer_source: true,
        color_attachment: true,
        sampled: true,
        storage: true,
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

    scaled_ldr_images
}

///Helper function to create all shadowmaps
fn create_shadow_maps(
    settings: Arc<Mutex<EngineSettings>>,
    device: Arc<vulkano::device::Device>,
    depth_format: Format
) -> Arc<AttachmentImage<Format>>{
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

    directional_image
}
