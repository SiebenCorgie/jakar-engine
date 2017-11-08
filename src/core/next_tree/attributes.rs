use cgmath::*;
use collision::Aabb3;

///A node can have this attributes
pub struct NodeAttributes {

    ///Transform of this node in local space
    pub transform: Decomposed<Vector3<f32>, Quaternion<f32>>,
    ///The bounds of this note, takes the `content` bound as well as the max and min values of
    ///all its children into consideration.
    bound: Aabb3<f32>,


    /// Can be turned off to disable shadow casting, usefull for many small objects
    pub cast_shadow: bool,
    /// Is used to determin at which point this object is rendered.
    /// There is the first pass for opaque objects, as wella s msked objects, and the second one for
    /// transparent ones.
    pub is_transparent: bool,
    /// If true the object won't be rendered if the engine is in gmae mode.
    pub hide_in_game: bool,
}
