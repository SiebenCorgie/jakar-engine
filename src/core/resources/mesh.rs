//TODO Add command buffer creation per mesh
use std::sync::{Arc, Mutex};
use cgmath::*;
use collision;

use render::frame_system::{FrameStage, FrameSystem};
use render::light_system::LightSystem;
use render::render_traits::ForwardRenderAble;

use vulkano::buffer::ImmutableBuffer;
use vulkano::device::Device;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::BufferAccess;
use vulkano::device::Queue;


use core::ReturnBoundInfo;
use core::resources::material;



///Defines the information a Vertex should have
#[derive(Clone,Copy)]
pub struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2],
    normal: [f32; 3],
    tangent: [f32; 4],
    color: [f32; 4],
}

//Implements the vulkano::vertex trait on Vertex
impl_vertex!(Vertex, position, tex_coord, normal, tangent, color);

//TODO
//Every mesh needs its own indice and vertex buffer plus its pipeline to be drawn
impl Vertex{
    ///Creates a new Vertex
    pub fn new(
        position: [f32; 3],
        tex_coord: [f32; 2],
        normal: [f32; 3],
        tangent: [f32; 4],
        color: [f32; 4]
        ) -> Self
    {
        Vertex{
            position: position,
            tex_coord: tex_coord,
            normal: normal,
            tangent: tangent,
            color: color,
        }
    }
}

///Defines a mesh, a mesh mostly consists of: Name, Vertices (and the corresbondig vertex buffer)
///, the vertex indices, a material and its AABB (bounding box)
#[derive(Clone)]
pub struct Mesh {
    pub name: String,

    device: Arc<Device>,

    ///Holds the raw vertices of this mesh
    vertices: Vec<Vertex>,
    ///Holds the vulkan buffer, gets change if the vertices change
    vertex_buffer: Option<Arc<ImmutableBuffer<[Vertex]>>>,

    indices: Vec<u32>,

    index_buffer: Option<Arc<ImmutableBuffer<[u32]>>>,

    material: Arc<Mutex<material::Material>>,

    bound: collision::Aabb3<f32>,
}

impl Mesh {
    ///Returns the Member with the passed `name`
    pub fn new(
        name: &str,
        device: Arc<Device>,
        material: Arc<Mutex<material::Material>>
        )
        ->Self{
        //Creating the box extend from the location, there might be a better way
        let min = Point3::new(0.5, 0.5, 0.5);
        let max = Point3::new(0.5, 0.5, 0.5);

        Mesh{
            name: String::from(name),

            device: device.clone(),

            //TODO Create a persistend vertex and indice buffer
            vertices: Vec::new(),
            vertex_buffer: None,

            indices: Vec::new(),

            index_buffer: None,

            material: material,

            bound: collision::Aabb3::new(min, max),
        }
    }

    ///Sets the vertex and indice buffer to a new set of `Vertex` and `u32` indices
    ///The supplied queue will be used for uploading the buffer. If there are several, try to to use
    /// the worker queue for this job.
    pub fn set_vertices_and_indices(&mut self, vertices: Vec<Vertex>, indices: Vec<u32>, upload_queue: Arc<Queue>){

        //Set the new values
        self.vertices = vertices;
        self.indices = indices;
        //Rebuild vertex and indice buffer with new vertices
        self.re_create_buffer(upload_queue);
        //self.indices = indices;
    }

    ///Returns the name of the material this mesh uses
    #[inline]
    pub fn get_material_name(&self) -> String{

        let mat_lck = self.material.lock().expect("failed to lock meshs material");
        (mat_lck).get_name()
    }

    ///Returns the material in use by this mesh as clone
    #[inline]
    pub fn get_material(&self) -> Arc<Mutex<material::Material>>{
        self.material.clone()
    }

    ///Can be used to set the mesh's material to a new one
    #[inline]
    pub fn set_material(&mut self, new_mat: Arc<Mutex<material::Material>>){
        self.material = new_mat;
    }

    ///Returns all indices
    #[inline]
    pub fn get_indices(&self) -> Vec<u32>{
        self.indices.clone()
    }

    ///Return all vertices
    #[inline]
    pub fn get_all_vertices(&self) -> Vec<Vertex>{
        self.vertices.clone()
    }

    ///Returns all pos data
    #[inline]
    pub fn get_all_positions(&self)-> Vec<[f32; 3]>{
        let mut return_vec = Vec::new();
        for i in self.vertices.iter(){
            return_vec.push(i.position);
        }
        return_vec
    }

    ///Returns all pos data
    #[inline]
    pub fn get_all_uvs(&self)-> Vec<[f32; 2]>{
        let mut return_vec = Vec::new();
        for i in self.vertices.iter(){
            return_vec.push(i.tex_coord);
        }
        return_vec
    }

    ///Returns all pos data
    pub fn get_all_normals(&self)-> Vec<[f32; 3]>{
        let mut return_vec = Vec::new();
        for i in self.vertices.iter(){
            return_vec.push(i.normal);
        }
        return_vec
    }

    ///Returns all pos data
    pub fn get_all_tangents(&self)-> Vec<[f32; 4]>{
        let mut return_vec = Vec::new();
        for i in self.vertices.iter(){
            return_vec.push(i.tangent);
        }
        return_vec
    }

