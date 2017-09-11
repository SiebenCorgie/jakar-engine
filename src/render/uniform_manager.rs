
use render::shader_impls::pbr_fragment;

use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano;

use cgmath::*;

use std::sync::Arc;

///Handles the public uniforms and an uniform allocator
pub struct UniformManager {
    ///Describes the universal world properties (see `render:://`)
    pub u_world: pbr_fragment::ty::Data,

    u_point_lights: pbr_fragment::ty::point_lights,
    u_directional_lights: pbr_fragment::ty::directional_lights,
    u_spot_lights: pbr_fragment::ty::spot_lights,

    u_light_count: pbr_fragment::ty::LightCount,


    ///First uniform buffer pool block, used or model, view and perspecive matrix
    buffer_pool_01_mvp: vulkano::buffer::cpu_pool::CpuBufferPool<pbr_fragment::ty::Data>,

    ///4th uniform buffer pool block, used for point lights
    buffer_pool_02_point: vulkano::buffer::cpu_pool::CpuBufferPool<pbr_fragment::ty::point_lights>,

    ///4th uniform buffer pool block, used for directional lights
    buffer_pool_03_dir: vulkano::buffer::cpu_pool::CpuBufferPool<pbr_fragment::ty::directional_lights>,

    ///4th uniform buffer pool block, used for spot lights
    buffer_pool_04_spot: vulkano::buffer::cpu_pool::CpuBufferPool<pbr_fragment::ty::spot_lights>,
    ///5th for the light count
    buffer_pool_05_count: vulkano::buffer::cpu_pool::CpuBufferPool<pbr_fragment::ty::LightCount>,

}

//Create a buffer and the pool
//Recreate set in material not pipeline
impl UniformManager{
    pub fn new(device: Arc<vulkano::device::Device>) -> Self{

        //Create a uniform buffer with just [[f32; 4]; 4], the buffer will be updated bevore the first loop
        let world = pbr_fragment::ty::Data {
            camera_position: [0.0; 3],
            _dummy0: [0; 4],
            model : <Matrix4<f32>>::identity().into(),
            view : <Matrix4<f32>>::identity().into(),
            proj : <Matrix4<f32>>::identity().into(),
        };


        /*
        let points = //s::PointLightInfo{
            l_point: Vec::new(),
        };
        */
        let points = {
            let empty_light = pbr_fragment::ty::PointLight{
                color: [1.0; 3],
                location: [0.0; 3],
                intensity: 0.0,
                _dummy0: [0; 4],
            };

            let add_array = [empty_light.clone(); 6];

            pbr_fragment::ty::point_lights{
                p_light: add_array,
            }

        };
        /*
        let direct = //s::DirectionlLightInfo{
            l_directional: Vec::new(),
        };
        */
        let direct = {
            let empty_light = pbr_fragment::ty::DirectionalLight{
                color: [1.0; 3],
                direction: [1.0; 3],
                location: [0.0; 3],
                intensity: 0.0,
                _dummy0: [0; 4],
                _dummy1: [0; 4],
            };
            let add_array = [empty_light.clone(); 6];

            pbr_fragment::ty::directional_lights{
                d_light: add_array,
            }
        };
        /*
        let spots = //s::SpotLightInfo{
            l_spot: Vec::new(),
        };
        */
        let spots = {
            let empty_light = pbr_fragment::ty::SpotLight{
                color: [1.0; 3],
                direction: [1.0; 3],
                location: [0.0; 3],
                intensity: 0.0,
                outer_radius: 0.0,
                inner_radius: 0.0,
                _dummy0: [0; 4],
                _dummy1: [0; 4],
                _dummy2: [0; 8],
            };
            let add_array = [empty_light.clone(); 6];

            pbr_fragment::ty::spot_lights{
                s_light: add_array,
            }
        };


        let light_count_tmp = pbr_fragment::ty::LightCount{
            points: 0,
            directionals: 0,
            spots: 0,
        };

        //Create some pools to allocate from
        let tmp_uniform_buffer_pool_01 = CpuBufferPool::<pbr_fragment::ty::Data>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );

