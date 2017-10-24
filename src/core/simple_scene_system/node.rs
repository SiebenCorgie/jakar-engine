
use std::sync::{Arc, Mutex};

use cgmath;
use cgmath::*;
use collision::*;
//use core::AABB3Intersection;
//use collision::Discrete;
use core::ReturnBoundInfo;

use rt_error;
use core;
use core::simple_scene_system::node_helper;
use core::resources::mesh;
use core::resources::light;
use core::resources::empty;
use core::resources::camera;
use core::resources::camera::Camera;
use render::pipeline_builder;

//used for sorting and searching the children
use std::collections::BTreeMap;

///All possible types of content a Node can hold.
///Changed in order to apply a new type
#[derive(Clone)]
pub enum ContentType {
    /// is a mesh with a vertex buffer as well as a material
    Mesh(Arc<Mutex<mesh::Mesh>>),
    /// is a light casting a 360° light
    PointLight(light::LightPoint),
    /// cast light into one direction
    DirectionalLight(light::LightDirectional),
    /// creates a spot light cone
    SpotLight(light::LightSpot),
    /// an empty type, can be used as "folder" in an node hierachy
    Empty(empty::Empty),
    /// a camera attached to the tree (TODO needs to be implemented correctly)
    Camera(camera::DefaultCamera),
}

///Flags an node can have
#[derive(Clone, PartialEq)]
pub struct NodeFlags {
    /// Can be turned off to disable shadow casting, usefull for many small objects
    pub cast_shadow: bool,
    /// Is used to determin at which point this object is rendered.
    /// There is the first pass for opaque objects, as wella s msked objects, and the second one for
    /// transparent ones.
    pub is_transparent: bool,
    /// If true the object won't be rendered if the engine is in gmae mode.
    pub hide_in_game: bool,
}

impl NodeFlags{
    ///Creates new flags with:
    /// - cast_shadows: true
    /// - is_transparent: false,
    /// - hide_in_game: false,
    pub fn new() -> Self{
        NodeFlags{
            cast_shadow: true,
            is_transparent: false,
            hide_in_game: false,
        }
    }
}



///Some implementations to make the programmers life easier
impl ContentType{
    ///Returns the name of this node
    pub fn get_name(&self) -> String{
        match self{
            &ContentType::Mesh(ref c) =>{
                let mesh_lock = c.lock().expect("failed to lock mesh");
                (*mesh_lock).name.clone()
            },
            &ContentType::PointLight(ref c) => {
                c.name.clone()
            },
            &ContentType::DirectionalLight(ref c) => {
                c.name.clone()
            },
            &ContentType::SpotLight(ref c) => {
                c.name.clone()
            },

            &ContentType::Empty(ref c) => {
                c.name.clone()
            },
            &ContentType::Camera(ref c) => {
                //c.name.clone() TODO add a camera name
                String::from("CameraName")
            },
        }
    }

    ///Returns the bound of this object
    pub fn get_bound(&self) -> Aabb3<f32>{
        match self{
            &ContentType::Mesh(ref c) =>{
                let mesh_lock = c.lock().expect("failed to lock mesh");
                (*mesh_lock).get_bound()
            },
            &ContentType::PointLight(ref c) => {
                c.get_bound()
            },
            &ContentType::DirectionalLight(ref c) => {
                c.get_bound()
            },
            &ContentType::SpotLight(ref c) => {
                c.get_bound()
            },

            &ContentType::Empty(ref c) => {
                c.get_bound()
            },
            &ContentType::Camera(ref c) => {
                //c.name.clone() TODO add a camera name
                Aabb3::new(
                    Point3::new(-0.5, -0.5, -0.5),
                    Point3::new(0.5, 0.5, 0.5)
                )
            },
        }
    }
}

///The normal Node of this Scene Tree
///
/// **Why a BTreeMap and no HashMap?**
/// I decided to use a BTreeMap of Structs where the name is in the struct and the tree mainly because of
/// performance reasons. With small datasets (5-100 entries) the BTreeMap is faster and provides
/// some comfort (you can store the name as a String as key value). However, if you have bigger
/// datasets (over 1,000,000) the HashMap is faster, as specially in `--release` mode.
/// However, this should not be relevant to this node tree because it should mostly consist of mid
/// sized BTreeMaps.
#[derive(Clone)]
pub struct GenericNode {

