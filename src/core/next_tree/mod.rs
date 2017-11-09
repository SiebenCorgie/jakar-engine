use jakar_tree;

///Describes the Value bit of this tree
pub mod content;

///Describes the attributes the tree can have
pub mod attributes;

///Describes the jobs this tree can execute when updated
pub mod jobs;

use cgmath::*;
use collision::*;

///The comparer type used to comapre a SceneNode to attribtues.
///You can use this for instance to get every node which is transparent.
pub struct SceneComparer{
        ///Some if the transform component should be compared
        pub transform: Option<Decomposed<Vector3<f32>, Quaternion<f32>>>,
        ///Some if the bound component should be compared
        pub bound: Option<Aabb3<f32>>,


        ///Some if the cast_shadow component should be compared
        pub cast_shadow: Option<bool>,
        ///Some if the is_transparent component should be compared
        pub is_transparent: Option<bool>,
        ///Some if the hide_in_game component should be compared
        pub hide_in_game: Option<bool>,
}


///The trait for special engine funtions1
pub trait SceneNode<T: jakar_tree::node::NodeContent, J: Clone, A: jakar_tree::node::Attribute<J>> {
    fn get_all_meshes() -> Vec<jakar_tree::node::Node<T, J, A>>;
}

impl<T: jakar_tree::node::NodeContent, J: Clone, A: jakar_tree::node::Attribute<J>> SceneNode<T, J, A> for jakar_tree::tree::Tree<T, J, A>{
    ///Returns all meshes in this tree.
    ///TODO actually implement
    fn get_all_meshes() -> Vec<jakar_tree::node::Node<T, J, A>>{
        Vec::new()
    }
}

//TODO Custom impls on node for:
/*
rebuild_bounds
get_bound_min
get_bound_max
get_bound
get_all_spot_lights/meshes/cameras
get_meshes_in_frustum
get_meshes_in_volume
*/
