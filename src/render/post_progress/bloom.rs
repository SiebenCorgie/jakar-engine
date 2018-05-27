use vulkano;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::sampler::Sampler;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::image::ImageDimensions;
use vulkano::image::Dimensions;
use vulkano::image::traits::{ImageAccess, ImageViewAccess};
use vulkano::sampler::Filter;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::framebuffer::FramebufferAbstract;

use core::engine_settings::EngineSettings;
use render::frame_system::FrameSystem;
use render::pipeline;
use render::pipeline_manager;
use render::pipeline_builder;
use render::render_passes::RenderPassConf;
use render::shader::shaders::blur;
use render::render_passes::post_images::BlurStage;

use std::sync::{Arc,Mutex};

///Contains all components needed to generate the the bloom images
pub struct Bloom {
    engine_settings: Arc<Mutex<EngineSettings>>,

    blur_settings_pool: CpuBufferPool<blur::ty::blur_settings>,
    blur_descset_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,
    blur_pipe: Arc<pipeline::Pipeline>,
}

impl Bloom{

    pub fn new(
        engine_settings: Arc<Mutex<EngineSettings>>,
        device: Arc<vulkano::device::Device>,
        pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,
    ) -> Self{

        let blur_pipe = pipeline_manager.lock()
        .expect("failed to lock new pipeline manager")
        .get_pipeline_by_config(
            pipeline_builder::PipelineConfig::default()
                .with_shader("PpBlur".to_string())
                .with_render_pass(RenderPassConf::BlurPass)
                .with_depth_and_stencil_settings(
                    pipeline_builder::DepthStencilConfig::NoDepthNoStencil
                ),
        );

        let blur_descset_pool = FixedSizeDescriptorSetsPool::new(blur_pipe.get_pipeline_ref().clone(), 0);
        let blur_settings_pool = CpuBufferPool::uniform_buffer(device.clone());

        Bloom{
            engine_settings,
            blur_settings_pool,
            blur_descset_pool,
            blur_pipe,
        }
    }

    pub fn execute_blur(&mut self,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
        sampler: Arc<Sampler>,
        vertex_buf: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    ) -> AutoCommandBufferBuilder{
        //Resize unitl we hit the set limit
        let mut new_command_buffer = self.resize(
            command_buffer,
            frame_system,
        );
        //Now blur in reversed order and add image up unil we hit the last image
        new_command_buffer = self.blur(
            new_command_buffer,
            frame_system,
            sampler,
            vertex_buf
        );

        new_command_buffer
    }

    fn resize(&self,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
    ) -> AutoCommandBufferBuilder{
        //Our first image is the hdr fragments image
        let mut source: Arc<ImageAccess + Send + Sync + 'static> = frame_system
        .get_passes().object_pass.get_images().hdr_fragments.clone();
        //first target is the first image in the row of blur target
        let mut target: Arc<ImageAccess + Send + Sync + 'static> = frame_system
        .get_passes().blur_pass.get_images().bloom[0].input_image.clone();

        let num_blur_levels = frame_system
        .get_passes().blur_pass.get_images().bloom.len();

        //Resize the hdr frags to the first level
        let mut new_cb = self.resize_to(
            command_buffer,
            frame_system,
            source,
            target
        );


        for idx in 1..num_blur_levels{
            //Set new source and target
            source = frame_system
            .get_passes().blur_pass.get_images().bloom[idx - 1].input_image.clone();
            //first target is the first image in the row of blur target
            target = frame_system
            .get_passes().blur_pass.get_images().bloom[idx].input_image.clone();

            new_cb = self.resize_to(
                new_cb,
                frame_system,
                source,
                target
            );
        }


        new_cb
    }
    ///Helper function which takes two image acces images and resizes `source` to `target`
    fn resize_to(
        &self,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
        source: Arc<ImageAccess + Send + Sync + 'static>,
        target: Arc<ImageAccess + Send + Sync + 'static>
    ) -> AutoCommandBufferBuilder{

        let source_dim = ImageAccess::dimensions(&source);
        //Now same for the target
        let source_lower_right: [i32; 3] = {
            match source_dim{
                ImageDimensions::Dim2d{width, height, array_layers, cubemap_compatible} => [width as i32, height as i32, 1],
                _ => {
                    println!("Faking image source dim", );
                    [1,1,1]
                }
            }
        };

        let target_dim = ImageAccess::dimensions(&target);
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

        //Currently only resizeing single level, the hdr fragments to the blur image
        let local_cb = command_buffer.blit_image(
            source,
            [0; 3],
            source_lower_right,
            0,
            0,
            target,
            [0; 3],
            target_lower_right,
            0,
            0,
            1,
            Filter::Linear
        ).expect("failed to blit image");
        println!("Resizerd from {:?} to {:?}", source_lower_right, target_lower_right);

        local_cb
    }


