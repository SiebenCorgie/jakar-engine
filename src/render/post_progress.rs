
use render::shader::shaders::default_pstprg_fragment;
use render::shader::shaders::blur;
use render::pipeline;
use render::frame_system::FrameStage;
use render::frame_system::FrameSystem;
use core::engine_settings;

use vulkano;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::sampler::Sampler;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::buffer::DeviceLocalBuffer;
use vulkano::buffer::BufferUsage;
use vulkano::image::traits::ImageAccess;
use vulkano::sampler::Filter;
use vulkano::sampler::MipmapMode;
use vulkano::sampler::SamplerAddressMode;

use vulkano::image::ImageDimensions;

use std::sync::{Arc, Mutex};


///Should be used in screenspace
#[derive(Clone,Copy)]
pub struct PostProgressVertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}

impl PostProgressVertex{
    pub fn new(pos: [f32; 2], uv: [f32;2]) -> Self{
        PostProgressVertex {
            position: pos,
            tex_coord: uv,
        }
    }
}

//Implements the vulkano::vertex trait on Vertex
impl_vertex!(PostProgressVertex, position, tex_coord);


///Is able to perform the post progressing on a command buffer based on a stored pipeline
pub struct PostProgress{
    engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
    //device: Arc<vulkano::device::Device>,

    pipeline: Arc<pipeline::Pipeline>,
    resolve_pipe: Arc<pipeline::Pipeline>,
    //Used for the average compute pass
    average_pipe: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
    blur_pipe: Arc<pipeline::Pipeline>,


    screen_vertex_buffer: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    //Stores the future "current" and the value of the last frame
    average_buffer: Arc<DeviceLocalBuffer<average_lumiosity_compute_shader::ty::LumiosityBuffer>>,
    average_set_pool: FixedSizeDescriptorSetsPool<Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>>,

    screen_sampler: Arc<Sampler>,
    hdr_settings_pool: vulkano::buffer::cpu_pool::CpuBufferPool<default_pstprg_fragment::ty::hdr_settings>,
    blur_settings_pool: vulkano::buffer::cpu_pool::CpuBufferPool<blur::ty::blur_settings>,
    exposure_settings_pool: vulkano::buffer::cpu_pool::CpuBufferPool<average_lumiosity_compute_shader::ty::ExposureSettings>,
}


impl PostProgress{
    ///Create the postprogressing chain
    pub fn new(
        engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
        post_progress_pipeline: Arc<pipeline::Pipeline>,
        resolve_pipe: Arc<pipeline::Pipeline>,
        blur_pipe: Arc<pipeline::Pipeline>,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
        //passes: &render_passes::RenderPasses,
    ) -> Self{
        //generate a vertex buffer
        let mut vertices: Vec<PostProgressVertex> = Vec::new();
        //the screen space vertexes
        vertices.push(PostProgressVertex::new([-1.0; 2], [0.0; 2]));
        vertices.push(PostProgressVertex::new([-1.0, 1.0], [0.0, 1.0]));
        vertices.push(PostProgressVertex::new([1.0; 2], [1.0; 2]));

        vertices.push(PostProgressVertex::new([-1.0; 2], [0.0; 2]));
        vertices.push(PostProgressVertex::new([1.0, -1.0], [1.0, 0.0]));
        vertices.push(PostProgressVertex::new([1.0; 2], [1.0; 2]));

        let sample_vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
                                    ::from_iter(device.clone(), vulkano::buffer::BufferUsage::all(), vertices.iter().cloned())
                                    .expect("failed to create buffer");

        //we also have to maintain a buffer pool for the settings which can potentually change
        let hdr_settings_pool = CpuBufferPool::<default_pstprg_fragment::ty::hdr_settings>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );

