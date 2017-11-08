use jakar_tree;

///Describes the Value bit of this tree
pub mod content;

///Describes the attributes the tree can have
pub mod attributes;

///Describes the jobs this tree can execute when updated
pub mod jobs;


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
