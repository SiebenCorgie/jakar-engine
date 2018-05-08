use jakar_tree;

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::BTreeMap;

use core::resources::*;
use core::resources::camera::Camera;


///Describes the Value bit of this tree
pub mod content;
///Describes the attributes the tree can have
pub mod attributes;
use jakar_tree::node::Attribute;
use jakar_tree::node::Node;
///Describes the jobs this tree can execute when updated
pub mod jobs;

use cgmath::*;
use collision::*;

///The type of a typical jakar-tree node in this engine
pub type JakarNode = Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>;


///Can be set to specialize which type of content a node should have to be considered im the comparing
#[derive(Clone, PartialEq)]
pub struct ValueTypeBool {
    pub render_able: bool,
    pub mesh: bool,
    pub point_light: bool,
    pub directional_light: bool,
    pub spot_light: bool,
    pub empty: bool,
    pub camera: bool
}

impl ValueTypeBool {
    ///Returns with all bools set to true
    pub fn all() -> Self{
        ValueTypeBool {
            render_able: true,
            mesh: true,
            point_light: true,
            directional_light: true,
            spot_light: true,
            empty: true,
            camera: true
        }
    }

    ///Returns with all bool set to false
    pub fn none() -> Self{
        ValueTypeBool {
            render_able: false,
            mesh: false,
            point_light: false,
            directional_light: false,
            spot_light: false,
            empty: false,
            camera: false
        }
    }

    ///Returns `true` if self is parth of `other`
    pub fn is_part_of(&self, other: &ValueTypeBool) -> bool{
        if (self.render_able && !other.render_able) || (!self.render_able && other.render_able){
            return false;
        }
        if (self.mesh && !other.mesh) || (!self.mesh && other.mesh){
            return false;
        }
        if (self.point_light && !other.point_light) || (!self.point_light && other.point_light){
            return false;
        }
        if (self.spot_light && !other.spot_light) || (!self.spot_light && other.spot_light){
            return false;
        }
        if (self.directional_light && !other.directional_light) || (!self.directional_light && other.directional_light){
            return false;
        }
        if (self.empty && !other.empty) || (!self.empty && other.empty){
            return false;
        }
        if (self.camera && !other.camera) || (!self.camera && other.camera){
            return false;
        }
        //everything self has is also contained in other therefore return true
        true
    }

    pub fn with_render_able(mut self) -> Self{
        self.render_able = true;
        self
    }

    pub fn with_mesh(mut self) -> Self{
        self.mesh = true;
        self
    }

    pub fn with_point_light(mut self) -> Self{
        self.point_light = true;
        self
    }

    pub fn with_directional_light(mut self) -> Self{
        self.directional_light = true;
        self
    }

    pub fn with_spot_light(mut self) -> Self{
        self.spot_light = true;
        self
    }

    pub fn with_empty(mut self) -> Self{
        self.empty = true;
        self
    }

    pub fn with_camera(mut self) -> Self{
        self.camera = true;
        self
    }
}

///The comparer type used to comapre a SceneTree to attribtues.
///You can use this for instance to get every node which is transparent.
#[derive(Clone)]
pub struct SceneComparer{
        ///Some if the transform component should be compared
        pub transform: Option<Decomposed<Vector3<f32>, Quaternion<f32>>>,
        ///Some if the bound of the node should be in this bound
        pub bound: Option<Aabb3<f32>>,
        ///Some the node should be in this frustum
        pub frustum: Option<Frustum<f32>>,
        ///Some if the value bound of the node should be in this bound
        pub value_bound: Option<Aabb3<f32>>,
        ///Specifies if a node value should be a certain node type
        pub value_type: Option<ValueTypeBool>,
        ///Some if the cast_shadow component should be compared
        pub cast_shadow: Option<bool>,
        ///Some if the is_transparent component should be compared
        pub is_transparent: Option<bool>,
        ///Some if the hide_in_game component should be compared
        pub hide_in_game: Option<bool>,
        ///Some if the is_emessive component should be compared. Good to get all objects which
        /// can produce light.
        pub is_emessive: Option<bool>,
        /// If enabled it will not add any node where the screen coverage of the AABB is lower than
        /// the float which is supplied as the first argument
        pub distance_cull: Option<(f32, Matrix4<f32>)>,
}