        let blur_pool = CpuBufferPool::<blur::ty::blur_settings>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );

        //The compute stuff...
        let compute_shader = Arc::new(average_lumiosity_compute_shader::Shader::load(device.clone())
            .expect("failed to create compute shader module"));

        let average_pipe: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync> = Arc::new(
            vulkano::pipeline::ComputePipeline::new(
                device.clone(), &compute_shader.main_entry_point(), &()
            ).expect("failed to build average pipeline")
        );

        let average_buffer = DeviceLocalBuffer::new(
            device.clone(), BufferUsage::all(), vec![queue.family()].into_iter()
        ).expect("failed to create average lumiosity buffer!");

        let average_set_pool = FixedSizeDescriptorSetsPool::new(average_pipe.clone(), 0);

        let exp_set_pool = CpuBufferPool::<average_lumiosity_compute_shader::ty::ExposureSettings>::new(
            device.clone(), vulkano::buffer::BufferUsage::all());

        let screen_sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Linear,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            0.0,
            1.0,
            1.0,
            1.0,
        ).expect("failed to create screen sampler");

        PostProgress{
            engine_settings: engine_settings,
            //device: device,

            pipeline: post_progress_pipeline,
            resolve_pipe: resolve_pipe,
            average_pipe: average_pipe,
            blur_pipe: blur_pipe,

            screen_vertex_buffer: sample_vertex_buffer,
            average_buffer: average_buffer,
            average_set_pool: average_set_pool,

            screen_sampler: screen_sampler,
            hdr_settings_pool: hdr_settings_pool,
            blur_settings_pool: blur_pool,
            exposure_settings_pool: exp_set_pool,
        }
    }

    fn get_hdr_settings(&mut self) -> vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer
    <default_pstprg_fragment::ty::hdr_settings, Arc<vulkano::memory::pool::StdMemoryPool>> {
        //Might add screen extend
        let (gamma, msaa, show_mode_int, far, near, auto_exp_setting) = {
            let es_lck = self.engine_settings
            .lock()
            .expect("failed to lock settings for frame creation");

            let gamma = es_lck.get_render_settings().get_gamma();
            let msaa = es_lck.get_render_settings().get_msaa_factor();
            let debug_int = es_lck.get_render_settings().get_debug_settings().debug_view.as_shader_int();
            let far_plane = es_lck.camera.far_plane.clone();
            let near_plane = es_lck.camera.near_plane.clone();
            let auto_exp_setting = {
                if es_lck.get_render_settings().get_exposure().use_auto_exposure{
                    0.0
                }else{
                    es_lck.get_render_settings().get_exposure().min_exposure
                }
            };
            (gamma, msaa, debug_int, far_plane, near_plane, auto_exp_setting)
        };


        let hdr_settings_data = default_pstprg_fragment::ty::hdr_settings{
              gamma: gamma,
              sampling_rate: msaa as i32,
              use_auto_exposure: auto_exp_setting,
              show_mode: show_mode_int,
              near: near,
              far: far,
        };


        //the settings for this pass
        self.hdr_settings_pool.next(hdr_settings_data).expect("failed to alloc HDR settings")
    }

    pub fn sort_hdr(&mut self,
        command_buffer: FrameStage,
        frame_system: &FrameSystem,
    ) -> FrameStage{
        //match the current stage, if wrong, panic
        match command_buffer{
            FrameStage::HdrSorting(cb) => {

                //create the descriptor set for the current image
                let attachments_ds = PersistentDescriptorSet::start(self.resolve_pipe.get_pipeline_ref(), 0) //at binding 0
                    .add_image(frame_system.object_pass_images.forward_hdr_image.clone())
                    .expect("failed to add hdr_image to sorting pass descriptor set")
                    .build()
                    .expect("failed to build postprogress cb");

                //the settings for this pass
                let settings = self.get_hdr_settings();

                let settings_buffer = PersistentDescriptorSet::start(self.resolve_pipe.get_pipeline_ref(), 1) //At binding 1
                    .add_buffer(settings)
                    .expect("failed to add hdr image settings buffer to post progress attachment")
                    .build()
                    .expect("failed to build settings attachment for postprogress pass");

                //perform the post progress
                let mut command_buffer = cb;
                command_buffer = command_buffer.draw(
                    self.resolve_pipe.get_pipeline_ref(),
                    frame_system.get_dynamic_state().clone(),
                    vec![self.screen_vertex_buffer.clone()],
                    (attachments_ds, settings_buffer),
                    ()
                ).expect("failed to add draw call for the sorting plane");

                return FrameStage::HdrSorting(command_buffer);
            },
            _ => {
                println!("Can't execute post_progress, wrong frame", );
            }
        }

        command_buffer
    }

    pub fn execute_blur(&mut self,
        command_buffer: FrameStage,
        frame_system: &FrameSystem,
    ) -> FrameStage{

        let blur_settings = {
            self.engine_settings.lock().expect("failed to get settings").get_render_settings().get_bloom()
        };

        //get the command buffer and decide which blur to apply
        let (unw_cb, is_horizontal) = match command_buffer{
            FrameStage::BlurH(cb) => (cb, true),
            FrameStage::BlurV(cb) => (cb, false),
            _ => {
                println!("We are in the wrong frame stage, no blur", );
                panic!("wrong frame stage");
            }
        };

        let blur_int = {
            if is_horizontal{
                1
            }else{
                0
            }
        };

        let blur_settings = blur::ty::blur_settings{
            horizontal: blur_int,
            scale: blur_settings.scale,
            strength: blur_settings.strength,
        };

        let blur_settings = self.blur_settings_pool.next(blur_settings).expect("failed to allocate blur settings");

        let attachments_ds = PersistentDescriptorSet::start(self.blur_pipe.get_pipeline_ref(), 0) //at binding 0
            .add_sampled_image(
                if is_horizontal{
                    frame_system.object_pass_images.hdr_fragments.clone()
                }else{
                    frame_system.post_pass_images.after_blur_h.clone()
                },
                self.screen_sampler.clone()
            )
            .expect("failed to add blur image")
            .add_buffer(blur_settings)
            .expect("failed to add blur settings")
            .build()
            .expect("failed to build postprogress cb");

        let command_buffer = unw_cb.draw(
            self.blur_pipe.get_pipeline_ref(),
            frame_system.get_dynamic_state().clone(),
            vec![self.screen_vertex_buffer.clone()],
            (attachments_ds),
            ()
        ).expect("failed to add draw call for the blur plane");

        //now return the right stage
        if is_horizontal{
            FrameStage::BlurH(command_buffer)
        }else{
            FrameStage::BlurV(command_buffer)
        }

    }

    ///Takes the hdr_image computes the average lumiosity and stores it in its buffer. The information is used
    /// in the assamble stage to set the exposure setting.
    pub fn compute_lumiosity(&mut self,
        command_buffer: FrameStage,
        frame_system: &FrameSystem,
    ) -> FrameStage{
        match command_buffer{
            FrameStage::ComputeAverageLumiosity(cb) => {

                //The walking source image, first one is the current ldr image.
                let mut local_source_image_attachemtn = frame_system.object_pass_images.ldr_fragments.clone();
                let mut local_source_image = frame_system.post_pass_images.scaled_ldr_images[0].clone();

                let mut local_cb = cb;

                //First of all we create all "mipmaps" of the currently rendered frame
                for (index, image) in frame_system.post_pass_images.scaled_ldr_images.iter().enumerate(){
                    //Get the extend of the source (the firstone comes from the attachment)
                    let sourc_dim = {
                        if index == 0{
                            ImageAccess::dimensions(&local_source_image_attachemtn)
                        }else{
                            ImageAccess::dimensions(&local_source_image)
                        }
                    };
                    let source_lower_right: [i32; 3] = {
                        match sourc_dim{
                            ImageDimensions::Dim2d{width, height, array_layers, cubemap_compatible} => [width as i32, height as i32, 1],
                            _ => {
                                println!("Faking image source", );
                                [1,1,1]
                            }
                        }
                    };

                    let target_dim = ImageAccess::dimensions(&image);
                    //Now same for the target
                    let target_lower_right: [i32; 3] = {
                        match target_dim{
                            ImageDimensions::Dim2d{width, height, array_layers, cubemap_compatible} => [width as i32, height as i32, 1],
                            _ => {
                                println!("Faking image destination", );
                                [1,1,1]
                            }
                        }
                    };

                    if index == 0{
                        //Now blit the current source to the target, first time its the attachment image
                        local_cb = local_cb.blit_image(
                            local_source_image_attachemtn.clone(),
                            [0; 3],
                            source_lower_right,
                            0,
                            0,
                            image.clone(),
                            [0; 3],
                            target_lower_right,
                            0,
                            0,
                            1,
                            Filter::Linear
                        ).expect("failed to blit ldr image to one pixel iamge");
                    }else{
                        //Now blit the current source to the target
                        local_cb = local_cb.blit_image(
                            local_source_image.clone(),
                            [0; 3],
                            source_lower_right,
                            0,
                            0,
                            image.clone(),
                            [0; 3],
                            target_lower_right,
                            0,
                            0,
                            1,
                            Filter::Linear
                        ).expect("failed to blit ldr image to one pixel iamge");
                    }

                    //Now setup the current target as the next source
                    local_source_image = image.clone();
                }



                //Since we blittet all images, we take the last one (assuming that it is 1x1)
                // and push it to the calculation on the gpu
                let one_pix_image = frame_system.post_pass_images.scaled_ldr_images.iter().last().expect("failed to get last average image").clone();


                let exposure_settings = {
                    self.engine_settings.lock().expect("failed to lock settings").get_render_settings().get_exposure()
                };

                let exposure_data = average_lumiosity_compute_shader::ty::ExposureSettings{
                    min_exposure: exposure_settings.min_exposure,
                    max_exposure: exposure_settings.max_exposure,
                    scale_up_speed: exposure_settings.scale_up_speed,
                    scale_down_speed: exposure_settings.scale_down_speed,
                    target_lumiosity: exposure_settings.target_lumiosity,
                    use_auto_exposure: if exposure_settings.use_auto_exposure {
                        0.0
                    }else{
                        self.engine_settings.lock().expect("failed to lock settings")
                        .get_render_settings().get_exposure().min_exposure
                    },
                };

                let exposure_settings_data = self.exposure_settings_pool.next(exposure_data)
                .expect("failed to allocate new exposure settings data");


                let des_set = self.average_set_pool.next()
                    .add_sampled_image(
                        one_pix_image.clone(),
                        self.screen_sampler.clone()
                    ).expect("failed to add sampled screen image")
                    .add_buffer(self.average_buffer.clone())
                    .expect("failed to add average buffer to compute descriptor")
                    .add_buffer(exposure_settings_data)
                    .expect("failed to add exposure settings to descriptor")
                .build().expect("failed to build average compute descriptor");

                //Start the compute operation
                //Only one thread...
                let new_command_buf = local_cb.dispatch([1, 1, 1], self.average_pipe.clone(), (des_set), ())
                .expect("failed to add compute operation for average lumiosity");

                return FrameStage::ComputeAverageLumiosity(new_command_buf);
            }
            _ => {
                println!("Wrong frame stage, returning the gotten one", );
            }
        }
        command_buffer
    }


    ///Executes the post progress on the recived command buffer and returns it, returns the buffer
    /// unchanged if it is in the wrong stage.
    pub fn assemble_image(&mut self,
        command_buffer: FrameStage,
        frame_system: &FrameSystem,
    ) -> FrameStage{
        //match the current stage, if wrong, panic
        match command_buffer{
            FrameStage::Postprogress(cb) => {

                //TO find the ldr image we analyse the debug settings
                let ldr_level_image = {
                    let debug_level = {
                        self.engine_settings.lock().expect("failed to lock settings")
                        .get_render_settings().get_debug_settings().ldr_debug_view_level
                    };
                    let level = {
                        if debug_level > (frame_system.post_pass_images.scaled_ldr_images.len() - 1) as u32{
                            (frame_system.post_pass_images.scaled_ldr_images.len() - 1) as u32
                        }else{
                            debug_level
                        }
                    };
                    //now we can savely return the n-th image
                    frame_system.post_pass_images.scaled_ldr_images[level as usize].clone()

                };



                //create the descriptor set for the current image
                let attachments_ds = PersistentDescriptorSet::start(self.pipeline.get_pipeline_ref(), 0) //at binding 0
                    .add_sampled_image(
                        frame_system.object_pass_images.ldr_fragments.clone(),
                        self.screen_sampler.clone()
                    )
                    .expect("failed to add hdr_image to postprogress descriptor set")
                    //needs to be a input attachment since we don't want to also downsample the depths
                    .add_image(frame_system.object_pass_images.forward_hdr_depth.clone())
                    .expect("failed to add depth map")
                    .add_sampled_image(
                        frame_system.post_pass_images.after_blur_v.clone(),
                        self.screen_sampler.clone()
                    ).expect("failed to add hdr fragments to assemble stage")
                    .add_sampled_image(
                        ldr_level_image,
                        self.screen_sampler.clone()
                    ).expect("failed to add average lumiosity texture to assemble stage")
                    .add_sampled_image(
                        frame_system.shadow_images.directional_shadows.clone(),
                        self.screen_sampler.clone()
                    ).expect("failed to add shadow texture to assemble stage")
                    .build()
                    .expect("failed to build postprogress cb");

                //the settings for this pass
                let settings = self.get_hdr_settings();

                let settings_buffer = PersistentDescriptorSet::start(self.pipeline.get_pipeline_ref(), 1) //At binding 1
                    .add_buffer(settings)
                    .expect("failed to add hdr image settings buffer to post progress attachment")
                    .add_buffer(self.average_buffer.clone())
                    .expect("failed to add lumiosity buffer to assemble pass")
                    .build()
                    .expect("failed to build settings attachment for postprogress pass");

                //perform the post progress
                let mut command_buffer = cb;
                command_buffer = command_buffer.draw(
                    self.pipeline.get_pipeline_ref(),
                    frame_system.get_dynamic_state().clone(),
                    vec![self.screen_vertex_buffer.clone()],
                    (attachments_ds, settings_buffer),
                    ()
                ).expect("failed to add draw call for the post progress plane");

                return FrameStage::Postprogress(command_buffer);
            },
            _ => {
                println!("Can't execute post_progress, wrong frame", );
            }
        }

        command_buffer
    }
}


///The compute shader used to compute the current average lumiosity of this image
pub mod average_lumiosity_compute_shader{
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "data/shader/average_luminance.comp"]
    struct Dummy;
}
