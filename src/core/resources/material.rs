use render::uniform_manager;
use core::resources::texture;
use core::resources::mesh;
use render::shader_impls::pbr_fragment;

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano;

use cgmath::*;

use std::sync::{Mutex,Arc};


///A Struct used for prototyping the usage flags of the textures
#[derive(Clone)]
pub struct TextureUsageFlags {
    pub albedo: i32,
    pub normal: i32,
    pub metal: i32,
    pub roughness: i32,
    pub occlusion: i32,
    pub emissive: i32
    //TODO Implement additional textures:
    // -
}

impl TextureUsageFlags{
    ///Creates a new flag info where all textures are used
    pub fn new() -> Self{
        TextureUsageFlags{
            albedo: 0,
            normal: 0,
            metal: 0,
            roughness: 0,
            occlusion: 0,
            emissive: 0,
        }
    }


    ///Creates with a set albedo status
    pub fn with_albedo(mut self, albedo: i32) ->Self{
        self.albedo = albedo;
        self
    }

    ///Creates with a set normal status
    pub fn with_normal(mut self, normal: i32) ->Self{
        self.normal = normal;
        self
    }

    ///Creates with a set metal status
    pub fn with_metal(mut self, metal: i32) ->Self{
        self.metal = metal;
        self
    }

    ///Creates with a set roughness status
    pub fn with_roughness(mut self, roughness: i32) ->Self{
        self.roughness = roughness;
        self
    }

    ///Creates with a set occlusion status
    pub fn with_occlusion(mut self, occlusion: i32) ->Self{
        self.occlusion = occlusion;
        self
    }

    ///Creates with a set emissive status
    pub fn with_emissive(mut self, emissive: i32) ->Self{
        self.emissive = emissive;
        self
    }

    pub fn to_shader_flags(self) -> pbr_fragment::ty::TextureUsageInfo{
        pbr_fragment::ty::TextureUsageInfo{
            b_albedo: {
                if self.albedo != 0{
                    1
                }else{
                    0
                }
            },
            b_normal: {
                if self.normal != 0{
                    1
                }else{
                    0
                }
            },
            b_metal: {
                if self.metal != 0{
                    1
                }else{
                    0
                }
            },
            b_roughness: {
                if self.roughness != 0{
                    1
                }else{
                    0
                }
            },
            b_occlusion: {
                if self.occlusion != 0{
                    1
                }else{
                    0
                }
            },
            b_emissive: {
                if self.emissive != 0{
                    1
                }else{
                    0
                }
            },
        }
    }
}

///A Struct defining the the material factors. They are used as Colors/factors if no textures
/// are present
#[derive(Clone)]
pub struct MaterialFactors{
    albedo_factor: [f32; 4],
    normal_factor: f32,
    emissive_factor: [f32; 3],
    metal_factor: f32,
    roughness_factor: f32,
    occlusion_factor: f32,

}

impl MaterialFactors{
    ///Creates a set of default factors
    pub fn new()-> Self{
        MaterialFactors{
            albedo_factor: [1.0; 4],
            //this needs to be set to just blue for not manipulating the rest
            normal_factor: 1.0,
            emissive_factor: [1.0; 3],
            metal_factor: 1.0,
            roughness_factor: 1.0,
            occlusion_factor: 1.0,
        }
    }


    ///Creates the Factor struct with a given albdeo factor
    pub fn with_factor_albedo(mut self, factor: [f32; 4]) -> Self{
        self.albedo_factor = factor;
        self
    }

    ///Creates the Factor struct with a given normal factor
    pub fn with_factor_normal(mut self, factor: f32) -> Self{
        self.normal_factor = factor;
        self
    }

    ///Creates the Factor struct with a given metal factor
    pub fn with_factor_metal(mut self, factor: f32) -> Self{
        self.metal_factor = factor;
        self
    }

    ///Creates the Factor struct with a given roughness factor
    pub fn with_factor_roughness(mut self, factor: f32) -> Self{
        self.roughness_factor = factor;
        self
    }

    ///Creates the Factor struct with a given occlusion factor
    pub fn with_factor_occlusion(mut self, factor: f32) -> Self{
        self.occlusion_factor = factor;
        self
    }

    ///Creates the Factor struct with a given emissive factor
    pub fn with_factor_emissive(mut self, factor: [f32; 3]) -> Self{
        self.emissive_factor = factor;
        self
    }

    pub fn to_shader_factors(&self) -> pbr_fragment::ty::TextureFactors{
        pbr_fragment::ty::TextureFactors{
            albedo_factor: self.albedo_factor,
            normal_factor: self.normal_factor,
            emissive_factor: self.emissive_factor,
            metal_factor: self.metal_factor,
            roughness_factor: self.roughness_factor,
            occlusion_factor: self.occlusion_factor,
        }
    }
}




