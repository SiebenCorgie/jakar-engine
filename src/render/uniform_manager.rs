
//use render::shader_impls::pbr_fragment;
use render::shader::shader_inputs::default_data;
use render::shader::shader_inputs::lights;


use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano;

use cgmath::*;

use std::sync::Arc;

///Handles the public uniforms and an uniform allocator.
///
/// Public uniforms are:
/// - DATA (camera location, model transform, camera perspective, and view matrix)
/// - POINT_LIGHTS
/// - DIRECTIONAL_LIGTHS
/// - SPOT_LIGHTS
/// - LIGHT_COUNT (holds a variable representing the acutally used light number per type)
pub struct UniformManager {

    device: Arc<vulkano::device::Device>,

    ///Describes the universal world properties (see `render:://`)
    pub u_world: default_data::ty::Data,

    u_point_lights: Vec<lights::ty::PointLight>,
    u_directional_lights: Vec<lights::ty::DirectionalLight>,
    u_spot_lights: Vec<lights::ty::SpotLight>,

    u_light_count: lights::ty::LightCount,

    ///First uniform buffer pool block, used for model, view and perspecive matrix nas well as current
    /// camera location
    buffer_pool_01_mvp: vulkano::buffer::cpu_pool::CpuBufferPool<default_data::ty::Data>,

    ///5th for the light count fo each light type
    buffer_pool_05_count: vulkano::buffer::cpu_pool::CpuBufferPool<lights::ty::LightCount>,
}

//Create a buffer and the pool
//Recreate set in material not pipeline
impl UniformManager{
    pub fn new(device: Arc<vulkano::device::Device>) -> Self{

        //Create a uniform buffer with just [[f32; 4]; 4], the buffer will be updated bevore the first loop
        let world = default_data::ty::Data {
            camera_position: [0.0; 3],
            _dummy0: [0; 4],
            model : <Matrix4<f32>>::identity().into(),
            view : <Matrix4<f32>>::identity().into(),
            proj : <Matrix4<f32>>::identity().into(),
            near: 0.1,
            far: 100.0,
        };

        let light_count_tmp = lights::ty::LightCount{
            points: 0,
            directionals: 0,
            spots: 0,
        };

        //Create some pools to allocate from
        let tmp_uniform_buffer_pool_01 = CpuBufferPool::<default_data::ty::Data>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );

        let tmp_uniform_buffer_pool_05 = CpuBufferPool::<lights::ty::LightCount>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );



        UniformManager{

            device: device,

            u_world: world,

            u_point_lights: Vec::new(),
            u_directional_lights: Vec::new(),
            u_spot_lights: Vec::new(),

            u_light_count: light_count_tmp,

            ///First uniform buffer pool block, used or model, view and perspecive matrix
            buffer_pool_01_mvp: tmp_uniform_buffer_pool_01,

            buffer_pool_05_count: tmp_uniform_buffer_pool_05,

        }
    }

    ///Returns a subbuffer of the u_world item, can be used to create a u_world_set
    pub fn get_subbuffer_data (&mut self, transform_matrix: Matrix4<f32>) ->
    CpuBufferPoolSubbuffer<default_data::ty::Data, Arc<vulkano::memory::pool::StdMemoryPool>>{

        //prepare the Data struct
        let mut tmp_data_struct = self.u_world.clone();

        tmp_data_struct.model = transform_matrix.into();

        match self.buffer_pool_01_mvp.next(tmp_data_struct){
            Ok(k) => k,
            Err(e) => {
                println!("{:?}", e);
                panic!("failed to allocate new sub buffer!")
            },
        }
    }

    ///Sets the current light objects to be rendered. The resulting array will be used for all
    ///light calculations.
    pub fn set_point_lights(&mut self, new_point_lights: Vec<lights::ty::PointLight>){
        self.u_point_lights = new_point_lights;
        self.u_light_count.points = self.u_point_lights.len() as u32;
    }

    ///Returns a subbuffer of the u_point_light
    pub fn get_subbuffer_point_lights (&mut self) ->
    Arc<CpuAccessibleBuffer<[lights::ty::PointLight]>>{

        let current_points_copy = self.u_point_lights.clone(); //we don't want to free the vec in here each time :)

        CpuAccessibleBuffer::from_iter(
            self.device.clone(), BufferUsage::all(), current_points_copy.into_iter())
            .expect("failed to create PointLightBuffer")
    }

    ///Sets the current light objects to be rendered. The resulting array will be used for all
    ///light calculations.
    pub fn set_directional_lights(&mut self, new_dir_lights: Vec<lights::ty::DirectionalLight>){
        self.u_directional_lights = new_dir_lights;
        self.u_light_count.directionals = self.u_directional_lights.len() as u32;
    }

    ///Returns a subbuffer of the u_directional_light
    pub fn get_subbuffer_directional_lights (&mut self) ->
    Arc<CpuAccessibleBuffer<[lights::ty::DirectionalLight]>>{

        let current_dir_copy = self.u_directional_lights.clone(); //we don't want to free the vec in here each time :)

        CpuAccessibleBuffer::from_iter(
            self.device.clone(), BufferUsage::all(), current_dir_copy.into_iter())
            .expect("failed to create PointLightBuffer")
    }

    ///Sets the current light objects to be rendered. The resulting array will be used for all
    ///light calculations.
    pub fn set_spot_lights(&mut self, new_spot_lights: Vec<lights::ty::SpotLight>){
        self.u_spot_lights = new_spot_lights;
        self.u_light_count.spots = self.u_spot_lights.len() as u32;
    }

    ///Returns a subbuffer of the u_spot_light
    pub fn get_subbuffer_spot_lights (&mut self) ->
    Arc<CpuAccessibleBuffer<[lights::ty::SpotLight]>>{

        let current_spots_copy = self.u_spot_lights.clone(); //we don't want to free the vec in here each time :)

        CpuAccessibleBuffer::from_iter(
            self.device.clone(), BufferUsage::all(), current_spots_copy.into_iter())
            .expect("failed to create PointLightBuffer")
    }

    ///Returns a subbuffer of the light counts
    pub fn get_subbuffer_light_count (&mut self) ->
    CpuBufferPoolSubbuffer<lights::ty::LightCount, Arc<vulkano::memory::pool::StdMemoryPool>>{

        match self.buffer_pool_05_count.next(self.u_light_count.clone()){
            Ok(k) => k,
            Err(e) => {
                println!("{:?}", e);
                panic!("failed to allocate new sub buffer for light count!")
            },
        }
    }

    ///Updates the internal data used for the uniform buffer creation
    pub fn update(
        &mut self, new_u_world: default_data::ty::Data,
    ){
        self.u_world = new_u_world;
    }
}
