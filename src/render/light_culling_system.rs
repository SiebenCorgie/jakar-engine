use render::uniform_manager;
use render::frame_system;
use render::pipeline;
use jakar_tree;
use core::next_tree;

use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::buffer::*;
use vulkano::pipeline::ComputePipeline;
use vulkano;


use cgmath::*;

use std::sync::{Arc,Mutex};

/// This modules handles the efficient light culling. It used a clustered aproach which was described
/// here: http://www.cse.chalmers.se/~uffe/clustered_shading_preprint.pdf.
/// However, contrary to the solution used there we don't create a frustum per tile and cull against the
/// lights radius in worldspace. We transform the AABB of the light to screenspace and cull it against
/// the x-y location of the current thread as well as the min and max depth values. We decide which depth
/// values / clusters are overlaped and mark them with the light indice in the static "lights array".
/// The resulting buffer is the used in the normal forward pass to only use lights which intersect the
/// local cluster of the current pixel.
///
/// Note: the max light count per cluster is 512 point lights and 512 spot light (Directional light don't need culling).
/// So we have a static array of n intergers:
/// representing the light indices used per tile.
/// `[[[[1024] z-steps] y-tile-number] x-tile-number]` currently this means `[[[[i32; 1024]; 5]; 16]; 16]` for the
/// integer size of 4 byte this means around 5 mb of buffer we have to send to the compute shader,
/// additionaly to "all lights". Which might be a much bigger size. We also store the actual light
/// count per type in a struct called `cluster_t`. This way we actully send 16*16*8 of this structs which makes
/// two bytes more per cluster.

/// Within the light shader we then can get the lights something like that:
/// ```
///     //point light
///     for (int i=0; i<= lights[x_pos][y_pos][z_pos].point_light_count || MAX_POINT_LIGHTS){
///         do expensive light calc for lights[x_pos][y_pos][z_pos][i];
///     }
///     //or for spot lights with the offset:

///     //spot light
///     for (int i=0; i<= lights[x_pos][y_pos][z_pos].spot_light_count || MAX_SPOT_LIGHTS){
///         do expensive light calc for lights[x_pos][y_pos][z_pos][512+i];
///     }
/// ```


pub struct PreDpethSystem {
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    light_indice_buffer: Arc<CpuAccessibleBuffer< Vec<Vec<Vec<light_cull_shader::ty::Cluster>>> >>,

    empty_cluster: light_cull_shader::ty::Cluster,



}

impl PreDpethSystem{
    pub fn new(
        uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
    ) -> Self {

        let empty_cluster = light_cull_shader::ty::Cluster{
              point_light_count: 0,
              spot_light_count: 0,
              light_indices: [-1; 1024]
        };
        println!("Creating light indice buffer for the first time!", );
        //Since we are using too much data for the stack (around 5 mb) we have to heap allocate into a Vec<Vec<Vec<Cluster>>>
        let index_buffer = vec![vec![vec![empty_cluster.clone(); 8]; 16]; 16];
        println!("Finished...", );
        //We now create the buffer for the first binding from this data
        let index_buffer = CpuAccessibleBuffer::from_data(
            device.clone(), BufferUsage::all(), index_buffer)
            .expect("failed to create index buffer");


        PreDpethSystem{
            uniform_manager: uniform_manager,
            device: device,
            queue: queue,
            light_indice_buffer: index_buffer,
            empty_cluster: empty_cluster,
        }
    }

    pub fn dispatch_compute_shader(
        &mut self,
        command_buffer: frame_system::FrameStage,
    ) -> frame_system::FrameStage{
        match command_buffer{
            frame_system::FrameStage::LightCompute(cb) => {
                /*
                //We start by creating the sized index buffer for the first binding
                let empty_indexes = [[[self.empty_cluster; 8]; 16]; 16];
                //We now create the buffer for the first binding from this data
                let index_buffer = CpuAccessibleBuffer::from_data(
                    self.device.clone(), BufferUsage::all(), empty_indexes)
                    .expect("failed to create index buffer");

                //TODO Get all the light buffer from the uniform manager and store them to be used later
                // in the fragment shader within each object to guarantee that the indices are right.

                let shader = light_cull_shader::Shader::load(self.device.clone())
                    .expect("failed to create shader module");

                let compute_pipeline = Arc::new(
                    ComputePipeline::new(self.device.clone(), &shader.main_entry_point(), &()
                )
                .expect("failed to create compute pipeline"));

                //adds the light buffers (all lights and indice buffer)
                let set_01 = Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
                    .add_buffer(index_buffer).expect("failed to add index buffer")
                    .add_buffer(light_count).expect("failed to add light count data")
                    .add_buffer(point_lights).expect("failed to add point_lights")
                    .add_buffer(spot_lights).expect("failed to add spot_lights")
                    .build().unwrap()
                );

                let set_02 = {
                    let mut uniform_manager_lck = self.uniform_manager.lock()
                    .expect("Failed to lock unfiorm_mng");
                    Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 1)
                        .add_buffer(uniform_manager_lck.get_subbuffer_data(Matrix4::identity())).unwrap()
                        .build().unwrap()
                    )
                };

                //Now add to cb
                let new_cb = cb.dispatch([16, 16, 8], compute_pipeline.clone(), (set_01.clone(), set_02.clone()), ()).expect("failed to add compute operation");
                */
                //END
                return frame_system::FrameStage::LightCompute(cb);

            }
            _ => {
                println!("Got wrong framestage for dispatching compute shader, not going to do it ...", );
                return command_buffer;
            }
        }
    }

    ///Creates a descriptor set from a node
    fn get_descriptor(
        &self,
        compute_pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,
        transform_matrix: Matrix4<f32>
    ) -> Arc<DescriptorSet + Send + Sync>{
        let mut uniform_manager_lck = self.uniform_manager.lock().expect("Failed to lock unfiorm_mng");
        let new_set = Arc::new(PersistentDescriptorSet::start(
                compute_pipeline.clone(), 0
            )
            .add_buffer(uniform_manager_lck.get_subbuffer_data(transform_matrix)).expect("Failed to create descriptor set")
            .build().expect("failed to build descriptor \"Depth\"")
        );
        new_set
    }
    /*
    pub fn get_index_buffer(&self) -> Arc<CpuAccessibleBuffer<[i32]>>{
        self.light_indice_buffer.clone()
    }
    */
}

mod light_cull_shader {
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "data/shader/light_culling.comp"]
    struct Dummy;
}