///A Structure used to build a material from in the MaterialBuilder described attributes
pub struct MaterialBuilder {
    albedo: Option<Arc<texture::Texture>>,
    normal: Option<Arc<texture::Texture>>,
    metallic_roughness: Option<Arc<texture::Texture>>,
    occlusion: Option<Arc<texture::Texture>>,
    emissive: Option<Arc<texture::Texture>>,
    fallback_texture: Arc<texture::Texture>,
    //texture and material infos
    texture_usage_info: TextureUsageFlags,
    material_factors: MaterialFactors,

}

impl MaterialBuilder{
    ///Creates a new Builder for this `texture::Texture`s with default parameters
    pub fn new(
        albedo: Option<Arc<texture::Texture>>,
        normal: Option<Arc<texture::Texture>>,
        metallic_roughness: Option<Arc<texture::Texture>>,
        occlusion: Option<Arc<texture::Texture>>,
        emissive: Option<Arc<texture::Texture>>,
        fallback_texture: Arc<texture::Texture>,
    ) -> Self {

        //Sort out the texture usage flags for this material
        let mut used_albedo = 0;
        let mut used_normal = 0;
        let mut used_emissive = 0;
        let mut used_physical = 0;
        let mut used_occlusion = 0;

        match albedo.clone(){
            Some(_) => used_albedo = 1,
             _=> {},
        }
        match normal.clone(){
            Some(_) => used_normal = 1,
             _=> {},
        }
        match metallic_roughness.clone(){
            Some(_) => used_physical = 1,
             _=> {},
        }
        match occlusion.clone(){
            Some(_) => used_occlusion = 1,
             _=> {},
        }
        match emissive.clone(){
            Some(_) => used_emissive = 1,
             _=> {},
        }

        //Create the usag flags from the info
        let mut usage_flags = TextureUsageFlags::new();
        usage_flags.albedo = used_albedo;
        usage_flags.normal = used_normal;
        usage_flags.roughness = used_physical;
        usage_flags.metal = used_physical;
        usage_flags.occlusion = used_occlusion;
        usage_flags.emissive = used_emissive;

        MaterialBuilder{
            albedo: albedo,
            normal: normal,
            metallic_roughness: metallic_roughness,
            occlusion: occlusion,
            emissive: emissive,
            fallback_texture: fallback_texture,
            //texture and material infos as shader usable struct
            texture_usage_info: usage_flags,
            //and default factors
            material_factors: MaterialFactors::new(),
        }
    }

    ///can be used to set different usage flags, most of the flag should be sorted correctly by
    ///the supplied textures though
    pub fn with_usage_flags(mut self, new_flags: TextureUsageFlags) -> Self{
        self.texture_usage_info = new_flags;
        self
    }

    ///can be used to set custom factors
    pub fn with_factors(mut self, new_factors: MaterialFactors) -> Self{
        self.material_factors = new_factors;
        self
    }

