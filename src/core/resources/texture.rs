use std::sync::{Arc};

use vulkano::image::immutable::ImmutableImage;
use vulkano::sampler::Filter;
use vulkano::sampler::SamplerAddressMode;
use vulkano::sampler::MipmapMode;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::sampler::Sampler;
use vulkano::image::Dimensions::Dim2d;
use vulkano::sync::GpuFuture;
use vulkano;

use image;
use image::DynamicImage::*;

pub struct TextureBuilder {
    //sampler
    //Sampling information if the image is larger or smaller than the original
    mag_filter: Filter,
    min_filter: Filter,
    //defines mipmapping mode
    mip_map_mode: MipmapMode,
    //defines how vulkano should handle U-V-W coordinates outside of 0.0-1.0
    address_u: SamplerAddressMode,
    address_v: SamplerAddressMode,
    address_w: SamplerAddressMode,

    // adds to the mip_mapping distance
    mip_lod_bias: f32,
    //set the filtering of this texture, this should usually be read from the settings
    max_anisotropy: f32,
    //Sets the max and min mipmapping level to use
    min_lod: f32,
    max_lod: f32,


    //image
    //Some helpful postprogressing
    b_blur: bool,
    blur_factor: f32,

    b_unsharpen: bool,
    sharp_factor: f32,
    sharp_threshold: i32,

    b_brighten: bool,
    brighten_factor: i32,

    b_flipv: bool,
    b_fliph: bool,

    b_rotate90: bool,
    b_rotate180: bool,
    b_rotate270: bool,

