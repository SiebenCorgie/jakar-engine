
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