    ///builds a material from the supplied textures and other info
    pub fn build(
        self,
        name: &str,
        pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
        uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
        device: Arc<vulkano::device::Device>,
    ) -> Material{
        //find out if a texture was supplied per slot
        //if not return the fallback texture for this builder
        //should usally be the 1x1 pixel texture
        let tmp_albedo = {
            match self.albedo{
                Some(texture) => texture,
                None => self.fallback_texture.clone(),
            }
        };

        let tmp_normal = {
            match self.normal{
                Some(texture) => texture,
                None => self.fallback_texture.clone(),
            }
        };

        let tmp_physical = {
            match self.metallic_roughness{
                Some(texture) => texture,
                None => self.fallback_texture.clone(),
            }
        };

        let tmp_occlusion = {
            match self.occlusion{
                Some(texture) => texture,
                None => self.fallback_texture.clone(),
            }
        };

        let tmp_emissive = {
            match self.emissive{
                Some(texture) => texture,
                None => self.fallback_texture.clone(),
            }
        };

        //The TextureUsageFlags and Factor Info comes from the builder, we create a pool for
        //them...
        //Create a pool to allocate from
        let usage_info_pool = vulkano::buffer::cpu_pool::CpuBufferPool::<pbr_fragment::ty::TextureUsageInfo>
                                   ::new(device.clone(), vulkano::buffer::BufferUsage::all());


        let material_factor_pool = vulkano::buffer::cpu_pool::CpuBufferPool::<pbr_fragment::ty::TextureFactors>
                                   ::new(device.clone(), vulkano::buffer::BufferUsage::all());


        //Additionaly lock the uniformanager to get the first global information
        let uniform_manager_isnt = uniform_manager.clone();
        let mut uniform_manager_lck = uniform_manager_isnt.lock().expect("Failed to locj unfiorm_mng");

        //TODO add set 02 for material information
        //println!("STATUS: MATERIAL: Creating set 01 for the first time", );
        let ident_mat_4: Matrix4<f32> = Matrix4::identity();
        let set_01 = Arc::new(PersistentDescriptorSet::start(
                pipeline.clone(), 0
            )
            .add_buffer((*uniform_manager_lck).get_subbuffer_01(ident_mat_4).clone()).expect("Failed to create descriptor set")

            .build().expect("failed to build first descriptor 01")
        );


        //Create the set 02
        //println!("STATUS: MATERIAL: Creating set 02 for the first time", );
        let set_02 = Arc::new(
            PersistentDescriptorSet::start(
                pipeline.clone(), 1
            )
            .add_sampled_image(tmp_albedo.get_raw_texture(), tmp_albedo.get_raw_sampler())
            .expect("failed to add sampled albedo")
            .add_sampled_image(tmp_normal.get_raw_texture(), tmp_normal.get_raw_sampler())
            .expect("failed to add sampled nrm")
            .add_sampled_image(tmp_physical.get_raw_texture(), tmp_physical.get_raw_sampler())
            .expect("failed to add sampled physical")
            .add_sampled_image(tmp_occlusion.get_raw_texture(), tmp_occlusion.get_raw_sampler())
            .expect("failed to add occlusion texture")
            .add_sampled_image(tmp_emissive.get_raw_texture(), tmp_emissive.get_raw_sampler())
            .expect("failed to add emissive texture")
            .build().expect("failed to build set_02")
        );

        //Create the Usage Flag descriptor
        let set_03 = Arc::new(PersistentDescriptorSet::start(
                pipeline.clone(), 2
            )
            .add_buffer(usage_info_pool.next(
                //need to clone for storing in struct later
                self.texture_usage_info.clone().to_shader_flags()
            ).clone()).expect("Failed to create descriptor set")
            .add_buffer(material_factor_pool.next(
                //need to clone for storing in struct later
                self.material_factors.clone().to_shader_factors()
            ).clone()).expect("failed to create the first material factor pool")
            .build().expect("failed to build first descriptor 03")
        );

        //Creates thje first descriptor set 04
        let set_04 = Arc::new(PersistentDescriptorSet::start(
                pipeline.clone(), 3
            )
            .add_buffer((*uniform_manager_lck).get_subbuffer_02())
            .expect("Failed to create descriptor set")
            .add_buffer((*uniform_manager_lck).get_subbuffer_03())
            .expect("Failed to create descriptor set")
            .add_buffer((*uniform_manager_lck).get_subbuffer_04())
            .expect("Failed to create descriptor set")
            .add_buffer((*uniform_manager_lck).get_subbuffer_05())
            .expect("Failed to create descriptor set")
            .build().expect("failed to build first descriptor 04")
        );

        //Now create the new material
        Material{
            name: String::from(name),
            //albedo describtion
            t_albedo: tmp_albedo,
            //normal
            t_normal: tmp_normal,
            //Physical
            t_metallic_roughness: tmp_physical,
            //Occlusion
            t_occlusion: tmp_occlusion,
            //Additional textures
            t_emissive: tmp_emissive,

            //All Unifrom infos
            pipeline: pipeline,

            uniform_manager: uniform_manager,

            set_01: set_01,

            set_02: set_02,

            set_03: set_03,

            texture_usage_info: self.texture_usage_info.to_shader_flags(),
            usage_info_pool: usage_info_pool,

            material_factors: self.material_factors.to_shader_factors(),
            material_factor_pool: material_factor_pool,

            set_04: set_04,
        }
    }
}


///The material descibes the physical implementation of the material
///It mostly contains three textures:
/// - albedo: the color representation (without light)
/// - normal: the normal representation of the surface
/// - metallic-roughness: is a system texture which is split by channels:
/// - occlusion: is a system texture used to make some areas darker
///
/// The metallic-roughness  texture.
///
/// This texture has two components:
///
/// * The first component (R) contains the metallic-ness of the material.
/// * The second component (G) contains the roughness of the material.
/// * If the third component (B) and/or the fourth component (A) are present
///   then they are ignored.

///Describes a standart material
pub struct Material {

