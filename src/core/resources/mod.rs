///Definition of a default camera, the momvement has to be controlled by a gameplay-controller
pub mod camera;
///Defines all possible light
///Spot, point and directional light so far
pub mod light;
///Defines a normal mesh along with its properties
pub mod mesh;
///An empty can be used if a node should not have any content
pub mod empty;
///Defines a material with all it's properties, NOTE: this might switch to a UE4 like
///node based approach in the future.
pub mod material;
///Defines a texture along with it's different settings, like mipmapping and tiling-mode
pub mod texture;
