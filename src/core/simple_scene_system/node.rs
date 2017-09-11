
use std::sync::{Arc, Mutex};

use cgmath;
use cgmath::Transform;
use cgmath::*;
use collision::*;
//use core::AABB3Intersection;
//use collision::Discrete;
use core::ReturnBoundInfo;

use rt_error;
use core;
use core::resources::mesh;
use core::resources::light;
use core::resources::empty;
use core::resources::camera;
use core::resources::camera::Camera;

//used for sorting and searching the children
use std::collections::BTreeMap;

///All possible types of content a Node can hold.
///Changed in order to apply a new type
#[derive(Clone)]
pub enum ContentType {
    Renderable(RenderableContent),
    Light(LightsContent),
    Other(OtherContent),
}
///All renderable types
#[derive(Clone)]
pub enum RenderableContent {
    Mesh(Arc<Mutex<mesh::Mesh>>),
}
///All lights
#[derive(Clone)]
pub enum LightsContent {
    PointLight(light::LightPoint),
    DirectionalLight(light::LightDirectional),
    SpotLight(light::LightSpot),
}
///All Other components
#[derive(Clone)]
pub enum OtherContent {
    Empty(empty::Empty),
    Camera(camera::DefaultCamera),
}

///Some implementations to make the programmers life easier
impl ContentType{
    ///Returns the name of this node
    pub fn get_name(&self) -> String{
        match self{
            &ContentType::Renderable(ref c) =>{
                match c{
                    &RenderableContent::Mesh(ref m) => {
                        let mesh_lock = m.lock().expect("failed to lock mesh");
                        (*mesh_lock).name.clone()
                    },
                }
            },
            &ContentType::Light(ref c) => {
                match c {
                    &LightsContent::PointLight(ref l) => {
                        l.name.clone()
                    },
                    &LightsContent::DirectionalLight(ref l) => {
                        l.name.clone()
                    },
                    &LightsContent::SpotLight(ref l) => {
                        l.name.clone()
                    },
                }
            },
            &ContentType::Other(ref c) => {
                match c {
                    &OtherContent::Empty(ref e) => {
                        e.name.clone()
                    },
                    &OtherContent::Camera(ref c) =>{
                        //c.name.clone() TODO add a camera name
                        String::from("CameraName")
                    }
                }
            }
        }
    }

    ///Returns the bound of this object
    pub fn get_bound(&self) -> Aabb3<f32>{
        match self{
            &ContentType::Renderable(ref c) =>{
                match c{
                    &RenderableContent::Mesh(ref m) => {
                        let mesh_lock = m.lock().expect("failed to lock mesh");
                        (*mesh_lock).get_bound()
                    },
                }
            },
            &ContentType::Light(ref c) => {
                match c {
                    &LightsContent::PointLight(ref l) => {
                        l.get_bound()
                    },
                    &LightsContent::DirectionalLight(ref l) => {
                        l.get_bound()
                    },
                    &LightsContent::SpotLight(ref l) => {
                        l.get_bound()
                    },
                }
            },
            &ContentType::Other(ref c) => {
                match c {
                    &OtherContent::Empty(ref e) => {
                        e.get_bound()
                    },
                    &OtherContent::Camera(ref c) =>{
                        Aabb3::new(
                            Point3::new(-1.0, -1.0, -1.0),
                            Point3::new(1.0, 1.0, 1.0)
                        )
                    }
                }
            }
        }
    }
}

///The normal Node of this Scene Tree
///
/// *Why a BTreeMap and no HashMap?*
/// I decided to use a BTreeMap of Structs where the name is in the struct and the tree mainly because of
/// performance reasons. With small datasets (5-100 entries) the BTreeMap is faster and provides
/// some comfort (you can store the name as a String as key value). However, if you have bigger
/// datasets (over 1,000,000) the HashMap is faster, as specially in `--release` mode.
/// However, this should not be relevant to this node tree because it should mostly consist of mid
// sized BTreeMaps
#[derive(Clone)]
pub struct GenericNode {

    //children: Vec<GenericNode>,
    children: BTreeMap<String, GenericNode>,
    ///There is a difference between a `Node`'s name and its `content` name
    pub name: String,
    ///And ID which needs to be unique TODO: Implement
    pub id: u32,
    ///Transform of this node in local space
    pub transform: Decomposed<Vector3<f32>, Quaternion<f32>>,
    ///The bounds of this note, takes the own `content` bound as well as the max and min values of
    ///all its children into consideration
    bound: Aabb3<f32>,
    ///The content is a contaier from the `ContentTypes` type which can hold any implemented
    ///Enum value
    content: ContentType,
}

