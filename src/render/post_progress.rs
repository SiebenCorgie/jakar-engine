
use render::shader_impls::default_pstprg_fragment;
use render::pipeline;
use render::frame_system::FrameStage;

use core::engine_settings;

use vulkano;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::image::traits::ImageViewAccess;
use vulkano::image::traits::ImageAccess;
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
    screen_vertex_buffer: Arc<vulkano::buffer::BufferAccess + Send + Sync>,
    settings_pool: vulkano::buffer::cpu_pool::CpuBufferPool<default_pstprg_fragment::ty::hdr_settings>
}


impl PostProgress{
    ///Create the postprogressing chain
    pub fn new(
        engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
        post_progress_pipeline: Arc<pipeline::Pipeline>,
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
            screen_vertex_buffer: sample_vertex_buffer,
            settings_pool: settings_pool,
        }
    }

    ///Executes the post progress on the recived command buffer and returns it, returns the buffer
    /// unchanged if it is in the wrong stage.
    pub fn execute(&self,
        command_buffer: FrameStage,
        hdr_image: Arc<ImageViewAccess  + Send + Sync>,
        depth_buffer: Arc<ImageViewAccess  + Send + Sync>,
    ) -> FrameStage{
        //match the current stage, if wrong, panic
        match command_buffer{
            FrameStage::Postprogress(cb) => {
                //debug
                self.pipeline.print_shader_name();
                //create the descriptor set for the current image
                let attachments_ds = PersistentDescriptorSet::start(self.pipeline.get_pipeline_ref(), 0) //at binding 0
                    .add_image(hdr_image)
                    .expect("failed to add hdr_image to postprogress descriptor set")
                    .add_image(depth_buffer)
                    .expect("failed to add depth image")
                    .build()
                    .expect("failed to build postprogress cb");

                let (exposure, gamma, msaa, dimensions) = {


                    let mut es_lck = self.engine_settings
                    .lock()
                    .expect("failed to lock settings for frame creation");

                    let exposure = es_lck.get_render_settings().get_exposure();
                    let gamma = es_lck.get_render_settings().get_gamma();
                    let msaa = es_lck.get_render_settings().get_msaa_factor();
                    let dimensions =  es_lck.get_dimensions();
                    (exposure, gamma, msaa, dimensions)
                };

                let hdr_settings_data = default_pstprg_fragment::ty::hdr_settings{
                      exposure: exposure,
                      gamma: gamma,
                      sampling_rate: msaa as i32,
                };

                //the settings for this pass
                let settings = match self.settings_pool.next(hdr_settings_data){
                    Ok(set) => set,
                    Err(_) => {
                        println!("Failed to allocate subbuffer for hdr pass", );
                        return FrameStage::Postprogress(cb);
                    }
                };

                let settings_buffer = PersistentDescriptorSet::start(self.pipeline.get_pipeline_ref(), 1) //At binding 1
                    .add_buffer(settings)
                    .expect("failed to add hdr image settings buffer to post progress attachment")
                    .build()
                    .expect("failed to build settings attachment for postprogress pass");





                println!("Adding post progress cb", );
                //perform the post progress
                let mut command_buffer = cb;
                command_buffer = command_buffer.draw(
                    self.pipeline.get_pipeline_ref(),
                    vulkano::command_buffer::DynamicState{
                        line_width: None,
                        viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                            origin: [0.0, 0.0],
                            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                            depth_range: 0.0 .. 1.0,
                        }]),
                        scissors: None,
                    },
                    vec![self.screen_vertex_buffer.clone()],
                    (attachments_ds, settings_buffer),
                    ()
                ).expect("failed to add draw call for the post progress plane");
                println!("Returning post progress cb", );
                return FrameStage::Postprogress(command_buffer);
            },
            _ => {
                println!("Can't execute post_progress, wrong frame", );
            }
        }

        command_buffer
    }
}
