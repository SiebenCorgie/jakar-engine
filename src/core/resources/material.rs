use render::uniform_manager;
use render::pipeline;
use core::resources::texture;
//use render::shader_impls::pbr_fragment;
use render::shader::shader_inputs::pbr_texture_info;
use render::shader::shaders::shadow_fragment::ty::MaskedInfo;
use render::light_system;
use render::frame_system::FrameSystem;

use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::descriptor::descriptor_set::DescriptorSet;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano;

use cgmath::*;

use std::sync::{Mutex,Arc};

//=================================================================================================

///A Struct used for prototyping the usage flags of the textures
#[derive(Clone)]
pub struct TextureUsageFlags {
    pub albedo: bool,
    pub normal: bool,
    pub metal: bool,
    pub roughness: bool,
    pub occlusion: bool,
    pub emissive: bool,
    pub is_masked: bool,
}

impl TextureUsageFlags{
    ///Creates a new flag info where all textures are unsed
    pub fn new() -> Self{
        TextureUsageFlags{
            albedo: false,
            normal: false,
            metal: false,
            roughness: false,
            occlusion: false,
            emissive: false,
            is_masked: false,
        }
    }


    ///Creates with a set albedo status
    pub fn with_albedo(mut self) ->Self{
        self.albedo = true;
        self
    }

    ///Creates with a set normal status
    pub fn with_normal(mut self) ->Self{
        self.normal = true;
        self
    }

    ///Creates with a set metal status
    pub fn with_metal(mut self) ->Self{
        self.metal = true;
        self
    }

    ///Creates with a set roughness status
    pub fn with_roughness(mut self,) ->Self{
        self.roughness = true;
        self
    }

    ///Creates with a set occlusion status
    pub fn with_occlusion(mut self) ->Self{
        self.occlusion = true;
        self
    }

    ///Creates with a set emissive status
    pub fn with_emissive(mut self) ->Self{
        self.emissive = true;
        self
    }

    ///Creates with a set emissive status
    pub fn is_masked(mut self) ->Self{
        self.is_masked = true;
        self
    }

    pub fn to_shader_flags(self) -> pbr_texture_info::ty::TextureUsageInfo{
        pbr_texture_info::ty::TextureUsageInfo{
            b_albedo: {
                if self.albedo{
                    1
                }else{
                    0
                }
            },
            b_normal: {
                if self.normal{
                    1
                }else{
                    0
                }
            },
            b_metal: {
                if self.metal{
                    1
                }else{
                    0
                }
            },
            b_roughness: {
                if self.roughness{
                    1
                }else{
                    0
                }
            },
            b_occlusion: {
                if self.occlusion{
                    1
                }else{
                    0
                }
            },
            b_emissive: {
                if self.emissive{
                    1
                }else{
                    0
                }
            },
            b_is_masked: {
                if self.is_masked{
                    1
                }else{
                    0
                }
            }
        }
    }
}

//=================================================================================================

///A Struct defining the the material factors. They are used as Colors/factors if no textures
/// are present
#[derive(Clone)]
pub struct MaterialFactors{
    albedo_factor: [f32; 4],
    normal_factor: f32,
    emissive_factor: [f32; 3],
    max_emission: f32,
    metal_factor: f32,
    roughness_factor: f32,
    occlusion_factor: f32,
    alpha_cutoff: f32,


}

impl MaterialFactors{
    ///Creates a set of default factors
    pub fn new()-> Self{
        MaterialFactors{
            albedo_factor: [1.0; 4],
            //this needs to be set to just blue for not manipulating the rest
            normal_factor: 1.0,
            emissive_factor: [1.0; 3],
            max_emission: 1.0,
            metal_factor: 1.0,
            roughness_factor: 1.0,
            occlusion_factor: 1.0,
            alpha_cutoff: 0.5,
        }
    }


    ///Creates the Factor struct with a given albdeo factor
    #[inline]
    pub fn with_factor_albedo(mut self, factor: [f32; 4]) -> Self{
        self.albedo_factor = factor;
        self
    }

    ///Creates the Factor struct with a given normal factor
    #[inline]
    pub fn with_factor_normal(mut self, factor: f32) -> Self{
        self.normal_factor = factor;
        self
    }

    ///Creates the Factor struct with a given metal factor
    #[inline]
    pub fn with_factor_metal(mut self, factor: f32) -> Self{
        self.metal_factor = factor;
        self
    }