        let tmp_uniform_buffer_pool_02 = CpuBufferPool::<pbr_fragment::ty::point_lights>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );

        let tmp_uniform_buffer_pool_03 = CpuBufferPool::<pbr_fragment::ty::directional_lights>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );

        let tmp_uniform_buffer_pool_04 = CpuBufferPool::<pbr_fragment::ty::spot_lights>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );

        let tmp_uniform_buffer_pool_05 = CpuBufferPool::<pbr_fragment::ty::LightCount>::new(
            device.clone(), vulkano::buffer::BufferUsage::all()
        );



        UniformManager{

            u_world: world,

            u_point_lights: points,
            u_directional_lights: direct,
            u_spot_lights: spots,

            u_light_count: light_count_tmp,

            ///First uniform buffer pool block, used or model, view and perspecive matrix
            buffer_pool_01_mvp: tmp_uniform_buffer_pool_01,

            ///4th uniform buffer pool block, used for point lights
            buffer_pool_02_point: tmp_uniform_buffer_pool_02,

            ///4th uniform buffer pool block, used for directional lights
            buffer_pool_03_dir: tmp_uniform_buffer_pool_03,

            ///4th uniform buffer pool block, used for spot lights
            buffer_pool_04_spot: tmp_uniform_buffer_pool_04,

            buffer_pool_05_count: tmp_uniform_buffer_pool_05,

        }
    }

    ///Returns a subbuffer of the u_world item, can be used to create a u_world_set
    pub fn get_subbuffer_01 (&mut self, transform_matrix: Matrix4<f32>) ->
    CpuBufferPoolSubbuffer<pbr_fragment::ty::Data, Arc<vulkano::memory::pool::StdMemoryPool>>{

        //prepare the Data struct
        let mut tmp_data_struct = self.u_world.clone();

        tmp_data_struct.model = transform_matrix.into();

        self.buffer_pool_01_mvp.next(tmp_data_struct)
    }

    ///Returns a subbuffer of the u_point_light
    pub fn get_subbuffer_02 (&mut self) ->
    CpuBufferPoolSubbuffer<pbr_fragment::ty::point_lights, Arc<vulkano::memory::pool::StdMemoryPool>>{
        self.buffer_pool_02_point.next(self.u_point_lights.clone())
    }

    ///Returns a subbuffer of the u_directional_light
    pub fn get_subbuffer_03 (&mut self) ->
    CpuBufferPoolSubbuffer<pbr_fragment::ty::directional_lights, Arc<vulkano::memory::pool::StdMemoryPool>>{
        self.buffer_pool_03_dir.next(self.u_directional_lights.clone())
    }

    ///Returns a subbuffer of the u_spot_light
    pub fn get_subbuffer_04 (&mut self) ->
    CpuBufferPoolSubbuffer<pbr_fragment::ty::spot_lights, Arc<vulkano::memory::pool::StdMemoryPool>>{
        self.buffer_pool_04_spot.next(self.u_spot_lights.clone())
    }

    ///Returns a subbuffer of the u_spot_light
    pub fn get_subbuffer_05 (&mut self) ->
    CpuBufferPoolSubbuffer<pbr_fragment::ty::LightCount, Arc<vulkano::memory::pool::StdMemoryPool>>{
        self.buffer_pool_05_count.next(self.u_light_count.clone())
    }


    ///Updates the internal data used for the uniform buffer creation
    pub fn update(
        &mut self, new_u_world: pbr_fragment::ty::Data,
        new_point: pbr_fragment::ty::point_lights,
        new_dir: pbr_fragment::ty::directional_lights,
        new_spot: pbr_fragment::ty::spot_lights,
        count_point: u32,
        count_dir: u32,
        count_spot: u32,
    ){
        self.u_world = new_u_world;
        self.u_point_lights = new_point;
        self.u_directional_lights = new_dir;
        self.u_spot_lights = new_spot;
        self.u_light_count = pbr_fragment::ty::LightCount{
            points: count_point,
            directionals: count_dir,
            spots: count_spot,
        }
    }
}
