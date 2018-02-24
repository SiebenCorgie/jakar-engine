///Collects all shader implementations. This might be moved to a sequenz which can be load by a config file
pub mod shaders;

///Collects the input structs generated while analysing the spriV code.
pub mod shader_inputs;

///The default pbr shader set used for drawing in a realistic scene
pub mod set_pbr;

///The current default post progress set
pub mod set_post_progress;

///Resolves a Hdr fragment into a ldr image and an hdr image
pub mod set_resolve;

///A set of simple shader, used to draw wireframes in 3D space
pub mod set_wireframe;

use vulkano::device::Device;

use render::shader_set::ToPipeline;
use render::shader_set::ShaderLibrary;

use std::sync::Arc;

///Indentifies the shader sets.
/// Contains:
/// - "Pbr"
/// - "Wireframe"
/// - "PpExposure"
/// - "PpResolveHdr"

#[derive(PartialEq)]
pub struct DefaultShaderSets {
}

impl DefaultShaderSets{
    pub fn new() -> Self{
        DefaultShaderSets{}
    }
}

impl ShaderLibrary for DefaultShaderSets{
    ///Returns true if the library has a shader set with this name
    fn has_shader_set(&self, name: String) -> bool{
        match name.as_ref(){
            "Pbr" => true,
            "Wireframe" => true,
            "PpExposure" => true,
            "PpResolveHdr" => true,
            _ => false,
        }
    }
    ///Returns the shader set with this name
    fn get_shader_set(&self, name: String, device: Arc<Device>) -> Option<Arc<ToPipeline + Send + Sync>>{
        match name.as_ref(){
            "Pbr" => return Some(Arc::new(set_pbr::PbrSet::load(device))),
            "Wireframe" => return Some(Arc::new(set_wireframe::SetWireframe::load(device))),
            "PpExposure" => return Some(Arc::new(set_post_progress::PostProgressSet::load(device))),
            "PpResolveHdr" => return Some(Arc::new(set_resolve::ResolveSet::load(device))),
            _ => {}, //will return none
        }
        println!("Could not find shader set: {}", name);
        None
    }

}