    //children: Vec<GenericNode>,
    children: BTreeMap<String, GenericNode>,
    ///There is a difference between a `Node`'s name and its `content` name
    pub name: String,
    ///Transform of this node in local space
    pub transform: Decomposed<Vector3<f32>, Quaternion<f32>>,
    ///The bounds of this note, takes the `content` bound as well as the max and min values of
    ///all its children into consideration.
    bound: Aabb3<f32>,

    ///The content is a contaier from the `ContentTypes` type which can hold any implemented
    ///Enum value
    content: ContentType,

    ///The rendering flags of this objects
    flags: NodeFlags,
}

///Implementation of the Node trait for Generic node
impl GenericNode{
    ///Creates a new, empty node
    pub fn new_empty(name: &str)-> Self{

        let tmp_bound = Aabb3::new(Point3::new(-0.5, -0.5, -0.5), Point3::new(0.5, 0.5, 0.5));

        GenericNode{
            children: BTreeMap::new(),
            name: String::from(name),
            transform: cgmath::Transform::one(),

            bound: tmp_bound,

            content: ContentType::Empty(empty::Empty::new(name.clone())),
            flags: NodeFlags::new(),
        }
    }

    ///Should return an node
    pub fn new(content: ContentType)->Self{

        //Create variables needed to fill the final struct but change them depending on the match
        let name = content.get_name();
        let bound = content.get_bound();

        GenericNode{
            children: BTreeMap::new(),
            name: String::from(name),
            transform: cgmath::Transform::one(),

            bound: bound,

            content: content,
            flags: NodeFlags::new(),

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

    ///Can be used to set rendering flags of `self`, NOT the children ones. Usally used at import time,
    /// or when constructing the level.
    pub fn set_flags(&mut self, new_flags: NodeFlags){
        self.flags = new_flags;
    }

    ///Returns the currently used node flags. Can be used to alter them.
    pub fn get_flags(&mut self) -> &mut NodeFlags{
        &mut self.flags
    }

    ///Adds a child node to this node
    #[inline]
    pub fn add_child(&mut self, child: ContentType){

        //match the content, if its a mesh, have a look if we need to change the transparency flag
        let is_transparent = {
            match child {
                ContentType::Mesh(ref mesh) =>{
                    //lock the mesh and have a look at the material properties/the pipeline
                    let mesh_lck = mesh.lock().expect("failed to lock mesh");
                    let material = (*mesh_lck).get_material();
                    let material_lck = material.lock().expect("failed to lock material");
                    let pipeline = (*material_lck).get_pipeline();
                    if pipeline.pipeline_config.blending_operation == pipeline_builder::BlendTypes::BlendAlphaBlending{
                        true
                    }else{
                        false
                    }
                },
                //if it's no mesh don't do anything
                _ => false,
            }
        };

        //create the new node from the type
        let mut tmp_child = GenericNode::new(child);
        if is_transparent{
            tmp_child.get_flags().is_transparent = true;
        }

        //and add it
        self.children.insert(tmp_child.name.clone(), tmp_child);
    }

    ///Adds a already prepared node, good for merging different trees
    #[inline]
    pub fn add_node(&mut self, node: GenericNode){
        //Add it based on its own name
        self.children.insert(node.name.clone(), node);
    }

    ///Adds a `node_to_add` as a child to a node with `node_name` as name
    ///good merging node trees at a specific point
    #[inline]
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
    #[inline]
    pub fn get_transform_matrix(&self) -> Matrix4<f32>{
        Matrix4::from(self.transform)
    }

    ///Sets the transform of this node without changing its children
    #[inline]
    pub fn set_transform_single(&mut self, new_transform: Decomposed<Vector3<f32>, Quaternion<f32>>){
        self.transform = new_transform;
    }

    ///Sets `transform` to the transformation of `self` and its children.
    /// NOTE: behind the scenes its just the `translate()` `rotate()` and `scale()` function constructed from
    /// the `transform` fields.
    pub fn set_transform(&mut self, transform: Decomposed<Vector3<f32>, Quaternion<f32>>){
        //Apply fieldwise
        self.transform.disp = transform.disp;
        self.transform.rot = transform.rot;
        self.transform.scale = transform.scale;
        //for all children
        for (_, child) in self.children.iter_mut(){
            child.set_transform(transform);
        }
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

    ///Set the location for `self`, but not for the children, used many at import time or if you want
    /// to offset this node relative to its children
    pub fn offset_location(&mut self, offset: Vector3<f32>){
        self.transform.disp += offset;
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

    ///Changes the rotation of `self`. Mostly used at import time, but can also be used if a node needs
    /// to be rotated without rotating the childs
    pub fn offset_rotation(&mut self, offset: Vector3<f32>){
        let q_rotation = Quaternion::from(Euler {
            x: Deg(offset.x),
            y: Deg(offset.y),
            z: Deg(offset.z),
        });

        self.transform.rot += q_rotation;
    }

    ///Scales this node by `ammount` as well as its children
    pub fn scale(&mut self, ammount: f32){
        self.transform.scale *= ammount;

        for (_, child) in self.children.iter_mut(){
            child.scale(ammount);
        }
    }

    ///Changes the scale of `self` but not the scale of this nodes children. Is mainly used while
    /// importing, but can also be used to offset the scale of a node.
    pub fn offset_scale(&mut self, offset: f32){
        self.transform.scale += offset;
    }

    ///Returns a mesh from childs with this name
    pub fn get_mesh(&mut self, name: &str)-> Option<Arc<Mutex<core::resources::mesh::Mesh>>>{
        let mut result_value: Option<Arc<Mutex<core::resources::mesh::Mesh>>> = None;

        //match self
        match self.content{
            ContentType::Mesh(ref m) => {
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
            ContentType::PointLight(ref mut sp) => {
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
            ContentType::DirectionalLight(ref mut sp) => {
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
            ContentType::SpotLight(ref mut sp) => {
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
    pub fn get_meshes_in_frustum(
        &mut self, camera: &camera::DefaultCamera,
        mesh_parameter: Option<node_helper::SortAttributes>
    ) -> Vec<(Arc<Mutex<mesh::Mesh>>, Matrix4<f32>)>{

        let mut return_vector = Vec::new();

        let camera_frustum = camera.get_frustum_bound();

        match self.content{
            //if selfs content is a mesh, check the bound
            ContentType::Mesh(ref mesh) => {

                //check if self is in the volume. If so we can have a look at the mesh's bound.
                //if not, we can return early because this mesh, and all its children won't be in the
                //volumen.
                let test = camera_frustum.contains(&self.bound); //we don't need to transform thoose, because they are already in world-space

                //If the bound is within the volume or crosses it, we can
                //A: go through the children and
                //B: check self's mesh if its in the volume (changes are that there are some childs
                // within the bound and outide so it gets "Cross" but this mesh in particular is
                // outside.)

                //return if not in bound
                match test{
                    Relation::Out => return return_vector,
                    //else have a look at self's mesh:
                    _ => {
                        //Read the bound of this mesh
                        let mesh_test = camera_frustum.contains(
                            &self.content.get_bound().transform(&self.transform) //the bound transformed by the nodes transorm information
                        );

                        match mesh_test{
                            //mesh is out, so don't add anything but continue with the children
                            Relation::Out => {},
                            //is at least crossing, so add the mesh
                            _ =>  {

                                //..but before adding, verfiy that the mesh has the right settings
                                let should_add = {
                                    match mesh_parameter.clone(){
                                        Some(param) => {
                                            should_be_added(&self.flags, param)
                                        },
                                        None =>{
                                            //No paramter needed, adding mesh
                                            true
                                        }
                                    }
                                };
                                if should_add{
                                    return_vector.push(
                                        (mesh.clone(), self.get_transform_matrix())
                                    );
                                }
                                //else don't add the mesh but proceede with the other meshes
                            }
                        }
                    },
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
            return_vector.append(&mut i.get_meshes_in_volume(
                &camera_frustum, camera.get_position(), mesh_parameter.clone()
            ));
        }
        return_vector
    }

    ///checks for bounds in a volume, view frustum or maybe for a locale collision check
    pub fn get_meshes_in_volume(
        &mut self, volume: &Frustum<f32>, location: Vector3<f32>,
        mesh_parameter: Option<node_helper::SortAttributes>
    ) -> Vec<(Arc<Mutex<mesh::Mesh>>, Matrix4<f32>)>{

        let mut return_vector = Vec::new();
        match self.content{
            //if selfs content is a mesh, check the bound
            ContentType::Mesh(ref mesh) => {
                //check if self is in the volume. If so we can have a look at the mesh's bound.
                //if not, we can return early because this mesh, and all its children won't be in the
                //volumen.
                let test = volume.contains(&self.bound);


                //If the bound is within the volume or crosses it, we can
                //A: go through the children and
                //B: check self's mesh if its in the volume (changes are that there are some childs
                // within the bound and outide so it gets "Cross" but this mesh in particular is
                // outside.)

                //return if not in bound
                match test{
                    Relation::Out => return return_vector,
                    //else have a look at self's mesh:
                    _ => {
                        let mesh_test = volume.contains(
                            &self.content.get_bound().transform(&self.transform)
                        );

                        match mesh_test{
                            //mesh is out, so don't add anything but continue with the children
                            Relation::Out => {},
                            //is at least crossing, so add the mesh
                            _ =>  {

                                //..but before adding, verfiy that the mesh has the right settings
                                let should_add = {
                                    match mesh_parameter.clone(){
                                        Some(param) => {
                                            should_be_added(&self.flags, param)
                                        },
                                        None =>{
                                            //No paramter needed, adding mesh
                                            true
                                        }
                                    }
                                };
                                if should_add{
                                    return_vector.push(
                                        (mesh.clone(), self.get_transform_matrix())
                                    );
                                }
                                //else don't add the mesh but proceede with the other meshes
                            }
                        }
                    },
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
            return_vector.append(&mut i.get_meshes_in_volume(&volume, location, mesh_parameter.clone()));
        }
        return_vector
    }

    ///Gets all meshes from this node down, you can provide a set of settings for which can be sorted
    /// or you provide `None`, then every mesh will be returned.
    pub fn get_all_meshes(&mut self, mesh_parameter: Option<node_helper::SortAttributes>) -> Vec<(Arc<Mutex<mesh::Mesh>>, Matrix4<f32>)>{
        let mut return_vector = Vec::new();

        match self.content{
            ContentType::Mesh(ref mesh) => {
                //This is a mesh, if needed sort for the parameter
                match mesh_parameter.clone(){
                    Some(param) => {

                        let add_child = should_be_added(&self.flags, param);

                        if add_child{
                            return_vector.push((mesh.clone(), self.get_transform_matrix()));
                        }                        //if one or more values are not right, don't add the child

                    },
                    None =>{
                        //No paramter needed, returning mesh
                        return_vector.push((mesh.clone(), self.get_transform_matrix()));
                    }
                }
            },
            _ => {}, //Is no mesh
        }

        //println!("Returning tanslation of: {:?}", self.get_transform_matrix());
        //Go down the tree
        for (_, i) in self.children.iter_mut(){
            return_vector.append(&mut i.get_all_meshes(mesh_parameter.clone()));
        }
        return_vector
    }

    ///Gets all LightPoint from this node down
    pub fn get_all_point_lights(&mut self) -> Vec<core::resources::light::LightPoint>{
        let mut return_vector = Vec::new();

        //Check self
        match self.content{
            ContentType::PointLight(ref pl) => return_vector.push(pl.clone()),
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
            ContentType::DirectionalLight(ref pl) => return_vector.push(pl.clone()),
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
            ContentType::SpotLight(ref pl) => return_vector.push(pl.clone()),
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

        let mut return_max = self.bound.transform(&self.transform).max.clone(); //transformed self intor worldspace to compare and build new bounds

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

        let mut return_min = self.bound.transform(&self.transform).min.clone();

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


///A helper functions which returns true if a ContentType can be added based on the `requierments`
/// or false if not
fn should_be_added(flags: &NodeFlags, requirements: node_helper::SortAttributes) -> bool{
    //Tets each of the attributes whicuh are needed to comapre and change the
    // return flag accordingly
    let mut should_be_added = true;

    //shadow flag
    match requirements.casts_shadow{
        node_helper::AttributeState::Yes =>{
            if !flags.cast_shadow{
                should_be_added = false; //should cast shadow, but doesn't
            }
        },
        node_helper::AttributeState::No =>{
            if flags.cast_shadow{
                should_be_added = false;
            }
        },
        _ => {} //doenst matter
    }

    //translucency flag
    match requirements.is_translucent{
        node_helper::AttributeState::Yes =>{
            if !flags.is_transparent{
                should_be_added = false; //should be translucent, but isn't
            }
        },
        node_helper::AttributeState::No =>{
            if flags.is_transparent{
                should_be_added = false;
            }
        },
        _ => {} //doenst matter
    }

    //hidden in game
    match requirements.hide_in_game{
        node_helper::AttributeState::Yes =>{
            if !flags.hide_in_game{
                should_be_added = false; //should be hidden, but isn't
            }
        },
        node_helper::AttributeState::No =>{
            if flags.hide_in_game{
                should_be_added = false;
            }
        },
        _ => {} //doenst matter
    }

    should_be_added
}
