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
        settings: Arc<Mutex<EngineSettings>>,
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
        /*
        let after_h_img = AttachmentImage::sampled_input_attachment(
            device.clone(),
            dimensions,
            hdr_msaa_format
        ).expect("failed to create after_h image for blur stage!");

        let final_image = AttachmentImage::sampled_input_attachment(
            device.clone(),
            dimensions,
            hdr_msaa_format
        ).expect("failed to create final image for blur stage!");
        */
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

///Collects the two PostImages
pub struct PostImages {
    pub bloom: Vec<BlurStage>,
    pub scaled_ldr_images: Vec<Arc<StorageImage<Format>>>,
}

impl PostImages{
    pub fn new(
        settings: Arc<Mutex<EngineSettings>>,
        hdr_msaa_format: Format,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
    ) -> Self{

        let (current_dimensions, mut bloom_level, initial_scale_down) = {
            let set_lck = settings
            .lock()
            .expect("failed to lock settings for frame creation");

            let dur_dim = set_lck
            .get_dimensions();

            let bloom_lvl = set_lck.get_render_settings().get_bloom().levels;

            let scale_down = set_lck.get_render_settings().get_bloom().initial_scale_down;

            (dur_dim, bloom_lvl, scale_down)
        };

        //Always do at least one bloom level if activated
        if bloom_level <= 0{
            bloom_level = 1;
        }

        //For the bloom we want to scale the hdr_frags only image some levels down, blur each of them,
        // and add them back together.
        let mut blur_level_dim = [
            current_dimensions[0] / initial_scale_down,
            current_dimensions[1] / initial_scale_down
        ];
        let mut bloom_images = Vec::new();

        //Create new image for each level
        for idx in  0..bloom_level{
            //Thoose will eb created in a loop as well
            let blur_image = BlurStage::new(
                settings.clone(),
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


        PostImages{
            bloom: bloom_images,
            scaled_ldr_images: scaled_ldr_images,
        }
    }
    ///Returns the final bloom image
    pub fn get_final_bloom_img(&self) -> Arc<StorageImage<Format>>{
        self.bloom[0].final_image.clone()
    }
}