    ///Returns all pos data
    pub fn get_all_colors(&self)-> Vec<[f32; 4]>{
        let mut return_vec = Vec::new();
        for i in self.vertices.iter(){
            return_vec.push(i.color);
        }
        return_vec
    }

    ///Returns the current vertex buffer of this mesh
    pub fn get_vertex_buffer(&self) -> Option<Vec<Arc<BufferAccess + Send + Sync>>>{
        let mut return_vec = Vec::new();
        match self.vertex_buffer.clone(){
            Some(vb) => {
                return_vec.push(vb as Arc<BufferAccess + Send + Sync>);
                return Some(return_vec);
            }
            _ => return None,
        }

    }

    ///Recreates the vertex buffer from a specified device and queue
    pub fn re_create_buffer(&mut self, upload_queue: Arc<Queue>)
    {
        //create both buffers and wait for the graphics card to actually upload them
        let (vertex_buffer, buffer_future) = ImmutableBuffer::from_iter(
            self.vertices.iter().cloned(),
            BufferUsage::all(),
            upload_queue.clone()
        ).expect("failed to create vertex buffer");





        let (index_buffer, future) = ImmutableBuffer::from_iter(
            self.indices.iter().cloned(),
            BufferUsage::all(),
            upload_queue.clone()
        ).expect("failed to create index buffer");

        //overwrite internally
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);

    }


    ///Returns a index bufffer for this mesh
    pub fn get_index_buffer(&self) ->
        Option<Arc<ImmutableBuffer<[u32]>>>
    {
        self.index_buffer.clone()
    }

    ///Renders this mesh if the supplied framestage is in the froward stage
    pub fn draw(
        &self,
        frame_stage: FrameStage,
        frame_system: &FrameSystem,
        light_system: &LightSystem,
        transform: Matrix4<f32>,
    ) -> FrameStage{
        //Before doing anything, we check if that mesh is active, if not we just pass
        if self.vertex_buffer.is_none(){
            return frame_stage;
        }

        match frame_stage{
            FrameStage::Forward(cb) => {

                let material_locked = self.get_material();
                let mut material = material_locked
                .lock()
                .expect("failed to lock mesh for command buffer generation");

                let pipeline = material.get_vulkano_pipeline();

                let set_01 = {
                    //aquirre the tranform matrix and generate the new set_01
                    material.get_set_01(transform)
                };

                let set_02 = {
                    material.get_set_02()
                };

                let set_03 = {
                    material.get_set_03()
                };

                let set_04 = {
                    material.get_set_04(&light_system, &frame_system)
                };

                //extend the current command buffer by this mesh
                let new_cb = cb.draw_indexed(
                    pipeline,
                    frame_system.get_dynamic_state().clone(),
                    self.get_vertex_buffer().expect("Found no vertex buffer, should not happen"), //vertex buffer (static usually)
                    self.get_index_buffer().expect("Found no index buffer, should not happen"), //index buffer
                    (set_01, set_02, set_03, set_04), //descriptor sets (currently static)
                    ()
                )
                .expect("Failed to draw mesh in command buffer!");

                return FrameStage::Forward(new_cb);
            },
            _ => {
                println!("Tried to draw mesh in wrong stage!", );
            }
        }

        return frame_stage;
    }

}

impl ReturnBoundInfo for Mesh{
    ///return the max size of its bound
    #[inline]
    fn get_bound_max(&self)-> Point3<f32>{
        self.bound.max.clone()
    }
    ///return the min size of its bound
    #[inline]
    fn get_bound_min(&self)-> Point3<f32>{
        self.bound.min.clone()
    }
    ///Sets the bound to the new values (in mesh space)
    #[inline]
    fn set_bound(&mut self, min: Point3<f32>, max: Point3<f32>){
        self.bound = collision::Aabb3::new(min, max);
    }

    ///Returns it' bound
    #[inline]
    fn get_bound(&self) -> collision::Aabb3<f32>{
        self.bound.clone()
    }


    ///Returns the vertices of the bounding mesh, good for debuging
    fn get_bound_points(&self)-> Vec<Vector3<f32>>{
        let mut return_vector = Vec::new();

        let b_min = self.bound.min.clone();
        let b_max = self.bound.max.clone();

        //low
        return_vector.push(Vector3::new(b_min[0], b_min[1], b_min[2])); //Low
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1], b_min[2])); //+x
        return_vector.push(Vector3::new(b_min[0], b_min[1] + b_max[1], b_min[2])); //+y
        return_vector.push(Vector3::new(b_min[0],  b_min[1], b_min[2] + b_max[2])); // +z
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1] + b_max[1], b_min[2])); //+xy
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1], b_min[2] + b_max[2])); //+xz
        return_vector.push(Vector3::new(b_min[0] , b_min[1] + b_max[1], b_min[2] + b_max[1])); //+yz
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1] + b_max[1], b_min[2] + b_max[2])); //+xyz

        return_vector
    }

    ///Rebuilds nothing, but might be able in the future to actually rebuild the bound based on all of the vertexes
    fn rebuild_bound(&mut self){
        //

    }
}
