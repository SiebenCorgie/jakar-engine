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