    pub name: String,
    //albedo describtion
    t_albedo: Arc<texture::Texture>,
    //normal
    t_normal: Arc<texture::Texture>,
    //metallic_roughness
    t_metallic_roughness: Arc<texture::Texture>,
    ///occlusion texture
    t_occlusion: Arc<texture::Texture>,
    //Additional textures: TODO implent
    t_emissive: Arc<texture::Texture>,

    //Technical implementation
    ///Reference to parent pipeline
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    ///A reference to the global uniform manager
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,

    //The set for the u_world information
    set_01: Arc<DescriptorSet + Send + Sync>,

    //A persistent material set which only needs to be alter if a texture changes
    set_02: Arc<DescriptorSet + Send + Sync>,

    //Usage flags of the different buffers, stored in a seperate set as well as material factors buffer
    set_03: Arc<DescriptorSet + Send + Sync>,
    //as shader usable struct
    texture_usage_info: pbr_fragment::ty::TextureUsageInfo,
    usage_info_pool: vulkano::buffer::cpu_pool::CpuBufferPool<pbr_fragment::ty::TextureUsageInfo>,

    material_factors: pbr_fragment::ty::TextureFactors,
    material_factor_pool: vulkano::buffer::cpu_pool::CpuBufferPool<pbr_fragment::ty::TextureFactors>,
    //Responsible for lighting information
    set_04: Arc<DescriptorSet + Send + Sync>,
}


impl Material {

    ///Returns the used uniform manager
    pub fn get_uniform_manager(&self) -> Arc<Mutex<uniform_manager::UniformManager>>{
        self.uniform_manager.clone()
    }

    ///Adds a albedo texture to the material
    pub fn set_albedo_texture(&mut self, albedo: Arc<texture::Texture>){
        self.t_albedo = albedo;
        self.texture_usage_info.b_albedo = 1;
    }

    ///Adds a normal Texture
    pub fn set_normal_texture(&mut self, normal: Arc<texture::Texture>){
        self.t_normal = normal;
        self.texture_usage_info.b_normal = 1;
    }

    ///Adds a physical texture
    pub fn set_metallic_roughness_texture(&mut self, physical: Arc<texture::Texture>){
        self.t_metallic_roughness = physical;
    }

    ///Adds a emissive texture
    pub fn set_emissive_texture(&mut self, emissive: Arc<texture::Texture>){
        self.t_emissive = emissive;
        self.texture_usage_info.b_emissive = 1;
    }

    pub fn set_texture_usage_info(&mut self, info: TextureUsageFlags){
        self.texture_usage_info = info.to_shader_flags();
    }

    ///Sets the material factors
    pub fn set_material_factor_info(&mut self, info: MaterialFactors){
        self.material_factors = info.to_shader_factors();
    }

    ///Recreates set_02, set_03
    pub fn recreate_static_sets(&mut self){
        //println!("STATUS: MATERIAL: Recreation static sets", );
        //Create the set 02
        //println!("STATUS: MATERIAL: ReCreating set 02", );
        let set_02 = Arc::new(
            PersistentDescriptorSet::start(
                self.pipeline.clone(), 1
            )
            .add_sampled_image(
                self.t_albedo.get_raw_texture().clone(), self.t_albedo.get_raw_sampler().clone()
            )
            .expect("failed to add sampled albedo")
            .add_sampled_image(
                self.t_normal.get_raw_texture().clone(), self.t_normal.get_raw_sampler().clone()
            )
            .expect("failed to add sampled nrm")
            .add_sampled_image(
                self.t_metallic_roughness.get_raw_texture().clone(), self.t_metallic_roughness.get_raw_sampler().clone()
            )
            .expect("failed to add sampled physical")
            .add_sampled_image(
                self.t_occlusion.get_raw_texture().clone(), self.t_occlusion.get_raw_sampler().clone()
            )
            .expect("failed to add sampled physical")
            .add_sampled_image(
                self.t_emissive.get_raw_texture().clone(), self.t_emissive.get_raw_sampler().clone()
            )
            .expect("failed to add sampled physical")
            .build().expect("failed to build set_02")
        );

        self.set_02 = set_02;

        //Create the Usage Flag descriptor
        let set_03 = Arc::new(PersistentDescriptorSet::start(
                self.pipeline.clone(), 2
            )
            .add_buffer(
                self.get_usage_info_subbuffer()
            ).expect("Failed to create descriptor set")
            .add_buffer(
                self.get_material_factor_subbuffer()
            ).expect("failed to create the material factor pool")
            .build().expect("failed to build descriptor 03")
        );

        self.set_03 = set_03;
    }