///Implementation of the Node trait for Generic node
impl GenericNode{
    ///Creates a new, empty node
    pub fn new_empty(name: &str)-> Self{

        let tmp_bound = Aabb3::new(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));

        GenericNode{
            children: BTreeMap::new(),
            name: String::from(name),
            id: 1,
            transform: cgmath::Transform::one(),

            bound: tmp_bound,

            content: ContentType::Other(OtherContent::Empty(empty::Empty::new(name.clone()))),
        }
    }

    ///Should return an node
    pub fn new(content: ContentType)->Self{

        //Create variables needed to fill the final struct but change them depending on the match
        let mut name = content.get_name();
        let mut bound = content.get_bound();

        GenericNode{
            children: BTreeMap::new(),
            name: String::from(name),
            id: 1,
            transform: cgmath::Transform::one(),

            bound: bound,

            content: content,
        }
    }

    ///should release a node from memory
    pub fn release(&mut self, name: &str){
        match self.children.remove(&String::from(name)){
            //if `name` was in self.children return, else search in children
            Some(_) => return,
            None => {
                for (name, child) in self.children.iter_mut(){
                    child.release(name);
                }
            }
        }
    }

    ///Destroy this node and all its children
    pub fn destroy(&mut self){
        //First delete all children
        for (_,i) in self.children.iter_mut(){
            i.destroy();
        }
        //then self
        drop(self);
    }

    ///Adds a child node to this node
    pub fn add_child(&mut self, child: ContentType){
        //create the new node from the type
        let tmp_child = GenericNode::new(child);
        //and add it
        self.children.insert(tmp_child.name.clone(), tmp_child);
    }

    ///Adds a already prepared node, good for merging different trees
    pub fn add_node(&mut self, node: GenericNode){
        //Add it based on its own name
        self.children.insert(node.name.clone(), node);
    }

    ///Adds a `node_to_add` as a child to a node with `node_name` as name
    ///good merging node trees at a specific point
    pub fn add_node_at_sub_node(&mut self, node_name: &str,  node_to_add: GenericNode){
        let node = self.get_node(node_name);
        match node{
            None => rt_error("NODE: ", "Could not find subnode while trying to add"),
            Some(nd)=> nd.add_node(node_to_add),
        }

    }

    ///Returns a node with this name (the name of a node is pulled from the name of its content)
    pub fn get_node(&mut self, node_name: &str)-> Option<&mut Self>{

        let mut tmp_return: Option<&mut Self> = None;

        if self.name == String::from(node_name){
            return Some(self);
        }

        match tmp_return{
            //if something was found return it
            Some(item) => return Some(item),
            //else search in childrens children
            None=>{
                for (_,i) in self.children.iter_mut(){
                    match tmp_return{
                        None=> tmp_return = i.get_node(node_name),
                        Some(value)=> return Some(value),
                    }
                }
            }
        }
        //if the function comes here tmp_return will be `None`
        tmp_return
    }

    ///Returns the transform matrix
    pub fn get_transform_matrix(&self) -> Matrix4<f32>{
    Matrix4::from(self.transform)

    }

    ///Sets the transform of this node without changing its children
    pub fn set_transform_single(&mut self, new_transform: Decomposed<Vector3<f32>, Quaternion<f32>>){
        self.transform = new_transform;
    }

    ///Translates this node by `translation` and all its children
    pub fn translate(&mut self, translation: Vector3<f32>){
        //for self
        self.transform.disp = self.transform.disp + translation;

        //for all children
        for (_, child) in self.children.iter_mut(){
            child.translate(translation);
        }
    }

    ///Sets the location to `location` and changes the location of all its children as well
    pub fn set_location(&mut self, location: Vector3<f32>){
        //get the difference of the current and the new position
        let difference = location - self.transform.disp;

        //Set it for self
        self.translate(difference);
    }

    ///Rotates this node and all of its child by `rotation` around `point`
    pub fn rotate_around_point(&mut self, rotation: Vector3<f32>, point: Vector3<f32>){

        //FIXME reimplemt from https://gamedev.stackexchange.com/questions/16719/what-is-the-correct-order-to-multiply-scale-rotation-and-translation-matrices-f
        //move to point
        //create a rotation Quaternion from the angles in rotation.xyz
        let q_rotation = Quaternion::from(Euler {
            x: Deg(rotation.x),
            y: Deg(rotation.y),
            z: Deg(rotation.z),
        });

        self.transform.disp -= point;
        //do rotation
        self.transform.rot = self.transform.rot * q_rotation;
        //self.transform.rot = q_rotation.rotate_vector(self.transform.rot);
        self.transform.disp = q_rotation.rotate_vector(self.transform.disp);
        //move back to origin
        self.transform.disp += point;

        //now do the same for all childs
        for (_, child) in self.children.iter_mut(){
            child.rotate_around_point(rotation, point);
        }

    }

    ///Rotates this note and its children by `rotation`
    pub fn rotate(&mut self, rotation: Vector3<f32>){
        let q_rotation = Quaternion::from(Euler {
            x: Deg(rotation.x),
            y: Deg(rotation.y),
            z: Deg(rotation.z),
        });

        self.transform.rot = self.transform.rot * q_rotation;

        for (_, child) in self.children.iter_mut(){
            child.rotate_around_point(rotation, self.transform.disp);
        }

    }

    ///Returns a mesh from childs with this name
    pub fn get_mesh(&mut self, name: &str)-> Option<Arc<Mutex<core::resources::mesh::Mesh>>>{
        let mut result_value: Option<Arc<Mutex<core::resources::mesh::Mesh>>> = None;

        //match self
        match self.content{
            ContentType::Renderable(RenderableContent::Mesh(ref m)) => {
                let mesh_lock = m.lock().expect("failed to lock mesh");
                if (*mesh_lock).name == String::from(name){
                    result_value = Some(m.clone());
                }
            },
            _ => {}, //no mesh
        }

        //Have a look if we found it in the content
        //if not search in childs
        match result_value{
            //if we already found somthing, don't do anything
            Some(_)=> {},
            None=> {
                //Cycling though the children till we got any Some(x)
                for (_, i) in self.children.iter_mut(){
                    //make sure we dont overwrite the right value with a none of the next value
                    match result_value{
                        None=> result_value = i.get_mesh(name.clone()),
                        //if tmp holds something overwerite the result_value
                        //the early return makes sure we dont overwrite our found falue with another
                        //none
                        Some(value)=> return Some(value),
                    }

                }
            }

        }
        result_value
    }

    ///Returns the first light point with this name
    pub fn get_light_point(&mut self, name: &str) -> Option<&mut light::LightPoint>{
        let mut result_value: Option<&mut light::LightPoint> = None;
        match self.content{
            ContentType::Light(LightsContent::PointLight(ref mut sp)) => {
                result_value = Some(sp);
            }
            _ => {}, //its not self
        }
        //Have a look if we found it in the content
        //if not search in childs
        match result_value{
            //if we already found somthing, don't do anything
            Some(_)=> {},
            None=> {
                //Cycling though the children till we got any Some(x)
                for (_, i) in self.children.iter_mut(){
                    //make sure we dont overwrite the right value with a none of the next value
                    match result_value{
                        None=> result_value = i.get_light_point(name.clone()),
                        Some(value)=> return Some(value),
                    }
                }
            }
        }
        result_value
    }

    ///Returns the first directional light with this name
    pub fn get_light_directional(&mut self, name: &str) -> Option<&mut light::LightDirectional>{
        let mut result_value: Option<&mut light::LightDirectional> = None;
        match self.content{
            ContentType::Light(LightsContent::DirectionalLight(ref mut sp)) => {
                result_value = Some(sp);
            }
            _ => {}, //its not self
        }
        //Have a look if we found it in the content
        //if not search in childs
        match result_value{
            //if we already found somthing, don't do anything
            Some(_)=> {},
            None=> {
                //Cycling though the children till we got any Some(x)
                for (_, i) in self.children.iter_mut(){
                    //make sure we dont overwrite the right value with a none of the next value
                    match result_value{
                        None=> result_value = i.get_light_directional(name.clone()),
                        Some(value)=> return Some(value),
                    }
                }
            }
        }
        result_value
    }

    ///Returns the first light spot with this name
    pub fn get_light_spot(&mut self, name: &str) -> Option<&mut light::LightSpot>{
        let mut result_value: Option<&mut light::LightSpot> = None;
        match self.content{
            ContentType::Light(LightsContent::SpotLight(ref mut sp)) => {
                result_value = Some(sp);
            }
            _ => {}, //its not self
        }
        //Have a look if we found it in the content
        //if not search in childs
        match result_value{
            //if we already found somthing, don't do anything
            Some(_)=> {},
            None=> {
                //Cycling though the children till we got any Some(x)
                for (_, i) in self.children.iter_mut(){
                    //make sure we dont overwrite the right value with a none of the next value
                    match result_value{
                        None=> result_value = i.get_light_spot(name.clone()),
                        Some(value)=> return Some(value),
                    }
                }
            }
        }
        result_value
    }


    ///Returns all meshes in view frustum as well as their transform
    pub fn get_meshes_in_frustum(&mut self, camera: &camera::DefaultCamera) -> Vec<(Arc<Mutex<mesh::Mesh>>, Matrix4<f32>)>{

        let mut return_vector = Vec::new();

        let camera_frustum = camera.get_frustum_bound();

        //FIXME also add the dynamic meshse
        match self.content{
            //if selfs content is a mesh, check the bound
            ContentType::Renderable(RenderableContent::Mesh(ref mesh)) => {
                //check if self is in bound

                let test = {
                    camera_frustum.contains(&self.bound)
                };

                match test{
                    Relation::In => return_vector.push((mesh.clone(), self.get_transform_matrix())),
                    Relation::Cross => return_vector.push((mesh.clone(), self.get_transform_matrix())),
                    Relation::Out => return return_vector,
                }
            },
            //if self is no mesh, just check the bound
            _ => {
                let test = {
                    camera_frustum.contains(&self.bound)
                };
                match test{
                    Relation::In => {},
                    Relation::Cross => {},
                    Relation::Out => return return_vector,
                }
            }
        }


        //if not already return because the bound is too small, check the children
        for (_, i) in self.children.iter_mut(){
            return_vector.append(&mut i.get_meshes_in_volume(&camera_frustum, camera.get_position()));
        }
        return_vector
    }

    ///checks for bounds in a volume, view frustum or maybe for a locale collision check
    pub fn get_meshes_in_volume(
        &mut self, volume: &Frustum<f32>, location: Vector3<f32>
    ) -> Vec<(Arc<Mutex<mesh::Mesh>>, Matrix4<f32>)>{

        let mut return_vector = Vec::new();
        match self.content{
            //if selfs content is a mesh, check the bound
            ContentType::Renderable(RenderableContent::Mesh(ref mesh)) => {
                //check if self is in bound
                let test = {
                    volume.contains(&self.bound)
                };
                match test{
                    Relation::In => return_vector.push((mesh.clone(), self.get_transform_matrix())),
                    Relation::Cross => return_vector.push((mesh.clone(), self.get_transform_matrix())),
                    Relation::Out => return return_vector,
                }
            },
            //if self is no mesh, just check the bound
            _ => {
                let test = {
                    volume.contains(&self.bound)
                };
                match test{
                    Relation::In => {},
                    Relation::Cross => {},
                    Relation::Out => return return_vector,
                }
            }
        }


        //if not already return because the bound is too small, check the children
        for (_, i) in self.children.iter_mut(){
            return_vector.append(&mut i.get_meshes_in_volume(&volume, location));
        }
        return_vector
    }

    ///Gets all meshes from this node down
    pub fn get_all_meshes(&mut self) -> Vec<(Arc<Mutex<mesh::Mesh>>, Matrix4<f32>)>{
        let mut return_vector = Vec::new();

        match self.content{
            ContentType::Renderable(RenderableContent::Mesh(ref mesh)) => {
                return_vector.push((mesh.clone(), self.get_transform_matrix()));
            },
            _ => {},
        }

        //println!("Returning tanslation of: {:?}", self.get_transform_matrix());
        //Go down the tree
        for (_, i) in self.children.iter_mut(){
            return_vector.append(&mut i.get_all_meshes());
        }
        return_vector
    }

    ///Gets all LightPoint from this node down
    pub fn get_all_point_lights(&mut self) -> Vec<core::resources::light::LightPoint>{
        let mut return_vector = Vec::new();

        //Check self
        match self.content{
            ContentType::Light(LightsContent::PointLight(ref pl)) => return_vector.push(pl.clone()),
            _ => {},
        }

        //Go down the tree
        for (_, i) in self.children.iter_mut(){
            return_vector.append(&mut i.get_all_point_lights());
        }
        return_vector
    }

    ///Gets all LightPoint from this node down
    pub fn get_all_directional_lights(&mut self) -> Vec<core::resources::light::LightDirectional>{
        let mut return_vector = Vec::new();

        //Check self
        match self.content{
            ContentType::Light(LightsContent::DirectionalLight(ref pl)) => return_vector.push(pl.clone()),
            _ => {},
        }

        //Go down the tree
        for (_, i) in self.children.iter_mut(){
            return_vector.append(&mut i.get_all_directional_lights());
        }
        return_vector
    }

    ///Gets all LightSpot from this node down
    pub fn get_all_spot_lights(&mut self) -> Vec<core::resources::light::LightSpot>{
        let mut return_vector = Vec::new();
        //Check self
        //Check self
        match self.content{
            ContentType::Light(LightsContent::SpotLight(ref pl)) => return_vector.push(pl.clone()),
            _ => {},
        }
        for (_, i) in self.children.iter_mut(){
            return_vector.append(&mut i.get_all_spot_lights());
        }
        return_vector
    }

    ///Returns the bound of `content` in self as mutable reference
    pub fn get_bound(&mut self) -> &mut Aabb3<f32>{
        &mut self.bound
    }

    ///Returns the maximum bound values from this node down
    pub fn get_bound_max(&mut self) -> Point3<f32>{

        let mut return_max = self.bound.max.clone();

        //Compare self with the children an their children etc.
        for (_, i) in self.children.iter_mut(){
            let child_max = i.get_bound_max();

            //Comapare per axis    X
            if child_max[0] > return_max[0]{
                return_max[0] = child_max[0].clone();
            }

            //Comapare per axis    Y
            if child_max[1] > return_max[1]{
                return_max[1] = child_max[1].clone();
            }

            //Comapare per axis    Z
            if child_max[2] > return_max[2]{
                return_max[2] = child_max[2].clone();
            }
        }
        //Retrurn the smallest values

        return_max
    }

    ///Returns the min bound values from this node down
    ///Compares per axis
    pub fn get_bound_min(&mut self) -> Point3<f32>{

        let mut return_min = self.bound.min.clone();

        //Compare self with the children an their children etc.
        for (_, i) in self.children.iter_mut(){
            let child_min = i.get_bound_min();

            //Comapare per axis    X
            if child_min[0] < return_min[0]{
                return_min[0] = child_min[0].clone();
            }

            //Comapare per axis    Y
            if child_min[1] < return_min[1]{
                return_min[1] = child_min[1].clone();
            }

            //Comapare per axis    Z
            if child_min[2] < return_min[2]{
                return_min[2] = child_min[2].clone();
            }
        }
        //Retrurn the smallest values
        return_min
    }

    ///Rebuilds bounds for this node down
    ///should usually be applied to the root node only not
    ///if you are sure that the new bound doesnt extend the old parent bound of a node
    pub fn rebuild_bounds(&mut self){

        //First rebuild the bounds of all sub children
        for (_, k) in self.children.iter_mut(){
            k.rebuild_bounds();
        }
        //Then get the new max and min values
        let new_min = self.get_bound_min();
        let new_max = self.get_bound_max();
        //and use them for own bound
        self.bound = Aabb3::new(new_min, new_max);
    }

    ///prints a visual representation of the tree to the terminal
    pub fn print_member(&self, depth: u32){
        //add space
        for _ in 0..depth{
            print!("    ", );
        }
        //print name behind space
        //as well as its bound for debug reason
        print!("NAME: {} BOUNDS: ", self.name);
        print!("min: [{}, {}, {}]   max: [{}, {}, {}], Location: [{},{},{}] \n",
            self.bound.min[0],
            self.bound.min[1],
            self.bound.min[2],

            self.bound.max[0],
            self.bound.max[1],
            self.bound.max[2],
            self.transform.disp.x,
            self.transform.disp.y,
            self.transform.disp.z,
        );
        for (_, i) in self.children.iter(){
            i.print_member(depth + 1);
        }
    }
}
