use vulkano;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::sampler::Sampler;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::pipeline::GraphicsPipelineAbstract;

use core::engine_settings::EngineSettings;
use render::frame_system::FrameSystem;
use render::shader::shaders::blur;
use render::pipeline;

use std::sync::{Arc,Mutex};

///Contains all components needed to generate the the bloom images
pub struct Bloom {
    engine_settings: Arc<Mutex<EngineSettings>>,

    blur_settings_pool: CpuBufferPool<blur::ty::blur_settings>,
    blur_set_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,
    blur_pipe: Arc<pipeline::Pipeline>,

    screen_sampler: Arc<Sampler>,

}

impl Bloom{

    pub fn new(
        engine_settings: Arc<Mutex<EngineSettings>>,
        device: Arc<vulkano::device::Device>,
        blur_pipe: Arc<pipeline::Pipeline>,
        screen_sampler: Arc<Sampler>

    ) -> Self{

        let blur_pool = CpuBufferPool::<blur::ty::blur_settings>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );
        let blur_pool = CpuBufferPool::<blur::ty::blur_settings>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );

        let blur_set_pool = FixedSizeDescriptorSetsPool::new(blur_pipe.get_pipeline_ref(), 0);

        Bloom{
            engine_settings,
            blur_settings_pool: blur_pool,
            blur_set_pool,
            blur_pipe,
            screen_sampler,
        }
    }

    pub fn execute_blur(&mut self,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
        screen_vertex_buffer: Arc<vulkano::buffer::BufferAccess + Send + Sync>
    ) -> AutoCommandBufferBuilder{
        //Change to blur_h pass
        let blur_h_fb = frame_system.get_passes().blur_pass.get_fb_blur_h();
        let clearings_h = vec![
            [0.0, 0.0, 0.0, 0.0].into()
        ];
        let mut next_cb = command_buffer.begin_render_pass(
            blur_h_fb, false, clearings_h
        ).expect("failed to start blur_h pass");
        //now blur
        next_cb = self.blur(true, next_cb, frame_system, screen_vertex_buffer.clone());


        //now end this pass and change to the blur_v pass
        next_cb = next_cb.end_render_pass().expect("failed to end blur_h pass");

        let blur_v_fb = frame_system.get_passes().blur_pass.get_fb_blur_v();
        let clearings_v = vec![
            [0.0, 0.0, 0.0, 0.0].into()
        ];
        next_cb = next_cb.begin_render_pass(
            blur_v_fb, false, clearings_v
        ).expect("failed to start blur_v pass");
        //now blur
        next_cb = self.blur(false, next_cb, frame_system, screen_vertex_buffer);
        //finally change into neutral mode again and return
        next_cb = next_cb.end_render_pass().expect("failed to end blur_v renderpass");

        next_cb
    }

    ///Adds a command to blur an image either horizontal or vertical
    fn blur(
        &mut self,
        is_horizontal: bool,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
        screen_vertex_buffer: Arc<vulkano::buffer::BufferAccess + Send + Sync>
    )-> AutoCommandBufferBuilder{
        let blur_settings = {
            self.engine_settings.lock().expect("failed to get settings").get_render_settings().get_bloom()
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

        let attachments_ds = self.blur_set_pool.next() //at binding 0
            .add_sampled_image(
                if is_horizontal{
                    frame_system.get_passes().object_pass.get_images().hdr_fragments.clone()
                }else{
                    frame_system.get_passes().blur_pass.get_images().after_blur_h.clone()
                },
                self.screen_sampler.clone()
            )
            .expect("failed to add blur image")
            .add_buffer(blur_settings)
            .expect("failed to add blur settings")
            .build()
            .expect("failed to build postprogress cb");

        let new_command_buffer = command_buffer.draw(
            self.blur_pipe.get_pipeline_ref(),
            frame_system.get_dynamic_state().clone(),
            vec![screen_vertex_buffer],
            attachments_ds,
            ()
        ).expect("failed to add draw call for the blur plane");

        new_command_buffer
    }
}
