use jakar_tree;

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::BTreeMap;

use core::resources::*;


///Describes the Value bit of this tree
pub mod content;

///Describes the attributes the tree can have
pub mod attributes;
use jakar_tree::node::Attribute;
///Describes the jobs this tree can execute when updated
pub mod jobs;

use cgmath::*;
use collision::*;

///The comparer type used to comapre a SceneTree to attribtues.
///You can use this for instance to get every node which is transparent.
#[derive(Clone)]
pub struct SceneComparer{
        ///Some if the transform component should be compared
        pub transform: Option<Decomposed<Vector3<f32>, Quaternion<f32>>>,
        ///Some if the bound component should be compared
        pub bound: Option<Aabb3<f32>>,
        ///Some if the value bound should compared with self
        pub value_bound: Option<Aabb3<f32>>,
        ///Some if the size should be comapred. However, be aware that a same size doesn't necessarly mean
        /// that you got the same bound extends.
        pub size: Option<f32>,
        ///Some if the cast_shadow component should be compared
        pub cast_shadow: Option<bool>,
        ///Some if the is_transparent component should be compared
        pub is_transparent: Option<bool>,
        ///Some if the hide_in_game component should be compared
        pub hide_in_game: Option<bool>,
        ///Some if the is_emessive component should be compared. Good to get all objects which
        /// can produce light.
        pub is_emessive: Option<bool>,
}