    ///Creates the Factor struct with a given roughness factor
    #[inline]
    pub fn with_factor_roughness(mut self, factor: f32) -> Self{
        self.roughness_factor = factor;
        self
    }

    ///Creates the Factor struct with a given occlusion factor
    #[inline]
    pub fn with_factor_occlusion(mut self, factor: f32) -> Self{
        self.occlusion_factor = factor;
        self
    }

    ///Creates the Factor struct with a given emissive factor
    #[inline]
    pub fn with_factor_emissive(mut self, factor: [f32; 3]) -> Self{
        self.emissive_factor = factor;
        self
    }

    ///Can be used to increase the maximuim emission factor. Good if you want to highligh something etc.
    #[inline]
    pub fn with_max_emmision(mut self, factor: f32) -> Self{
        self.max_emission = factor;
        self
    }

    ///controlles until which factor a masked material will be "see through"
    #[inline]
    pub fn with_alpha_cutoff(mut self, factor: f32) -> Self{
        self.alpha_cutoff = factor;
        self
    }



    pub fn to_shader_factors(&self) -> pbr_texture_info::ty::TextureFactors{
        pbr_texture_info::ty::TextureFactors{
            albedo_factor: self.albedo_factor,
            normal_factor: self.normal_factor,
            emissive_factor: self.emissive_factor,
            max_emission: self.max_emission,
            metal_factor: self.metal_factor,
            roughness_factor: self.roughness_factor,
            occlusion_factor: self.occlusion_factor,
            alpha_cutoff: self.alpha_cutoff,
        }
    }
}

//=================================================================================================

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
        let mut used_albedo = false;
        let mut used_normal = false;
        let mut used_emissive = false;
        let mut used_physical = false;
        let mut used_occlusion = false;

        match albedo{
            Some(_) => used_albedo = true,
             _=> {},
        }
        match normal{
            Some(_) => used_normal = true,
             _=> {},
        }
        match metallic_roughness{
            Some(_) => used_physical = true,
             _=> {},
        }
        match occlusion{
            Some(_) => used_occlusion = true,
             _=> {},
        }
        match emissive{
            Some(_) => used_emissive = true,
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
    #[inline]
    pub fn with_usage_flags(mut self, new_flags: TextureUsageFlags) -> Self{
        self.texture_usage_info = new_flags;
        self
    }

    ///can be used to set custom factors
    #[inline]
    pub fn with_factors(mut self, new_factors: MaterialFactors) -> Self{
        self.material_factors = new_factors;
        self
    }

    ///Can be called if the material is masked. All other UsageFlags should be set at creation time of the
    /// builder
    pub fn mat_is_masked(&mut self){
        self.texture_usage_info.is_masked = true;
    }

    ///returns true if this material is masked
    pub fn is_masked(&self) -> bool{
        self.texture_usage_info.is_masked
    }

    ///builds a material from the supplied textures and other info
    pub fn build(
        self,
        name: &str,
        pipeline: Arc<pipeline::Pipeline>,
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

        //Now get a teporary pipeline reference to create the first descriptorsets from
        let pipeline_ref = pipeline.get_pipeline_ref();


        //The TextureUsageFlags and Factor Info comes from the builder, we create a pool for
        //them...
        //Create a pool to allocate from
        let usage_info_pool = vulkano::buffer::cpu_pool::CpuBufferPool::<pbr_texture_info::ty::TextureUsageInfo>
                                   ::new(device.clone(), vulkano::buffer::BufferUsage::all());


        let material_factor_pool = vulkano::buffer::cpu_pool::CpuBufferPool::<pbr_texture_info::ty::TextureFactors>
                                   ::new(device.clone(), vulkano::buffer::BufferUsage::all());


        //Additionaly lock the uniformanager to get the first global information
        let uniform_manager_isnt = uniform_manager.clone();
        let mut uniform_manager_lck = uniform_manager_isnt.lock().expect("Failed to locj unfiorm_mng");

        //TODO add set 02 for material information
        //println!("STATUS: MATERIAL: Creating set 01 for the first time", );
        let ident_mat_4: Matrix4<f32> = Matrix4::identity();
        let set_01 = Arc::new(PersistentDescriptorSet::start(
                pipeline_ref.clone(), 0
            )
            .add_buffer((*uniform_manager_lck).get_subbuffer_data(ident_mat_4).clone()).expect("Failed to create descriptor set")

            .build().expect("failed to build first descriptor 01")
        );


        //Create the set 02
        //println!("STATUS: MATERIAL: Creating set 02 for the first time", );
        let set_02 = Arc::new(
            PersistentDescriptorSet::start(
                pipeline_ref.clone(), 1
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

        let usage_info_sub_buffer = {
            match usage_info_pool.next(self.texture_usage_info.clone().to_shader_flags()){
                Ok(k) => k,
                Err(e) => {
                    println!("{:?}", e);
                    panic!("failed to allocate new sub buffer!")
                },
            }
        };

        let material_factor_sub_buffer = {
            match material_factor_pool.next(self.material_factors.clone().to_shader_factors()){
                Ok(k) => k,
                Err(e) => {
                    println!("{:?}", e);
                    panic!("failed to allocate new sub buffer!")
                },
            }
        };

        //Create the Usage Flag descriptor
        let set_03 = Arc::new(PersistentDescriptorSet::start(
                pipeline_ref.clone(), 2
            )
            .add_buffer(usage_info_sub_buffer
            ).expect("Failed to create descriptor set")
            .add_buffer(material_factor_sub_buffer
            ).expect("failed to create the first material factor pool")
            .build().expect("failed to build first descriptor 03")
        );

        let set_01_pool = FixedSizeDescriptorSetsPool::new(pipeline_ref.clone(), 0);

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

            set_01_pool: set_01_pool,

            set_02: set_02,

            set_03: set_03,

            texture_usage_info: self.texture_usage_info.to_shader_flags(),
            usage_info_pool: usage_info_pool,

            material_factors: self.material_factors.to_shader_factors(),
            material_factor_pool: material_factor_pool,
        }
    }
}

//=================================================================================================

///Describes a standart material
///
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
    pipeline: Arc<pipeline::Pipeline>,
    ///A reference to the global uniform manager
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,

    //The set for the u_world information
    set_01_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,

    //A persistent material set which only needs to be alter if a texture changes
    set_02: Arc<DescriptorSet + Send + Sync>,

    //Usage flags of the different buffers, stored in a seperate set as well as material factors buffer
    set_03: Arc<DescriptorSet + Send + Sync>,
    //as shader usable struct
    texture_usage_info: pbr_texture_info::ty::TextureUsageInfo,
    usage_info_pool: vulkano::buffer::cpu_pool::CpuBufferPool<pbr_texture_info::ty::TextureUsageInfo>,

    material_factors: pbr_texture_info::ty::TextureFactors,
    material_factor_pool: vulkano::buffer::cpu_pool::CpuBufferPool<pbr_texture_info::ty::TextureFactors>,
}


