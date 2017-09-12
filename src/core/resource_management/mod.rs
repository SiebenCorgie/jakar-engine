
///A high level material manager used for creating, managing and destroying the materials
pub mod material_manager;
///Handels all available meshes as well as the scenes created from an import with several meshes
pub mod mesh_manager;
///A high level asset manager which makes it easy to add and remove objects from/to the scene
///Graph. It also handles loading objects in a different thread and assiging materials from a material
///manager.
pub mod asset_manager;
///The scene manager manages all available scene, he is tightly packet with the mesh and light manager(todo)
pub mod scene_manager;
///Manages all available textues and gives out `Arc<Texture>` copys on request
pub mod texture_manager;


use render::pipeline_manager;
use render::uniform_manager;

use vulkano;

use std::sync::{Arc, Mutex};

///A small struct containing all common types which need to be send between functions and thread
///often
pub struct ManagerAndRenderInfo {
    ///The current pipeline manager
    pub pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,
    ///The current uniform manager
    pub uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
    ///The current device used for rendering
    pub device: Arc<vulkano::device::Device>,
    ///The currently used queues
    pub queue: Arc<vulkano::device::Queue>,
    ///The current texture manager
    pub texture_manager: Arc<Mutex<texture_manager::TextureManager>>,
    ///The current material manager
    pub material_manager: Arc<Mutex<material_manager::MaterialManager>>,
    ///The current mesh manager
    pub mesh_manager: Arc<Mutex<mesh_manager::MeshManager>>,
    ///The current scene manager
    pub scene_manager: Arc<Mutex<scene_manager::SceneManager>>
}