    //Create info (this won't be included in the final texture)
    image_path: String,
    //This is Some(data) if the image should be create from data
    image_data: Option<Vec<u8>>,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

///a small struct used to return image information
struct ImageInfo {
    pub dimensions: vulkano::image::Dimensions,
    pub format: vulkano::format::Format,
    pub data: Vec<u8>,
}


impl TextureBuilder {
    ///Creates a new builder struct with default parameters from an image at `image_path`
    pub fn from_image(
        image_path: &str,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Self{
        //Create the default builder
        TextureBuilder{
            //sampler
            //Sampling information if the image is larger or smaller than the original
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            //defines mipmapping mode
            mip_map_mode: MipmapMode::Nearest,
            //defines how vulkano should handle U-V-W coordinates outside of 0.0-1.0
            address_u: SamplerAddressMode::Repeat,
            address_v: SamplerAddressMode::Repeat,
            address_w: SamplerAddressMode::Repeat,

            // adds to the mip_mapping distance
            mip_lod_bias: 0.0,
            //set the filtering of this texture, this should usually be read from the settings
            max_anisotropy: 1.0,
            //Sets the max and min mipmapping level to use
            min_lod: 0.0,
            max_lod: 0.0,

            //image
            //Some helpful postprogressing
            b_blur: false,
            blur_factor: 0.0,

            b_unsharpen: false,
            sharp_factor: 0.0,
            sharp_threshold: 0,

            b_brighten: false,
            brighten_factor: 0,

            b_flipv: false,
            b_fliph: false,

            b_rotate90: false,
            b_rotate180: false,
            b_rotate270: false,

            //Create info (this won't be included in the final texture)
            image_path: String::from(image_path),
            image_data: None,
            device: device,
            queue: queue,
        }
    }

    ///Creates an image from provided data
    pub fn from_data<'a>(
        data: Vec<u8>,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> Self{
        //Create the default builder
        TextureBuilder{
            //sampler
            //Sampling information if the image is larger or smaller than the original
            mag_filter: Filter::Linear,
            min_filter: Filter::Linear,
            //defines mipmapping mode
            mip_map_mode: MipmapMode::Nearest,
            //defines how vulkano should handle U-V-W coordinates outside of 0.0-1.0
            address_u: SamplerAddressMode::Repeat,
            address_v: SamplerAddressMode::Repeat,
            address_w: SamplerAddressMode::Repeat,

            // adds to the mip_mapping distance
            mip_lod_bias: 0.0,
            //set the filtering of this texture, this should usually be read from the settings
            max_anisotropy: 1.0,
            //Sets the max and min mipmapping level to use
            min_lod: 0.0,
            max_lod: 0.0,

            //image
            //Some helpful postprogressing
            b_blur: false,
            blur_factor: 0.0,

            b_unsharpen: false,
            sharp_factor: 0.0,
            sharp_threshold: 0,

            b_brighten: false,
            brighten_factor: 0,

            b_flipv: false,
            b_fliph: false,

            b_rotate90: false,
            b_rotate180: false,
            b_rotate270: false,

            //Create info (this won't be included in the final texture)
            image_path: String::from("None"),
            image_data: Some(data),
            device: device,
            queue: queue,
        }
    }

    ///Sets new filtering technic for the sampler
    #[inline]
    pub fn with_sampling_filter(mut self, mag_filter: Filter, min_filter: Filter) -> Self{
        self.mag_filter = mag_filter;
        self.min_filter = min_filter;
        self
    }

    ///Sets new mipmapping mode for the sampler
    #[inline]
    pub fn with_mip_map_moe(mut self, new_mode: MipmapMode) -> Self{
        self.mip_map_mode = new_mode;
        self
    }

    ///Sets new tiling mode for the sampler
    #[inline]
    pub fn with_tiling_mode(
        mut self, u: SamplerAddressMode, v: SamplerAddressMode, w: SamplerAddressMode
    ) -> Self{
        self.address_u = u;
        self.address_v = v;
        self.address_w = w;
        self
    }
    ///Sets new mip lod bias for the sampler
    #[inline]
    pub fn with_lod_bias(mut self, bias: f32) -> Self{
        self.mip_lod_bias = bias;
        self
    }

    ///Sets new max anisotropic level for the sampler
    ///#panic This will panic if max < 1.0
    #[inline]
    pub fn with_max_anisotropy(mut self, max: f32) -> Self{
        //have to test that it is => 1.0 otherwise this will create a runtime error
        if max < 1.0 {
            panic!("The anisotropic max has to be equal ot larger than 1.0");
        }
        self.max_anisotropy = max;
        self
    }

    ///Sets new min and max mip map level for the sampler
    ///#panic this will panic if min is greater than max
    #[inline]
    pub fn with_min_and_max_mip_level(mut self, min: f32, max: f32) -> Self{
        //test min and max
        if min > max {
            panic!("the min mip map level has to be equal or smaller than the max level");
        }

        self.min_lod = min;
        self.max_lod = max;
        self
    }

    ///The imported image will be blured by `factor` after importing
    #[inline]
    pub fn with_blur(mut self, factor: f32) -> Self{
        self.b_blur = true;
        self.blur_factor = factor;
        self
    }

    ///The imported image get a unsharpen masked applied with the blur of `factor` and the sharpening of `thresold` after importing
    #[inline]
    pub fn with_unsharpening(mut self, factor: f32, thresold: i32) -> Self{
        self.b_unsharpen = true;
        self.sharp_factor = factor;
        self.sharp_threshold = thresold;
        self
    }

    ///The imported image will be brightened by `factor` after importing (tip the value can be)
    ///negative to darken the image
    #[inline]
    pub fn with_brightening(mut self, factor: i32) -> Self{
        self.b_brighten = true;
        self.brighten_factor = factor;
        self
    }

    ///this will flip the image vertically
    #[inline]
    pub fn with_flipped_v(mut self) -> Self{
        self.b_flipv = true;
        self
    }

    ///this will flip the image horizontally
    #[inline]
    pub fn with_flipped_h(mut self) -> Self{
        self.b_fliph = true;
        self
    }

    ///this will rotate the image 90 degree
    #[inline]
    pub fn with_rotation_90(mut self) -> Self{
        self.b_rotate90 = true;
        self
    }

    ///this will rotate the image 180 degree
    #[inline]
    pub fn with_rotation_180(mut self) -> Self{
        self.b_rotate180 = true;
        self
    }

    ///this will rotate the image 270 degree
    #[inline]
    pub fn with_rotation_270(mut self) -> Self{
        self.b_rotate270 = true;
        self
    }

    ///This function will use the information currently present in the `TextureBuilder`
    ///and create a `core::resources::Texture` from it
    pub fn build_with_name(self, texture_name: &str) -> Arc<Texture>
    {

        // This variable will be modified during the function, and will correspond to when the
        // transfer commands are finished.
        let final_future = Box::new(vulkano::sync::now(self.queue.device().clone())) as Box<vulkano::sync::GpuFuture>;

        //Setup a sampler from the info
        let tmp_sampler = Sampler::new(
            self.device.clone(),
            self.mag_filter,
            self.min_filter,
            self.mip_map_mode,
            self.address_u,
            self.address_v,
            self.address_w,
            self.mip_lod_bias,
            self.max_anisotropy,
            self.min_lod,
            self.max_lod,
        ).expect("Failed to generate sampler");
        //Now load a the texture
        let texture = {

            //first load the image
            let image = {
                //load the image::DynamicImage based on the type in the builder
                let mut image = {
                    match self.image_data{
                        Some(image_data) => {
                            //This image is some data buffer, will use this to load
                            //load with format from data
                            image::load_from_memory(&image_data)
                            .expect("failed to load image data based on guessing")
                        },
                        None => {
                            //There is no buffer, thats why we load it from the uri
                            image::open(&self.image_path.to_string())
                            .expect("failed to load png normal in creation")
                        }
                    }
                };

                //now apply, based on the settings all the post progressing
                //after applying everything we can convert the Dynamic image into the correct format
                //blur
                if self.b_blur {
                    image = image.blur(self.blur_factor);
                }
                //unsharpening
                if self.b_unsharpen {
                    image = image.unsharpen(self.sharp_factor, self.sharp_threshold);
                }
                //brighten
                if self.b_brighten {
                    image = image.brighten(self.brighten_factor);
                }
                //flipping
                if self.b_flipv{
                    image = image.flipv();
                }
                if self.b_fliph {
                    image = image.fliph();
                }
                //rotation 90-270 degree
                if self.b_rotate90 {
                    image = image.rotate90();
                }
                if self.b_rotate180 {
                    image = image.rotate180();
                }
                if self.b_rotate270 {
                    image = image.rotate270();
                }

                //now match the format of this image
                match image{
                    ImageLuma8(gray_image) => {
                        //Now transform the image::* into a vulkano image
                        let (width, height) = gray_image.dimensions();
                        let image_data = gray_image.into_raw().clone();
                        ImageInfo{
                            dimensions: Dim2d { width: width, height: height },
                            format: vulkano::format::Format::R8Unorm,
                            data: image_data,
                        }
                    },
                    ImageLumaA8(gray_alpha_image) => {
                        //Now transform the image::* into a vulkano image
                        let (width, height) = gray_alpha_image.dimensions();
                        let image_data = gray_alpha_image.into_raw().clone();
                        //(Dim2d { width: width, height: height },vulkano::format::R8G8Srgb, image_data)
                        ImageInfo{
                            dimensions: Dim2d { width: width, height: height },
                            format: vulkano::format::Format::R8G8Unorm,
                            data: image_data,
                        }
                    },
                    ImageRgb8(_) =>{
                        // Since RGB is often not supported by Vulkan, convert to RGBA instead.
                        let rgba = image.to_rgba();
                        //Now transform the image::* into a vulkano image
                        let (width, height) = rgba.dimensions();
                        let image_data = rgba.into_raw().clone();
                        //(Dim2d { width: width, height: height },vulkano::format::R8G8B8A8Srgb, image_data)
                        ImageInfo{
                            dimensions: Dim2d { width: width, height: height },
                            format: vulkano::format::Format::R8G8B8A8Unorm,
                            data: image_data,
                        }
                    },
                    ImageRgba8(grba_image) =>{
                        //Now transform the image::* into a vulkano image
                        let (width, height) = grba_image.dimensions();
                        let image_data = grba_image.into_raw().clone();
                        //(Dim2d { width: width, height: height },vulkano::format::R8G8B8A8Srgb, image_data)
                        ImageInfo{
                            dimensions: Dim2d { width: width, height: height },
                            format: vulkano::format::Format::R8G8B8A8Unorm,
                            data: image_data,
                        }
                    },
                }
            };
            //create a image from the optained format and resources
            let (texture_tmp, tex_future) = {
                ImmutableImage::from_iter(
                    image.data.iter().cloned(),
                    image.dimensions,
                    //Set format dependent on self.color_format
                    image.format,
                    self.queue.clone())
                .expect("failed to create immutable image")
            };
            //drop the future to wait for gpu
            let _  = Box::new(final_future.join(tex_future));

            texture_tmp
        };
        let texture_struct = Texture{
            name: String::from(texture_name),
            texture: texture,
            sampler: tmp_sampler,
            original_path: self.image_path.clone(),
        };

        Arc::new(texture_struct)
    }
}

///The Texture holds a images as well as the sampler, mipmapping etc for this texture is stored
/// withing the `vulkano::image::immutable::ImmutableImage`.
///Several textures can be compined in a material
#[derive(Clone)]
pub struct Texture {
    ///A name which can be used to reference the texture
    pub name: String,
    texture: Arc<ImmutableImage<vulkano::format::Format>>,
    sampler: Arc<vulkano::sampler::Sampler>,

    original_path: String,
}

///The implementation doesn't change anything on this texture
impl Texture{

    ///Returns the raw `Arc<ImmutableImage<T>>`
    #[inline]
    pub fn get_raw_texture(&self) -> Arc<ImmutableImage<vulkano::format::Format>>
    {
        self.texture.clone()
    }

    ///Returns the raw `Arc<vulkano::sampler::Sampler>`
    #[inline]
    pub fn get_raw_sampler(&self) -> Arc<vulkano::sampler::Sampler>{
        self.sampler.clone()
    }
}