impl Material {

    ///Returns the used uniform manager
    #[inline]
    pub fn get_uniform_manager(&self) -> Arc<Mutex<uniform_manager::UniformManager>>{
        self.uniform_manager.clone()
    }

    ///Adds a albedo texture to the material
    #[inline]
    pub fn set_albedo_texture(&mut self, albedo: Arc<texture::Texture>){
        self.t_albedo = albedo;
        self.texture_usage_info.b_albedo = 1;
    }

    ///Adds a normal Texture
    #[inline]
    pub fn set_normal_texture(&mut self, normal: Arc<texture::Texture>){
        self.t_normal = normal;
        self.texture_usage_info.b_normal = 1;
    }

    ///Adds a physical texture
    #[inline]
    pub fn set_metallic_roughness_texture(&mut self, physical: Arc<texture::Texture>){
        self.t_metallic_roughness = physical;
    }

    ///Adds a emissive texture
    #[inline]
    pub fn set_emissive_texture(&mut self, emissive: Arc<texture::Texture>){
        self.t_emissive = emissive;
        self.texture_usage_info.b_emissive = 1;
    }

    ///Overrwrites the old usage infor with the new ones.
    #[inline]
    pub fn set_texture_usage_info(&mut self, info: TextureUsageFlags){
        self.texture_usage_info = info.to_shader_flags();
    }

    ///Sets the material factors
    #[inline]
    pub fn set_material_factor_info(&mut self, info: MaterialFactors){
        self.material_factors = info.to_shader_factors();
    }

