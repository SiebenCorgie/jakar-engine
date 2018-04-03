use render::uniform_manager;
use render::frame_system;
use render::shadow_system::ShadowSystem;
use core::resource_management::asset_manager::AssetManager;
use core::next_tree::{SceneComparer, SaveUnwrap, SceneTree};
use core::resources::camera::Camera;

use render::shader::shader_inputs::lights::ty::LightCount;

use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::sampler::*;
use vulkano::buffer::*;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::ComputePipelineAbstract;
use render::shader::shader_inputs::lights;
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::buffer::device_local::DeviceLocalBuffer;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::buffer::immutable::ImmutableBuffer;
use vulkano::buffer::BufferUsage;
use vulkano;

use std::sync::{Arc,Mutex};

///The system is responsible for everything that has to do with actual light (no shadows). Itn will
/// dispatch a compute shader which builds a 3d matrix in worldspace which holds the following information
/// at each entry:
/// - how many point lights
/// - the indices of these point lights in the point_ligh_list
/// - how many spot lights
/// - the indices of these spot lights in the point_ligh_list
///
/// This information is used in the forward pass to determin which lights needs to be considered when shading.
/// because of this optimization it is possible to use around 1000 spot or point lights while still maintaining
/// over 30fps on a mid range gpu.
pub struct LightSystem {
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,

    //Gets allocated ones and is used to attach the current cluster data to other shaders
    cluster_buffer: Arc<DeviceLocalBuffer<light_cull_shader::ty::ClusterBuffer>>,

    //Descriptor pool to build the descriptorset faster
    descriptor_pool: FixedSizeDescriptorSetsPool<Arc<ComputePipelineAbstract + Send + Sync>>,

    //is the buffer of currently used point, directional and spotlights used
    current_point_light_list: Arc<ImmutableBuffer<[lights::ty::PointLight]>>,
    current_dir_light_list: Arc<ImmutableBuffer<[lights::ty::DirectionalLight]>>,
    current_spot_light_list: Arc<ImmutableBuffer<[lights::ty::SpotLight]>>,
    current_light_count: CpuBufferPoolSubbuffer<lights::ty::LightCount, Arc<vulkano::memory::pool::StdMemoryPool>>,
    //Pool to create the light count buffer.
    buffer_pool_05_count: vulkano::buffer::cpu_pool::CpuBufferPool<lights::ty::LightCount>,

    compute_pipeline: Arc<vulkano::pipeline::ComputePipelineAbstract + Send + Sync>,

    shadow_map_sampler: Arc<Sampler>,
}

impl LightSystem{
    pub fn new(
        uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
    ) -> Self {

        //Now we pre_create the first current buffers and store them, they will be updated each time
        //a compute shader for a new frame is dispatched
        let (c_point_light, c_dir_light, c_spot_lights) = {
            //Now build the buffers from the shader_infos and update them internaly
            let p_vec = Vec::new();
            let s_vec = Vec::new();
            let d_vec = Vec::new();

            let p_l = {
                let (buffer, future) = ImmutableBuffer::from_iter(
                    p_vec.clone().into_iter(),
                    BufferUsage::all(),
                    queue.clone()
                ).expect("Failed to create point light buffer");
                //Now drop the future (which will execute and then return)
                buffer
            };

            let s_l = {
                let (buffer, future) = ImmutableBuffer::from_iter(
                    s_vec.clone().into_iter(),
                    BufferUsage::all(),
                    queue.clone()
                ).expect("Failed to create spot light buffer");
                //Now drop the future (which will execute and then return)
                buffer
            };
            let d_l = {
                let (buffer, future) = ImmutableBuffer::from_iter(
                    d_vec.clone().into_iter(),
                    BufferUsage::all(),
                    queue.clone()
                ).expect("Failed to create directional light buffer");
                //Now drop the future (which will execute and then return)
                buffer
            };

            (p_l, d_l, s_l)
        };

        //pre load the shader
        let shader = Arc::new(light_cull_shader::Shader::load(device.clone())
            .expect("failed to create shader module"));

        //Now we create the buffer, it wont be deleted until the system gets shut down.
        let persistent_cluster_buffer = DeviceLocalBuffer::new(
            device.clone(), BufferUsage::all(), vec![queue.family()].into_iter()
        ).expect("failed to create cluster buffer!");

        //Store for fast usage
        let compute_pipeline: Arc<ComputePipelineAbstract + Send + Sync> = Arc::new(
            ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()
        )
        .expect("failed to create compute pipeline"));

        let descriptor_pool = FixedSizeDescriptorSetsPool::new(compute_pipeline.clone(), 0);

        let shadow_map_sampler = Sampler::new(
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
        ).expect("failed to create shadow sampler");

        let tmp_uniform_buffer_pool_05 = CpuBufferPool::<lights::ty::LightCount>::new(
            device.clone(), BufferUsage::all()
        );