    fn blur(
        &mut self,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
        sampler: Arc<Sampler>,
        vertex_buf: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    ) -> AutoCommandBufferBuilder{

        let mut local_cb = command_buffer;

        //Blur each level, this time we start with the last and add them up until we reached the last
        //image
        let num_blur_levels = frame_system
        .get_passes().blur_pass.get_images().bloom.len();
        let mut is_first_img = true;
        for idx in (0..num_blur_levels).rev(){

            let target_stack = frame_system.get_passes()
            .blur_pass.get_images().bloom[idx].clone();
            //We only want to add a previouse stack if we are not the lowest image
            let mut previouse_stack = None;
            if !is_first_img{
                previouse_stack = Some(
                    frame_system.get_passes()
                    .blur_pass.get_images().bloom[idx + 1].clone()
                );
            }else{
                is_first_img = false;
            }

            let to_h_fb = frame_system.get_passes().blur_pass.get_fb_blur_h(idx);
            let to_final = frame_system.get_passes().blur_pass.get_fb_to_final(idx);

            local_cb = self.blur_img(
                local_cb,
                frame_system,
                sampler.clone(),
                vertex_buf.clone(),
                target_stack,
                previouse_stack,
                to_h_fb,
                to_final
            );

        }


        local_cb
    }

    ///Adds a command to blur an image with an optional image to be added on top.
    fn blur_img(
        &mut self,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
        sampler: Arc<Sampler>,
        vertex_buf: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
        target_stack: BlurStage,
        previouse_stack: Option<BlurStage>,
        to_h_framebuffer: Arc<FramebufferAbstract + Send + Sync>,
        to_final_framebuffer: Arc<FramebufferAbstract + Send + Sync>,
    )-> AutoCommandBufferBuilder{

        //First we blur from the inital resized image to the after_h_img
        let after_h_cb = {
            let blur_img = target_stack.input_image.clone();

            let settings = blur::ty::blur_settings{
                is_horizontal: 1, // yes
                add_second: 0, // no
            };

            let buffer = self.blur_settings_pool.next(settings).expect("failed to get blur settings");

            let new_desc = self.blur_descset_pool.next()
            .add_sampled_image(blur_img.clone(), sampler.clone())
            .expect("failed to add first image to blur shader")
            .add_sampled_image(blur_img.clone(), sampler.clone()) //Same since we don't want to add
            .expect("failed to add sampled source image")
            .add_buffer(buffer)
            .expect("Failed to add settings buffer")

            .build()
            .expect("failed to build blur compute descriptor");

            //now get the frame buffer and change into the blur pass
            let clearings = vec![
                [0.0, 0.0, 0.0, 0.0].into()
            ];
            let mut new_cb = command_buffer.begin_render_pass(to_h_framebuffer, false, clearings)
            .expect("failed to start blur pass");

            //Now execute
            new_cb = new_cb.draw(
                self.blur_pipe.get_pipeline_ref(),
                frame_system.get_dynamic_state().clone(),
                vec![vertex_buf.clone()],
                new_desc,
                ()
            ).expect("failed to add draw call for the post progress plane");
            //noice, lets end the pass
            new_cb = new_cb.end_render_pass().expect("failed to end blur pass");
            new_cb
        };


        let final_cb = {
            //Now we take the after_h image and blur it on v to the final one
            //TODO make nice and abstract into an function
            let blur_v_img = target_stack.after_h_img.clone();
            //Test if we should add a previouse image
            let mut add_image = 0;
            if let Some(_) = previouse_stack{
                println!("Adding image!", );
                add_image = 1;
            }

            let settings_ah = blur::ty::blur_settings{
                is_horizontal: 0, // yes
                add_second: add_image, // no
            };

            let buffer_ah = self.blur_settings_pool.next(settings_ah).expect("failed to get blur settings");
            //Set the second iamge, same if we don't want to add, else the other
            let second_image = {
                    if let Some(other_stack) = previouse_stack{
                        other_stack.after_h_img.clone()
                    }else{
                        blur_v_img.clone()
                    }
            };


            let new_desc_ah = self.blur_descset_pool.next()
            .add_sampled_image(blur_v_img.clone(), sampler.clone())
            .expect("failed to add first image to blur shader")
            .add_sampled_image(second_image, sampler.clone()) //Same since we don't want to add
            .expect("failed to add sampled source image")
            .add_buffer(buffer_ah)
            .expect("Failed to add settings buffer")

            .build()
            .expect("failed to build blur compute descriptor");

            //now get the frame buffer and change into the blur pass
            let clearings_ah = vec![
                [0.0, 0.0, 0.0, 0.0].into()
            ];
            let mut new_cb = after_h_cb.begin_render_pass(to_final_framebuffer, false, clearings_ah)
            .expect("failed to start blur pass");

            //Now execute
            new_cb = new_cb.draw(
                self.blur_pipe.get_pipeline_ref(),
                frame_system.get_dynamic_state().clone(),
                vec![vertex_buf.clone()],
                new_desc_ah,
                ()
            ).expect("failed to add draw call for the post progress plane");
            //noice, lets end the pass
            new_cb = new_cb.end_render_pass().expect("failed to end blur pass");
            new_cb
        };

        final_cb

    }
}