impl SceneComparer{
    ///Creates a new comparer with only `None`s
    pub fn new() -> Self{
        SceneComparer{
            transform: None,
            bound: None,
            frustum: None,
            value_bound: None,
            value_type: None,
            cast_shadow: None,
            is_transparent: None,
            hide_in_game: None,
            is_emessive: None,
            distance_cull: None,
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

    pub fn with_frustum(mut self, frustum: Frustum<f32>) ->Self{
        self.frustum = Some(frustum);
        self
    }

    ///Adds a Some(value bound)
    pub fn with_value_bound(mut self, bound: Aabb3<f32>) -> Self{
        self.value_bound = Some(bound);
        self
    }

    ///Adds a Some(value_type) where the type of the node can be specified
    pub fn with_value_type(mut self, value_type: ValueTypeBool) -> Self{
        self.value_type = Some(value_type);
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

    ///Culls the obejct based on a bias value, see the struct documentation for more information.
    pub fn with_cull_distance(mut self, bias: f32, view_projection_matrix: Matrix4<f32>) -> Self{
        self.distance_cull = Some((bias, view_projection_matrix));
        self
    }
}


///The trait for special engine funtions
pub trait SceneTree<
T: jakar_tree::node::NodeContent + Clone,
J: Clone, A: jakar_tree::node::Attribute<J> + Clone
> {
    ///Returns a list of names for every node that fulfills the `SceneComparer`,
    /// can be used to get each of them by name and add a job or access them in any other way.
    fn get_all_names(&self, sorting: &Option<SceneComparer>) -> Vec<String>;
    ///Returns all nodes in the tree that fulfill the `SceneComparer`.
    /// # Usage:
    /// 1. NOTE If you want for instance all meshes, just specifie `with_meshes` for a `ValueTypeBool`
    /// in the `SceneComparer`s `value_type`. You can then use the `SaveUnwrap` trait to unpack the
    /// vector of nodes into a vector of meshes. TODO provide a code sample.
    /// 2. NOTE: Each node is copied from the tree into a stand alone node without any childern!
    ///    The `SceneJobs` are also reseted to none.
    fn copy_all_nodes(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<T, J, A>>;

    ///Rebuilds the bounds for the whole tree
    fn rebuild_bounds(&mut self);

}

impl SceneTree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>
    for jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>{

    fn get_all_names(&self, sorting: &Option<SceneComparer>) -> Vec<String>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        let mut return_vec = Vec::new();
        for (_, child) in self.children.iter(){
            return_vec.append(&mut child.get_all_names(sorting)); //append all children
        }
        //first of all test if self has the right attributes, if not we can already return the child
        // vector
        match sorting{
            &Some(ref comparer) => {
                //early return if self doesnt match the sorting
                if !self.attributes.compare(comparer){
                    return return_vec;
                }
                //NOTE since the scene attributes don't know about the node value we havbe to compare
                //them manually here. Not nice but it works.
                //Check if the value type is in the scope we are searching for
                match comparer.value_type{
                    Some(ref val_ty) => {
                        //value type, checks if the current value is within the parameters.
                        let mut tmp_bool = ValueTypeBool::none();
                        match self.value{
                            content::ContentType::Renderable(_) => tmp_bool.render_able = true,
                            content::ContentType::Mesh(_) => tmp_bool.mesh = true,
                            content::ContentType::PointLight(_) => tmp_bool.point_light = true,
                            content::ContentType::DirectionalLight(_) => tmp_bool.directional_light = true,
                            content::ContentType::SpotLight(_) => tmp_bool.spot_light = true,
                            content::ContentType::Empty(_) => tmp_bool.empty = true,
                            content::ContentType::Camera(_) => tmp_bool.camera = true,
                        }

                        if tmp_bool.is_part_of(&val_ty) == false{
                            return return_vec; //We are not part of the sorting return early :/
                        }
                    },
                    None => {}, //all right not checking
                }

            },
            &None =>  {}, //all is nice, add
        }
        //Passed the test, lets add our own name
        return_vec.push(self.name.clone());
        return_vec
    }

    fn copy_all_nodes(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        let mut return_vec = Vec::new();
        for (_, child) in self.children.iter(){
            return_vec.append(&mut child.copy_all_nodes(sorting)); //append all children
        }
        //first of all test if self has the right attributes, if not we can already return the child
        // vector
        match sorting{
            &Some(ref comparer) => {
                //early return if self doesnt match the sorting
                if !self.attributes.compare(comparer){
                    return return_vec;
                }
                //NOTE since the scene attributes don't know about the node value we havbe to compare
                //them manually here. Not nice but it works.
                //Check if the value type is in the scope we are searching for
                match comparer.value_type{
                    Some(ref val_ty) => {
                        //value type, checks if the current value is within the parameters.
                        let mut tmp_bool = ValueTypeBool::none();
                        match self.value{
                            content::ContentType::Renderable(_) => tmp_bool.render_able = true,
                            content::ContentType::Mesh(_) => tmp_bool.mesh = true,
                            content::ContentType::PointLight(_) => tmp_bool.point_light = true,
                            content::ContentType::DirectionalLight(_) => tmp_bool.directional_light = true,
                            content::ContentType::SpotLight(_) => tmp_bool.spot_light = true,
                            content::ContentType::Empty(_) => tmp_bool.empty = true,
                            content::ContentType::Camera(_) => tmp_bool.camera = true,
                        }

                        if tmp_bool.is_part_of(&val_ty) == false{
                            return return_vec; //We are not part of the sorting return early :/
                        }
                    },
                    None => {}, //all right not checking
                }
            },
            &None =>  {}, //all is nice, add the mesh
        }

        //If self passed the ckeck for the attrributes, copy the current node and return
        let node_copy = jakar_tree::node::Node{
            name: self.name.clone(),
            value: self.value.clone(),
            children: BTreeMap::new(),
            jobs: Vec::new(),
            attributes: self.attributes.clone(),
            tick_closure: self.tick_closure.clone(),
        };
        return_vec.push(node_copy);

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
        //Calculate new mins and maxs value from the object bounds
        let object_bound = self.value.get_bound();
        let points = object_bound.to_corners();

        //Transform the points to worldspace
        let mut transformed_points = Vec::new();
        for point in points.iter(){
            //transform the point in worldspace
            transformed_points.push(
                self.attributes.transform.transform_point(point.clone())
            );
        }
        let (mut mins, mut maxs) = get_min_max(transformed_points);
        //Now its time to overwrite the value bounds for the new transformation, after this we'll
        //test out new bounds agains the children and create a new node extend which is used for
        //hierachy sorting etc.
        self.attributes.value_bound = Aabb3::new(mins.clone(), maxs.clone());
        //Finally update the draw distance
        self.attributes.max_draw_distance = get_max_aabb_len(&self.attributes.value_bound);


        //now get selfs min and max values in world space build by the object bound transformed by world space
        for (_, child) in self.children.iter(){
            //get child min and max values
            let child_mins = child.attributes.bound.min;
            let child_maxs = child.attributes.bound.max;
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

        //finished the checks, update self
        self.attributes.bound = Aabb3::new(mins, maxs);
    }


}


impl SceneTree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>
 for jakar_tree::tree::Tree<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>{


     ///Returns a list of all mesh names, can be used to get each of them by name and add a job.
     fn get_all_names(&self, sorting: &Option<SceneComparer>) -> Vec<String>{
         self.root_node.get_all_names(sorting)
     }

    fn copy_all_nodes(&self, sorting: &Option<SceneComparer>) -> Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
        //going recrusivly through each child (from the bottom), with each adding up to the whole.
        self.root_node.copy_all_nodes(sorting)
    }

    ///rebuilds the bounds for the whole tree
    fn rebuild_bounds(&mut self){
        self.root_node.rebuild_bounds()
    }
}

///unwraps the vector into a vector of meshes
pub trait SaveUnwrap{
    ///turns self into a vector of mutex guarded meshes
    fn into_meshes(&self) -> Vec<Arc<Mutex<mesh::Mesh>>>;
    ///turns self into a vector of point lights
    fn into_point_light(&self) -> Vec<light::LightPoint>;
    ///turns self into a vector of directional lights
    fn into_directional_light(&self) -> Vec<light::LightDirectional>;
    ///turns self into a vector of spot lights
    fn into_spot_light(&self) -> Vec<light::LightSpot>;
    ///turns self into a vector of emptys
    fn into_emptys(&self) -> Vec<empty::Empty>;
    ///turns self into a vector of cameras
    fn into_cameras(&self) -> Vec<camera::DefaultCamera>;
}

impl SaveUnwrap for Vec<jakar_tree::node::Node<content::ContentType, jobs::SceneJobs, attributes::NodeAttributes>>{
    ///turns self into a vector of mutex guarded meshes
    fn into_meshes(&self) -> Vec<Arc<Mutex<mesh::Mesh>>>{
        let mut return_vector = Vec::new();
        for mesh in self.into_iter(){
            //test and push
            match mesh.value{
                content::ContentType::Mesh(ref mesh) => return_vector.push(mesh.clone()),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of point lights
    fn into_point_light(&self) -> Vec<light::LightPoint>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::PointLight(ref light) => return_vector.push(light.clone()),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of point lights
    fn into_directional_light(&self) -> Vec<light::LightDirectional>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::DirectionalLight(ref light) => return_vector.push(light.clone()),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of point lights
    fn into_spot_light(&self) -> Vec<light::LightSpot>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::SpotLight(ref light) => return_vector.push(light.clone()),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of emptys
    fn into_emptys(&self) -> Vec<empty::Empty>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::Empty(ref empty) => return_vector.push(empty.clone()),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
    ///turns self into a vector of cameras
    fn into_cameras(&self) -> Vec<camera::DefaultCamera>{
        let mut return_vector = Vec::new();

        for node in self.into_iter(){
            //test and push
            match node.value{
                content::ContentType::Camera(ref cam) => return_vector.push(cam.clone()),
                _ => {}, //do nothing
            }
        }

        return_vector
    }
}

///Calculates the maximal values and minimal values for a set of 3D points. Returns `(min, max)`
/// as points.
/// There might be a better place for this function, but I only use it here so...
pub fn get_min_max(points: Vec<Point3<f32>>) -> (Point3<f32>, Point3<f32>){
    let mut mins = points[0];
    let mut maxs = points[0];

    //Go through the values and comapre each axis with the current determined mins and maxs
    for point in points.iter(){
        //check mins
        if point.x < mins.x{
            mins.x = point.x;
        }
        if point.y < mins.y{
            mins.y = point.y;
        }
        if point.z < mins.z{
            mins.z = point.z;
        }

        //check max
        if point.x > maxs.x{
            maxs.x = point.x;
        }
        if point.y > maxs.y{
            maxs.y = point.y;
        }
        if point.z > maxs.z{
            maxs.z = point.z;
        }
    }
    (mins, maxs)
}

///Computes the max length between one of the three coordinates x,y,z of a bound.
pub fn get_max_aabb_len(aabb: &Aabb3<f32>) -> f32{
    //first, get the min and maxes
    let mut length = aabb.max.x - aabb.min.x;
    if (aabb.max.y - aabb.min.y) > length { length = aabb.max.y - aabb.min.y; }
    if (aabb.max.z - aabb.min.z) > length { length = aabb.max.z - aabb.min.z; }

    length
}

///Projects each point in the list into the space of the matrix supplied.
pub fn project_points(points: &mut Vec<Point3<f32>>, matrix: &Matrix4<f32>){
    for point in points.iter_mut(){
        let mut tmp_point = matrix * point.to_vec().extend(1.0);
        tmp_point = tmp_point / tmp_point.w;
        *point = Point3::<f32>::from_vec(tmp_point.truncate());

    }
}

//takes a set of points and computes the maximum x,y distance possible between points. Used to
// compute the LOD / culling of objects
pub fn get_max_xy_len(points: &Vec<Point3<f32>>) -> f32{
    if points.len() == 0{
        return 0.0;
    }
    let mut max = Vector2::new(points[0].x, points[0].y);
    let mut min = Vector2::new(points[0].x, points[0].y);

    for poi in points.iter(){
        if poi.x > max.x{
            max.x = poi.x;
        }
        if poi.x < min.x{
            min.x = poi.x;
        }
        if poi.y > max.y{
            max.y = poi.y;
        }
        if poi.y < min.y{
            min.y = poi.y;
        }
    }

    //since we got the max values, gonna find the magnitude between them
    let dist_vec = max - min;
    return dist_vec.magnitude();
}
