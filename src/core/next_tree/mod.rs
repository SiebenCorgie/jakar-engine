use jakar_tree;

use std::sync::Arc;
use std::sync::Mutex;
use core::resources::*;


///Describes the Value bit of this tree
pub mod content;

///Describes the attributes the tree can have
pub mod attributes;

///Describes the jobs this tree can execute when updated
pub mod jobs;

use cgmath::*;
use collision::*;

///The comparer type used to comapre a SceneTree to attribtues.
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

impl SceneComparer{
    ///Creates a new comparer with only `None`s
    pub fn new() -> Self{
        SceneComparer{
            transform: None,
            bound: None,
            cast_shadow: None,
            is_transparent: None,
            hide_in_game: None,
        }
    }

    ///Adds a `Some(transform)` component to the comparer
    pub fn with_transform(mut self, transform: Decomposed<Vector3<f32>, Quaternion<f32>>) -> Self{
        self.transform = Some(transform);
        self
    }

    ///Adds a `Some(bound)`
    pub fn with_bound(mut self, bound: Aabb3<f32>) -> Self{
        self.bound = Some(bound);
        self
    }

    ///sets shadow casting to Some(true)
    pub fn with_shadows(mut self) -> Self{
        self.cast_shadow = Some(true);
        self
    }

    ///sets shadow casting to Some(false)
    pub fn without_shadows(mut self) -> Self{
        self.cast_shadow = Some(false);
        self
    }

    ///adds transparency as parameter to Some(true)
    pub fn with_transparency(mut self) -> Self{
        self.is_transparent = Some(true);
        self
    }

    ///adds transparency as parameter to Some(false)
    pub fn without_transparency(mut self) -> Self{
        self.is_transparent = Some(false);
        self
    }

    ///Sets to "object is hidden in game"
    pub fn with_is_hidden(mut self) -> Self{
        self.hide_in_game = Some(true);
        self
    }

    ///Sets to "object is not hidden in game"
    pub fn without_is_hidden(mut self) -> Self{
        self.hide_in_game = Some(false);
        self
    }
}


///The trait for special engine funtions
pub trait SceneTree<
T: jakar_tree::node::NodeContent + Clone,
J: Clone, A: jakar_tree::node::Attribute<J> + Clone
> {
    ///Returns all meshes in the tree
    fn get_all_meshes(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all meshse in the view frustum of `camera`
    fn get_all_meshes_in_frustum(&self, camera: &camera::DefaultCamera, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all point lights in the tree
    fn get_all_point_lights(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all directional lights
    fn get_all_directional_lights(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all spot lights
    fn get_all_spot_lights(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all empts
    fn get_all_emptys(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all cameras
    fn get_all_cameras(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Rebuilds the bounds for the whole tree
    fn rebuild_bounds(&mut self);
}

impl<
T: jakar_tree::node::NodeContent + Clone, J: Clone, A: jakar_tree::node::Attribute<J> + Clone
> SceneTree<T, J, A> for jakar_tree::tree::Tree<T, J, A>{

    ///Returns all meshes in this tree.
    ///TODO actually implement
    fn get_all_meshes(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///Returns all meshse in the view frustum of `camera`
    fn get_all_meshes_in_frustum(&self, camera: &camera::DefaultCamera, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///Returns all point lights in the tree
    fn get_all_point_lights(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///Returns all directional lights
    fn get_all_directional_lights(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///Returns all spot lights
    fn get_all_spot_lights(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///Returns all empts
    fn get_all_emptys(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///Returns all cameras
    fn get_all_cameras(&self, sorting: Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///rebuilds the bounds for the whole tree
    fn rebuild_bounds(&mut self){
        println!("Rebuilding bounds currently not supported", );
    }
}

///unwraps the vector into a vector of meshes
pub trait SaveUnwrap{
    ///turns self into a vector of mutex guarded meshes
    fn into_meshes(&mut self) -> Vec<Arc<Mutex<mesh::Mesh>>>;
    ///turns self into a vector of point lights
    fn into_point_light(&mut self) -> Vec<light::LightPoint>;
    ///turns self into a vector of directional lights
    fn into_directional_light(&mut self) -> Vec<light::LightDirectional>;
    ///turns self into a vector of spot lights
    fn into_spot_light(&mut self) -> Vec<light::LightSpot>;
    ///turns self into a vector of emptys
    fn into_emptys(&mut self) -> Vec<empty::Empty>;
    ///turns self into a vector of cameras
    fn into_cameras(&mut self) -> Vec<camera::DefaultCamera>;
}

impl<
T: jakar_tree::node::NodeContent + Clone, J: Clone, A: jakar_tree::node::Attribute<J> + Clone
> SaveUnwrap for Vec<jakar_tree::node::Node<T, J, A>>{
    ///turns self into a vector of mutex guarded meshes
    fn into_meshes(&mut self) -> Vec<Arc<Mutex<mesh::Mesh>>>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///turns self into a vector of point lights
    fn into_point_light(&mut self) -> Vec<light::LightPoint>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///turns self into a vector of point lights
    fn into_directional_light(&mut self) -> Vec<light::LightDirectional>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///turns self into a vector of point lights
    fn into_spot_light(&mut self) -> Vec<light::LightSpot>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///turns self into a vector of emptys
    fn into_emptys(&mut self) -> Vec<empty::Empty>{
        println!("Getting is currently not supported", );
        Vec::new()
    }
    ///turns self into a vector of cameras
    fn into_cameras(&mut self) -> Vec<camera::DefaultCamera>{
        println!("Getting is currently not supported", );
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