        let light_count_tmp = lights::ty::LightCount{
            points: 0,
            directionals: 0,
            spots: 0,
        };

        let c_light_count = tmp_uniform_buffer_pool_05
        .next(light_count_tmp).expect("Failed to alloc first light count buffer");


        LightSystem{
            uniform_manager: uniform_manager,
            device: device,
            queue: queue,

            cluster_buffer: persistent_cluster_buffer,
            descriptor_pool: descriptor_pool,

            current_point_light_list: c_point_light,
            current_dir_light_list: c_dir_light,
            current_spot_light_list: c_spot_lights,
            current_light_count: c_light_count,
            buffer_pool_05_count: tmp_uniform_buffer_pool_05,

            compute_pipeline: compute_pipeline,

            shadow_map_sampler: shadow_map_sampler
        }
    }

    ///Analyses the lights we currently need, sends the to the shadow system to decide which light
    /// gets a shadow, and where. Then builds the uniform buffers for the lights which get used
    /// in the compute and shadow passes.
    pub fn update_light_set(
        &mut self,
        shadow_system: &mut ShadowSystem,
        asset_manager: &mut AssetManager
    ){

        //First compile a list of needed point, spot and directional lights

        //The frustum of the current camera. Since the bound of a light is always its influence
        // radius as well we can use this info to cull not usable spot and point lights
        let frustum = asset_manager.get_camera().get_frustum_bound();

        let comparer = Some(SceneComparer::new().with_frustum(frustum));

        let point_lights = {
            asset_manager.get_active_scene().copy_all_point_lights(&comparer)
        };

        let spot_lights = {
            asset_manager.get_active_scene().copy_all_spot_lights(&comparer)
        };

        //Since directional lights see everything we always use all of them
        let directional_lights = {
            asset_manager.get_active_scene().copy_all_directional_lights(&None)
        };


        //Send the lights to the shadow system to set the right shadow regions and get the shader_infos
        //This will set the atlases accordingly and return the correct shader infos which will be
        // Transformed into buffers for alter supply to the descriptorsets
        let (points, spots, directionals) = shadow_system.set_shadow_atlases(
            asset_manager,
            point_lights,
            spot_lights,
            directional_lights
        );

        let light_counts = LightCount{
            points: points.len() as u32,
            directionals: directionals.len() as u32,
            spots: spots.len() as u32,
        };

        //Now build the buffers from the shader_infos and update them internaly
        self.current_point_light_list = {
            let (buffer, future) = ImmutableBuffer::from_iter(
                points.into_iter(),
                BufferUsage::all(),
                self.queue.clone()
            ).expect("Failed to create point light buffer");
            //Now drop the future (which will execute and then return)
            buffer
        };

        self.current_spot_light_list = {
            let (buffer, future) = ImmutableBuffer::from_iter(
                spots.into_iter(),
                BufferUsage::all(),
                self.queue.clone()
            ).expect("Failed to create spot light buffer");
            //Now drop the future (which will execute and then return)
            buffer
        };
        self.current_dir_light_list = {
            let (buffer, future) = ImmutableBuffer::from_iter(
                directionals.into_iter(),
                BufferUsage::all(),
                self.queue.clone()
            ).expect("Failed to create directional light buffer");
            //Now drop the future (which will execute and then return)
            buffer
        };

        //And finally allocate a new buffer of light counts which describes the buffers above
        self.current_light_count = self.buffer_pool_05_count.next(
            light_counts
        ).expect("Failed to allocate new light count buffer")
    }


    pub fn dispatch_compute_shader(
        &mut self,
        command_buffer: frame_system::FrameStage,
    ) -> frame_system::FrameStage{

        match command_buffer{
            frame_system::FrameStage::LightCompute(cb) => {

                //adds the light buffers (all lights and indice buffer)
                let set_01 = self.descriptor_pool.next()
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
                ;


                //Now add to cb the dispatch
                let new_cb = cb.dispatch([32, 16, 32], self.compute_pipeline.clone(), set_01, ())
                .expect("failed to add compute operation");


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
    /// - Binding 4 = The texture with all directional shadows.
    pub fn get_light_descriptorset(
        &mut self, binding_id: u32,
        pipeline: Arc<vulkano::pipeline::GraphicsPipelineAbstract + Send + Sync>,
        frame_system: &frame_system::FrameSystem,
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
            //The shadow textures we have
            .add_sampled_image(
                frame_system.shadow_images.directional_shadows.clone(),
                self.shadow_map_sampler.clone()
            )
            .expect("Failed to add shadow map image")
            .build().expect("failed to build descriptor 04")
        );

        new_set
    }
}

///The compute shader used to compute the light matrix in world space.
pub mod light_cull_shader {
    #[derive(VulkanoShader)]
    #[ty = "compute"]
    #[path = "data/shader/light_culling.comp"]
    struct Dummy;
}
