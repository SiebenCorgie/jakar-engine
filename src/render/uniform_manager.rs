
//use render::shader_impls::pbr_fragment;
use render::shader::shader_inputs::default_data;
use render::shader::shader_inputs::lights;


use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::ImmutableBuffer;
use vulkano;

use cgmath::*;

use std::sync::Arc;

///Handles the public uniforms and an uniform allocator.
///
/// Public uniforms are:
/// - DATA (camera location, model transform, camera perspective, and view matrix)
pub struct UniformManager {

    device: Arc<vulkano::device::Device>,
    ///Describes the universal world properties (see `render:://`)
    pub u_world: default_data::ty::Data,
    ///First uniform buffer pool block, used for model, view and perspecive matrix nas well as current
    /// camera location
    buffer_pool_01_mvp: vulkano::buffer::cpu_pool::CpuBufferPool<default_data::ty::Data>,

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
            ///First uniform buffer pool block, used or model, view and perspecive matrix
            buffer_pool_01_mvp: tmp_uniform_buffer_pool_01,
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

    ///Updates the internal data used for the uniform buffer creation
    pub fn update(
        &mut self, new_u_world: default_data::ty::Data,
    ){
        self.u_world = new_u_world;
    }
}
