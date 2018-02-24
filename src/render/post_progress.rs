
use render::shader::shaders::default_pstprg_fragment;
use render::pipeline;
use render::frame_system::FrameStage;
use render::frame_system::FrameSystem;

use core::engine_settings;

use vulkano;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::buffer::cpu_pool::CpuBufferPool;

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
    pipeline: Arc<pipeline::Pipeline>,
    resolve_pipe: Arc<pipeline::Pipeline>,
    screen_vertex_buffer: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    settings_pool: vulkano::buffer::cpu_pool::CpuBufferPool<default_pstprg_fragment::ty::hdr_settings>,
}


impl PostProgress{
    ///Create the postprogressing chain
    pub fn new(
        engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
        post_progress_pipeline: Arc<pipeline::Pipeline>,
        resolve_pipe: Arc<pipeline::Pipeline>,
        device: Arc<vulkano::device::Device>
    ) -> Self{
        //generate a vertex buffer
        let mut vertices: Vec<PostProgressVertex> = Vec::new();
        //the screen space vertexes
        vertices.push(PostProgressVertex::new([-1.0; 2], [-1.0; 2]));
        vertices.push(PostProgressVertex::new([-1.0, 1.0], [-1.0, 1.0]));
        vertices.push(PostProgressVertex::new([1.0; 2], [1.0; 2]));

        vertices.push(PostProgressVertex::new([-1.0; 2], [-1.0; 2]));
        vertices.push(PostProgressVertex::new([1.0, -1.0], [1.0, -1.0]));
        vertices.push(PostProgressVertex::new([1.0; 2], [1.0; 2]));

        let sample_vertex_buffer = vulkano::buffer::cpu_access::CpuAccessibleBuffer
                                    ::from_iter(device.clone(), vulkano::buffer::BufferUsage::all(), vertices.iter().cloned())
                                    .expect("failed to create buffer");

        //we also have to maintain a buffer pool for the settings which can potentually change
        let settings_pool = CpuBufferPool::<default_pstprg_fragment::ty::hdr_settings>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );


        PostProgress{
            engine_settings: engine_settings,
            pipeline: post_progress_pipeline,
            resolve_pipe: resolve_pipe,
            screen_vertex_buffer: sample_vertex_buffer,
            settings_pool: settings_pool,
        }
    }

    fn get_hdr_settings(&mut self) -> vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer
    <default_pstprg_fragment::ty::hdr_settings, Arc<vulkano::memory::pool::StdMemoryPool>> {
        //Might add screen extend
        let (exposure, gamma, msaa, show_mode_int, far, near) = {
            let mut es_lck = self.engine_settings
            .lock()
            .expect("failed to lock settings for frame creation");

            let exposure = es_lck.get_render_settings().get_exposure();
            let gamma = es_lck.get_render_settings().get_gamma();
            let msaa = es_lck.get_render_settings().get_msaa_factor();
            let debug_int = es_lck.get_render_settings().get_debug_view().as_shader_int();
            let far_plane = es_lck.camera.far_plane.clone();
            let near_plane = es_lck.camera.near_plane.clone();
            (exposure, gamma, msaa, debug_int, far_plane, near_plane)
        };

        let hdr_settings_data = default_pstprg_fragment::ty::hdr_settings{
              exposure: exposure,
              gamma: gamma,
              sampling_rate: msaa as i32,
              show_mode: show_mode_int,
              near: near,
              far: far,
        };


        //the settings for this pass
        self.settings_pool.next(hdr_settings_data).expect("failed to alloc HDR settings")
    }

    pub fn sort_hdr(&mut self,
        command_buffer: FrameStage,
        frame_system: &FrameSystem,
    ) -> FrameStage{
        //match the current stage, if wrong, panic
        match command_buffer{
            FrameStage::HdrSorting(cb) => {
                //debug
                self.pipeline.print_shader_name();
                //create the descriptor set for the current image
                let attachments_ds = PersistentDescriptorSet::start(self.resolve_pipe.get_pipeline_ref(), 0) //at binding 0
                    .add_image(frame_system.get_forward_hdr_image())
                    .expect("failed to add hdr_image to postprogress descriptor set")
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

    ///Executes the post progress on the recived command buffer and returns it, returns the buffer
    /// unchanged if it is in the wrong stage.
    pub fn execute(&mut self,
        command_buffer: FrameStage,
        frame_system: &FrameSystem,
    ) -> FrameStage{
        //match the current stage, if wrong, panic
        match command_buffer{
            FrameStage::Postprogress(cb) => {
                //debug
                self.pipeline.print_shader_name();
                //create the descriptor set for the current image
                let attachments_ds = PersistentDescriptorSet::start(self.pipeline.get_pipeline_ref(), 0) //at binding 0
                    .add_image(frame_system.get_forward_hdr_image())
                    .expect("failed to add hdr_image to postprogress descriptor set")
                    .add_image(frame_system.get_forward_hdr_depth())
                    .expect("failed to add depth image")

                    .build()
                    .expect("failed to build postprogress cb");

                //the settings for this pass
                let settings = self.get_hdr_settings();

                let settings_buffer = PersistentDescriptorSet::start(self.pipeline.get_pipeline_ref(), 1) //At binding 1
                    .add_buffer(settings)
                    .expect("failed to add hdr image settings buffer to post progress attachment")
                    //.add_sampled_image(frame_system.get_hdr_fragments(), self.default_sampler.clone())
                    //.expect("failed to add sampled attachment")
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
