use render::uniform_manager;
use render::frame_system;
use render::pipeline;
use jakar_tree;
use core::next_tree;

use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::buffer::*;
use vulkano::pipeline::ComputePipeline;
use render::shader_impls::lights;
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::buffer::device_local::DeviceLocalBuffer;
use vulkano;

use cgmath::*;

use std::sync::{Arc,Mutex};

///TODO Description how we (I) do this

pub struct PreDpethSystem {
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,

    //Gets allocated ones and is used to attach the current cluster data to other shaders
    cluster_buffer: Arc<DeviceLocalBuffer<light_cull_shader::ty::ClusterBuffer>>,

    //is the buffer of currently used point, directional and spotlights used
    current_point_light_list: Arc<CpuAccessibleBuffer<[lights::ty::PointLight]>>,
    current_dir_light_list: Arc<CpuAccessibleBuffer<[lights::ty::DirectionalLight]>>,
    current_spot_light_list: Arc<CpuAccessibleBuffer<[lights::ty::SpotLight]>>,
    current_light_count: CpuBufferPoolSubbuffer<lights::ty::LightCount, Arc<vulkano::memory::pool::StdMemoryPool>>,

    compute_shader: Arc<light_cull_shader::Shader>,
}

impl PreDpethSystem{
    pub fn new(
        uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
    ) -> Self {

        //Now we pre_create the first current buffers and store them, they will be updated each time
        //a compute shader for a new frame is dispatched
        let (c_point_light, c_dir_light, c_spot_lights, c_light_count) = {
            let mut uniform_lck = uniform_manager.lock().expect("Failed to lock uniformanager for light creation");
            let p_l = uniform_lck.get_subbuffer_point_lights();
            let s_l = uniform_lck.get_subbuffer_spot_lights();
            let d_l = uniform_lck.get_subbuffer_directional_lights();
            let l_c = uniform_lck.get_subbuffer_light_count();

            (p_l, d_l, s_l, l_c)
        };

        //pre load the shader
        let shader = Arc::new(light_cull_shader::Shader::load(device.clone())
            .expect("failed to create shader module"));

        //Now we create the buffer, it wont be deleted until the system gets shut down.
        let persistent_cluster_buffer = DeviceLocalBuffer::new(
            device.clone(), BufferUsage::all(), vec![queue.family()].into_iter()
        ).expect("failed to create cluster buffer!");


        PreDpethSystem{
            uniform_manager: uniform_manager,
            device: device,
            queue: queue,

            cluster_buffer: persistent_cluster_buffer,

            current_point_light_list: c_point_light,
            current_dir_light_list: c_dir_light,
            current_spot_light_list: c_spot_lights,
            current_light_count: c_light_count,

            compute_shader: shader,
        }
    }

    ///Pulls in a new set of the light lists. This needs only to be called when the lightount in the
    /// Currently rendered level changes.
    pub fn update_light_set(&mut self){
        //Now we pre_create the first current buffers and store them, they will be updated each time
        //a compute shader for a new frame is dispatched
        let mut uniform_lck = self.uniform_manager.lock().expect("Failed to lock uniformanager for light creation");
        self.current_point_light_list = uniform_lck.get_subbuffer_point_lights();
        self.current_spot_light_list = uniform_lck.get_subbuffer_spot_lights();
        self.current_dir_light_list = uniform_lck.get_subbuffer_directional_lights();
        self.current_light_count = uniform_lck.get_subbuffer_light_count();
    }


    pub fn dispatch_compute_shader(
        &mut self,
        command_buffer: frame_system::FrameStage,
    ) -> frame_system::FrameStage{

        match command_buffer{
            frame_system::FrameStage::LightCompute(cb) => {

                let shader = self.compute_shader.clone();

                let compute_pipeline = Arc::new(
                    ComputePipeline::new(self.device.clone(), &shader.main_entry_point(), &()
                )
                .expect("failed to create compute pipeline"));

                //Get the current camera data
                let camera_data = {
                    let mut uniform_manager_lck = self.uniform_manager
                    .lock().expect("Failed to lock unfiorm_mng");

                    uniform_manager_lck.get_subbuffer_data(Matrix4::identity())
                };

                //adds the light buffers (all lights and indice buffer)
                let set_01 = Arc::new(PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
                    .add_buffer(self.cluster_buffer.clone())
                    .expect("failed to add index buffer")
                    //lights and counter
                    .add_buffer(self.current_point_light_list.clone())
                    .expect("Failed to create descriptor set")

                    .add_buffer(self.current_dir_light_list.clone())
                    .expect("Failed to create descriptor set")

                    .add_buffer(self.current_spot_light_list.clone())
                    .expect("Failed to create descriptor set")

                    .add_buffer(self.current_light_count.clone())
                    .expect("Failed to create descriptor set")

                    .build().expect("failed to build compute desc set 1")
                );


                //Now add to cb the dispatch
                let new_cb = cb.dispatch([32, 16, 32], compute_pipeline.clone(), set_01, ()).expect("failed to add compute operation");


                //println!("Dispatched compute buffer", );
                //END
                return frame_system::FrameStage::LightCompute(new_cb);

            }
            _ => {
                println!("Got wrong framestage for dispatching compute shader, not going to do it ...", );
                return command_buffer;
            }
        }
    }

    ///Returns only the cluster buffer
    pub fn get_cluster_buffer(&self) -> Arc<DeviceLocalBuffer<light_cull_shader::ty::ClusterBuffer>>{
        self.cluster_buffer.clone()
    }

    ///Since all the objects drawn in the current frame need to get the same light info, we create
    /// one decriptorset based on the needed set id when asked for it.
    ///TODO: Have a look if we can put this in a ring buffer (cpubufferpool)
    ///NOTE:
    /// - Binding 0 = point lights
    /// - Binding 1 = directional lights
    /// - Binding 2 = spot lights
    /// - Binding 3 = struct which describes how many actual lights where send
    pub fn get_light_descriptorset(
        &mut self, binding_id: u32,
        pipeline: Arc<vulkano::pipeline::GraphicsPipelineAbstract + Send + Sync>
    ) -> Arc<DescriptorSet + Send + Sync>{
        let new_set = Arc::new(PersistentDescriptorSet::start(
                pipeline.clone(), binding_id as usize
            )
            //now we copy the current buffers to the descriptor set
            .add_buffer(self.cluster_buffer.clone())
            .expect("failed to add cluster_buffer")
            .add_buffer(self.current_point_light_list.clone())
            .expect("Failed to create descriptor set")
            .add_buffer(self.current_dir_light_list.clone())
            .expect("Failed to create descriptor set")
            .add_buffer(self.current_spot_light_list.clone())
            .expect("Failed to create descriptor set")
            .add_buffer(self.current_light_count.clone())
            .expect("Failed to create descriptor set")
            .build().expect("failed to build descriptor 04")
        );

        new_set
    }
}

//The compute shader
pub mod light_cull_shader {
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "data/shader/light_culling.comp"]
    struct Dummy;
}