    ///Recreates set_02, set_03
    pub fn recreate_static_sets(&mut self){

        let pipeline_ref = self.pipeline.get_pipeline_ref();

        let set_02 = Arc::new(
            PersistentDescriptorSet::start(
                pipeline_ref.clone(), 1
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
                pipeline_ref.clone(), 2
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


    ///Returns the currently used vulkano-pipeline
    #[inline]
    pub fn get_vulkano_pipeline(&self) -> Arc<GraphicsPipelineAbstract + Send + Sync>{
        let pipeline_ref = self.pipeline.get_pipeline_ref();
        pipeline_ref.clone()
    }

    ///Returns the currently used pipeline
    #[inline]
    pub fn get_pipeline(&self) -> Arc<pipeline::Pipeline>{
        self.pipeline.clone()
    }

    ///Updates all sets tied to this material
    #[inline]
    pub fn update(&mut self){
        //println!("STATUS: MATERIAL: In material, updating now", );
        //check if this pipeline actually needs light, if not don't do anything
        //if self.pipeline.get_inputs().has_light{
        //    self.recreate_set_04();
        //}
        //if needed, update the static sets
    }

    ///Returns a subbuffer from the `usage_info_pool` to be used when adding a buffer to a set
    fn get_usage_info_subbuffer(&self) ->
     vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer<pbr_texture_info::ty::TextureUsageInfo,
     Arc<vulkano::memory::pool::StdMemoryPool>>
     {
        match self.usage_info_pool.next(self.texture_usage_info.clone()){
            Ok(k) => k,
            Err(e) => {
                println!("{:?}", e);
                panic!("failed to allocate new sub buffer!")
            },
        }
    }

    ///Returns a subbuffer from the material_factor_pool to be used with the 3rd set
    fn get_material_factor_subbuffer(&self) ->
    vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer<pbr_texture_info::ty::TextureFactors,
    Arc<vulkano::memory::pool::StdMemoryPool>>
    {
        match self.material_factor_pool.next(self.material_factors.clone()){
            Ok(k) => k,
            Err(e) => {
                println!("{:?}", e);
                panic!("failed to allocate new sub buffer!")
            },
        }
    }

    ///Returns the u_world_set generated from a model specific `transform_matrix` as well as the
    ///global view and projection matrix
    #[inline]
    pub fn get_set_01(&mut self, transform_matrix: Matrix4<f32>) -> Arc<DescriptorSet + Send + Sync>{
        //println!("STATUS: MATERIAL: Trying to lock uniform manager", );
        let mut uniform_manager_lck = self.uniform_manager.lock().expect("Failed to lock unfiorm_mng");
        //println!("STATUS: MATERIAL: Generation new set_01", );
        let new_set = self.set_01_pool.next()
            .add_buffer(uniform_manager_lck.get_subbuffer_data(transform_matrix))
            .expect("Failed to create descriptor set")
            .build()
            .expect("failed to build descriptor 01");

        Arc::new(new_set)
    }


    ///Returns the second set which holds the material textures
    #[inline]
    pub fn get_set_02(&self) -> Arc<DescriptorSet + Send + Sync>{
        self.set_02.clone()

    }

    ///Returns the 3rd descriptor set, responsible for the material specific settings
    #[inline]
    pub fn get_set_03(&self) -> Arc<DescriptorSet + Send + Sync>{
        self.set_03.clone()
    }

    ///Returns the 4th desciptor set responsible for the lighting information based on the current lights in the culling system
    #[inline]
    pub fn get_set_04(
        &self, compute_sys: &light_system::LightSystem, frame_system: &FrameSystem,
    ) -> Arc<DescriptorSet + Send + Sync>{

        //This has to be build based on the currently used light lists in the compute system.
        compute_sys.get_light_descriptorset(3, self.get_vulkano_pipeline(), frame_system) //for pbr materials this has to be the three
    }

    ///Returns the `MaskedInfo` for the shadows as well as the texture containing the alpha values
    /// of this material
    pub fn get_shadow_mask_info(&self) -> (MaskedInfo, Arc<texture::Texture>){
        let info = MaskedInfo{
            b_is_masked: self.texture_usage_info.b_is_masked,
            alpha_cut_off: self.material_factors.alpha_cutoff,
        };
        (info, self.t_albedo.clone())
    }

    ///Sets a new pipeline
    #[inline]
    pub fn set_pipeline(&mut self, new_pipe: Arc<pipeline::Pipeline>){
        self.pipeline = new_pipe;
    }

    ///Returns a copy/clone of this name
    #[inline]
    pub fn get_name(&self) -> String{
        self.name.clone()
    }
}

//=================================================================================================