    ///Returns the name of the currently used pipeline
    pub fn get_pipeline(&self) -> Arc<GraphicsPipelineAbstract + Send + Sync>{
        self.pipeline.clone()
    }

    ///Updates all sets tied to this material
    pub fn update(&mut self){
        //println!("STATUS: MATERIAL: In material, updating now", );
        //The first set is now recreted on request from a transform matrix
        self.recreate_set_04();
        //if needed, update the static sets
    }

    ///Recreates set_01 based on the current unfiorm_manager information (mvp matrix)
    pub fn recreate_set_01(&mut self, transform_matrix: Matrix4<f32>){

        //println!("STATUS: MATERIAL: Trying to lock uniform manager", );
        let mut uniform_manager_lck = self.uniform_manager.lock().expect("Failed to locj unfiorm_mng");
        //println!("STATUS: MATERIAL: Generation new set_01", );
        let new_set = Arc::new(PersistentDescriptorSet::start(
                self.pipeline.clone(), 0
            )
            .add_buffer((*uniform_manager_lck).get_subbuffer_01(transform_matrix)).expect("Failed to create descriptor set")
            .build().expect("failed to build descriptor 01")
        );
        //println!("STATUS: MATERIAL: Returning new set to self", );
        //return the new set
        self.set_01 = new_set;
    }

    ///Recreates set_04 based on the current unfiorm_manager information (light)
    ///NOTE:
    /// - Binding 0 = point lights
    /// - Binding 1 = directional lights
    /// - Binding 2 = spot lights
    /// - Binding 3 = struct which describes how many actual lights wher sent
    ///*ENHANCE*: This and the first set could be created in the uniform manager because they are
    ///always the same
    pub fn recreate_set_04(&mut self){


        //TODO Add the buffers of the uniform manager to the descriptor set
        let mut uniform_manager_lck = self.uniform_manager.lock().expect("Failed to locj unfiorm_mng");
        //println!("STATUS: MATERIAL: Generation new set_04", );
        let new_set = Arc::new(PersistentDescriptorSet::start(
                self.pipeline.clone(), 3
            )
            .add_buffer((*uniform_manager_lck).get_subbuffer_02())
            .expect("Failed to create descriptor set")
            .add_buffer((*uniform_manager_lck).get_subbuffer_03())
            .expect("Failed to create descriptor set")
            .add_buffer((*uniform_manager_lck).get_subbuffer_04())
            .expect("Failed to create descriptor set")
            .add_buffer((*uniform_manager_lck).get_subbuffer_05())
            .expect("Failed to create descriptor set")
            .build().expect("failed to build descriptor 04")
        );
        //println!("STATUS: MATERIAL: Returning new set to self", );
        //return the new set
        self.set_04 = new_set;
    }

    ///Returns a subbuffer from the `usage_info_pool` to be used when adding a buffer to a set
    fn get_usage_info_subbuffer(&self) ->
     vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer<pbr_fragment::ty::TextureUsageInfo,
     Arc<vulkano::memory::pool::StdMemoryPool>>
     {
        self.usage_info_pool.next(self.texture_usage_info.clone())
    }

    ///Returns a subbuffer from the material_factor_pool to be used with the 3rd set
    fn get_material_factor_subbuffer(&self) ->
    vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer<pbr_fragment::ty::TextureFactors,
    Arc<vulkano::memory::pool::StdMemoryPool>>
    {
        self.material_factor_pool.next(self.material_factors.clone())
    }

    ///Returns the u_world_set generated from a model specific `transform_matrix` as well as the
    ///global view and projection matrix
    pub fn get_set_01(&mut self, transform_matrix: Matrix4<f32>) -> Arc<DescriptorSet + Send + Sync>{

        self.recreate_set_01(transform_matrix);
        self.set_01.clone()
    }

    ///Returns the second set which holds the material textures
    pub fn get_set_02(&self) -> Arc<DescriptorSet + Send + Sync>{
        self.set_02.clone()

    }

    ///Returns the 3rd descriptor set, responsible for the material specific settings
    pub fn get_set_03(&self) -> Arc<DescriptorSet + Send + Sync>{
        self.set_03.clone()
    }

    ///Returns the 4th desciptor set responsible for the lighting information
    pub fn get_set_04(&self) -> Arc<DescriptorSet + Send + Sync>{
        self.set_04.clone()
    }

    ///Sets a new pipeline
    pub fn set_pipeline(&mut self, new_pipe: Arc<GraphicsPipelineAbstract + Send + Sync>){
        self.pipeline = new_pipe;
    }

    ///Returns a copy/clone of this name
    pub fn get_name(&self) -> String{
        self.name.clone()
    }
}