impl SceneComparer{
    ///Creates a new comparer with only `None`s
    pub fn new() -> Self{
        SceneComparer{
            transform: None,
            bound: None,
            value_bound: None,
            size: None,
            cast_shadow: None,
            is_transparent: None,
            hide_in_game: None,
            is_emessive: None,
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

    ///Adds a Some(value bound)
    pub fn with_value_bound(mut self, bound: Aabb3<f32>) -> Self{
        self.value_bound = Some(bound);
        self
    }

    ///Adds a Some(size)
    pub fn with_size(mut self, size: f32) -> Self{
        self.size = Some(size);
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

    ///Sets to "object emmits light"
    pub fn with_is_emessive(mut self) -> Self{
        self.is_emessive = Some(true);
        self
    }

    ///Sets to "object emmits no light"
    pub fn without_is_emessive(mut self) -> Self{
        self.is_emessive = Some(false);
        self
    }

}


///The trait for special engine funtions
pub trait SceneTree<
T: jakar_tree::node::NodeContent + Clone,
J: Clone, A: jakar_tree::node::Attribute<J> + Clone
> {
    ///Returns all meshes in the tree
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_meshes(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all meshse in the view frustum of `camera`
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_meshes_in_frustum(&self, camera: &camera::DefaultCamera, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all point lights in the tree
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_point_lights(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all directional lights
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_directional_lights(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all spot lights
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_spot_lights(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all empts
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_emptys(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Returns all cameras
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_cameras(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;
    ///Rebuilds the bounds for the whole tree
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn rebuild_bounds(&mut self);
}

impl SceneTree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>
    for jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>{
    ///Returns all meshes in this tree.
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    ///TODO actually implement
    fn get_all_meshes(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        let mut return_vec = Vec::new();
        for (_, child) in self.children.iter(){
            return_vec.append(&mut child.get_all_meshes(sorting)); //append all children
        }
        //first of all test if self has the right attributes, if not we can already return the child
        // vector
        match sorting{
            &Some(ref comparer) => {
                //early return if self doesnt match the sorting
                if !self.attributes.compare(comparer){
                    return return_vec;
                }
            },
            &None =>  {}, //all is nice, add the mesh
        }



        //check self
        match self.value{
            content::ContentType::Mesh(ref mesh) => {
                let node_copy = jakar_tree::node::Node{
                    name: self.name.clone(),
                    value: content::ContentType::Mesh(mesh.clone()),
                    children: BTreeMap::new(),
                    jobs: Vec::new(),
                    attributes: self.attributes.clone(),
                };
                return_vec.push(node_copy);
            },
            _ => {}, //self is no mesh only going doing nothing
        }

        return_vec
    }
    ///Returns all meshse in the view frustum of `camera`
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_meshes_in_frustum(&self, camera: &camera::DefaultCamera, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        println!("Getting in frustum is currently not supported", );
        Vec::new()
    }
    ///Returns all point lights in the tree
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_point_lights(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        let mut return_vec = Vec::new();
        for (_, child) in self.children.iter(){
            return_vec.append(&mut child.get_all_point_lights(&sorting));
        }

        //first of all test if self has the right attributes, if not we can already return the child
        // vector
        match sorting{
            &Some(ref comparer) => {
                //early return if self doesnt match the sorting
                if !self.attributes.compare(comparer){
                    return return_vec;
                }
            },
            &None =>  {}, //all is nice, add the mesh
        }

        //check self
        match self.value{
            content::ContentType::PointLight(ref light) => {
                let node_copy = jakar_tree::node::Node{
                    name: self.name.clone(),
                    value: content::ContentType::PointLight(light.clone()),
                    children: BTreeMap::new(),
                    jobs: Vec::new(),
                    attributes: self.attributes.clone(),
                };
                return_vec.push(node_copy);
            },
            _ => {}, //self is no mesh only going doing nothing
        }

        return_vec
    }
    ///Returns all directional lights
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_directional_lights(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        let mut return_vec = Vec::new();
        for (_, child) in self.children.iter(){
            return_vec.append(&mut child.get_all_directional_lights(sorting));
        }

        //first of all test if self has the right attributes, if not we can already return the child
        // vector
        match sorting{
            &Some(ref comparer) => {
                //early return if self doesnt match the sorting
                if !self.attributes.compare(comparer){
                    return return_vec;
                }
            },
            &None =>  {}, //all is nice, add the mesh
        }

        //check self
        match self.value{
            content::ContentType::DirectionalLight(ref light) => {
                let node_copy = jakar_tree::node::Node{
                    name: self.name.clone(),
                    value: content::ContentType::DirectionalLight(light.clone()),
                    children: BTreeMap::new(),
                    jobs: Vec::new(),
                    attributes: self.attributes.clone(),
                };
                return_vec.push(node_copy);
            },
            _ => {}, //self is no mesh only going doing nothing
        }

        return_vec
    }
    ///Returns all spot lights
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_spot_lights(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        let mut return_vec = Vec::new();
        for (_, child) in self.children.iter(){
            return_vec.append(&mut child.get_all_spot_lights(sorting));
        }

        //first of all test if self has the right attributes, if not we can already return the child
        // vector
        match sorting{
            &Some(ref comparer) => {
                //early return if self doesnt match the sorting
                if !self.attributes.compare(comparer){
                    return return_vec;
                }
            },
            &None =>  {}, //all is nice, add the mesh
        }

        //check self
        match self.value{
            content::ContentType::SpotLight(ref light) => {
                let node_copy = jakar_tree::node::Node{
                    name: self.name.clone(),
                    value: content::ContentType::SpotLight(light.clone()),
                    children: BTreeMap::new(),
                    jobs: Vec::new(),
                    attributes: self.attributes.clone(),
                };
                return_vec.push(node_copy);
            },
            _ => {}, //self is no mesh only going doing nothing
        }

        return_vec
    }
    ///Returns all empts
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_emptys(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        let mut return_vec = Vec::new();
        for (_, child) in self.children.iter(){
            return_vec.append(&mut child.get_all_emptys(sorting));
        }

        //first of all test if self has the right attributes, if not we can already return the child
        // vector
        match sorting{
            &Some(ref comparer) => {
                //early return if self doesnt match the sorting
                if !self.attributes.compare(comparer){
                    return return_vec;
                }
            },
            &None =>  {}, //all is nice, add the mesh
        }

        //check self
        match self.value{
            content::ContentType::Empty(ref empty) => {
                let node_copy = jakar_tree::node::Node{
                    name: self.name.clone(),
                    value: content::ContentType::Empty(empty.clone()),
                    children: BTreeMap::new(),
                    jobs: Vec::new(),
                    attributes: self.attributes.clone(),
                };
                return_vec.push(node_copy);
            },
            _ => {}, //self is no mesh only going doing nothing
        }

        return_vec
    }
    ///Returns all cameras
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_cameras(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        let mut return_vec = Vec::new();
        for (_, child) in self.children.iter(){
            return_vec.append(&mut child.get_all_cameras(sorting));
        }

        //first of all test if self has the right attributes, if not we can already return the child
        // vector
        match sorting{
            &Some(ref comparer) => {
                //early return if self doesnt match the sorting
                if !self.attributes.compare(comparer){
                    return return_vec;
                }
            },
            &None =>  {}, //all is nice, add the mesh
        }

        //check self
        match self.value{
            content::ContentType::Camera(ref camera) => {
                let node_copy = jakar_tree::node::Node{
                    name: self.name.clone(),
                    value: content::ContentType::Camera(camera.clone()),
                    children: BTreeMap::new(),
                    jobs: Vec::new(),
                    attributes: self.attributes.clone(),
                };
                return_vec.push(node_copy);
            },
            _ => {}, //self is no mesh only going doing nothing
        }

        return_vec
    }
    ///rebuilds the bounds for the whole tree
    fn rebuild_bounds(&mut self){
        //first of all rebuild the bounds for the children, then, based on the current
        // biggest and smallet values of the children, rebuild self's
        //node bound.
        for (_, child) in self.children.iter_mut(){
            child.rebuild_bounds();
        }

        //now get selfs min and max values in world space
        let mut mins = self.attributes.transform.transform_point(self.attributes.get_value_bound().min);
        let mut maxs = self.attributes.transform.transform_point(self.attributes.get_value_bound().max);

        for (_, child) in self.children.iter(){
            //get child min and max values
            let child_mins = child.attributes.get_bound().min;
            let child_maxs = child.attributes.get_bound().max;
            //check mins
            if child_mins.x < mins.x{
                mins.x = child_mins.x;
            }
            if child_mins.y < mins.y{
                mins.y = child_mins.y;
            }
            if child_mins.z < mins.z{
                mins.z = child_mins.z;
            }

            //check max
            if child_maxs.x > maxs.x{
                maxs.x = child_maxs.x;
            }
            if child_maxs.y > maxs.y{
                maxs.y = child_maxs.y;
            }
            if child_maxs.z > maxs.z{
                maxs.z = child_maxs.z;
            }
        }
        /*
        //if this node has no children, use the value bounds translated by the transform instead
        if self.children.len() == 0{
            mins = self.attributes.transform.transform_point(self.attributes.get_value_bound().min);
            maxs = self.attributes.transform.transform_point(self.attributes.get_value_bound().max);
        }
        */
        //finished the checks, update self
        self.attributes.bound = Aabb3::new(mins, maxs);
    }
}


impl SceneTree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>
 for jakar_tree::tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>{

    ///Returns all meshes in this tree.
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    ///TODO actually implement
    fn get_all_meshes(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        self.root_node.get_all_meshes(sorting)
    }
    ///Returns all meshse in the view frustum of `camera`
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_meshes_in_frustum(&self, camera: &camera::DefaultCamera, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        self.root_node.get_all_meshes_in_frustum(camera, sorting)
    }
    ///Returns all point lights in the tree
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_point_lights(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        self.root_node.get_all_point_lights(sorting)
    }
    ///Returns all directional lights
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_directional_lights(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        self.root_node.get_all_directional_lights(sorting)
    }
    ///Returns all spot lights
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_spot_lights(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        self.root_node.get_all_spot_lights(sorting)
    }
    ///Returns all empts
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_emptys(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        self.root_node.get_all_emptys(sorting)
    }
    ///Returns all cameras
    /// NOTE: Each node is copied from the tree into a stand alone node without any childern!
    fn get_all_cameras(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        self.root_node.get_all_cameras(sorting)
    }
    ///rebuilds the bounds for the whole tree
    fn rebuild_bounds(&mut self){
        self.root_node.rebuild_bounds()
    }
}

///unwraps the vector into a vector of meshes
pub trait SaveUnwrap{
    ///turns self into a vector of mutex guarded meshes
    fn into_meshes(self) -> Vec<Arc<Mutex<mesh::Mesh>>>;
    ///turns self into a vector of point lights
    fn into_point_light(self) -> Vec<light::LightPoint>;
    ///turns self into a vector of directional lights
    fn into_directional_light(self) -> Vec<light::LightDirectional>;
    ///turns self into a vector of spot lights
    fn into_spot_light(self) -> Vec<light::LightSpot>;
    ///turns self into a vector of emptys
    fn into_emptys(self) -> Vec<empty::Empty>;
    ///turns self into a vector of cameras
    fn into_cameras(self) -> Vec<camera::DefaultCamera>;
}

impl SaveUnwrap for Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
    ///turns self into a vector of mutex guarded meshes
    fn into_meshes(self) -> Vec<Arc<Mutex<mesh::Mesh>>>{
        let mut return_vector = Vec::new();
        for mesh in self.into_iter(){
            //test and push
            match mesh.value{
                content::ContentType::Mesh(mesh) => return_vector.push(mesh),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of point lights
    fn into_point_light(self) -> Vec<light::LightPoint>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::PointLight(light) => return_vector.push(light),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of point lights
    fn into_directional_light(self) -> Vec<light::LightDirectional>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::DirectionalLight(light) => return_vector.push(light),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of point lights
    fn into_spot_light(self) -> Vec<light::LightSpot>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::SpotLight(light) => return_vector.push(light),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of emptys
    fn into_emptys(self) -> Vec<empty::Empty>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::Empty(empty) => return_vector.push(empty),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of cameras
    fn into_cameras(self) -> Vec<camera::DefaultCamera>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::Camera(cam) => return_vector.push(cam),
                _ => {}, //do nothing
            }
        }

        return_vector
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
