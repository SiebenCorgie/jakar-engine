use std::sync::{Arc, Mutex};
use std::fmt;

use core::resources::mesh;
use core::resources::light;
use core::resources::empty;
use core::resources::camera::{DefaultCamera, Camera};
use core::ReturnBoundInfo;

use super::attributes::NodeAttributes;
use core;
use render::render_traits::ForwardRenderAble;


use jakar_tree;

use cgmath::*;
use collision::*;

///All possible types of content a Node can hold.
///Changed in order to apply a new type
#[derive(Clone)]
pub enum ContentType {
    ///Can store other renderable structures. However, try to use meshes since they don't have to
    /// perform dynamic dispatch.
    Renderable(Arc<Mutex<ForwardRenderAble + Send + Sync>>),
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
    Camera(DefaultCamera),
}

impl fmt::Debug for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let string = {
            match self{
                ContentType::Renderable(_) => "renderable",
                ContentType::Mesh(_) => "mesh",
                ContentType::PointLight(_) => "point light",
                ContentType::DirectionalLight(_) => "directional light",
                ContentType::SpotLight(_) => "spot light",
                ContentType::Empty(_) => "empty",
                ContentType::Camera(_) => "camera",
            }
        };

        write!(f, "{}", string)
    }
}


impl ContentType{
    ///Returns the bound of this content
    pub fn get_bound(&self) -> Aabb3<f32>{
        match self{
            &ContentType::Renderable(ref renderable) => {
                let render_able_lck = renderable.lock().expect("Failed to lock renderable");
                render_able_lck.get_bound()
            },
            &ContentType::Mesh(ref mesh) => {
                //lock the mesh resource to get the bound
                let mesh_lck = mesh.lock().expect("failed to lock mesh");
                mesh_lck.get_bound()
            },
            &ContentType::PointLight(ref light) => {
                light.get_bound()
            },
            &ContentType::DirectionalLight(ref light) => {
                light.get_bound()
            },
            &ContentType::SpotLight(ref light) => {
                light.get_bound()
            },
            &ContentType::Empty(ref empty) => {
                empty.get_bound()
            },
            &ContentType::Camera(ref _camera) => {
                //Always returns a 1x1x1 bound
                Aabb3::new(Point3::new(-0.5, -0.5, -0.5), Point3::new(0.5, 0.5, 0.5))
            },
        }
    }

    ///Returns the either a mesh or a None
    pub fn as_mesh(&mut self) -> Option<Arc<Mutex<mesh::Mesh>>>{
        match self{
            &mut ContentType::Mesh(ref mesh) => return Some(mesh.clone()),
            _ => None
        }
    }

    ///Returns the either a point light or a None
    pub fn as_point_light(&mut self) -> Option<&mut light::LightPoint>{
        match self{
            &mut ContentType::PointLight(ref mut light) => return Some(light),
            _ => None
        }
    }

    ///Returns the either a directional light or a None
    pub fn as_directional_light(&mut self) -> Option<&mut light::LightDirectional>{
        match self{
            &mut ContentType::DirectionalLight(ref mut light) => return Some(light),
            _ => None
        }
    }

    ///Returns the either a spot light or a None
    pub fn as_spot_light(&mut self) -> Option<&mut light::LightSpot>{
        match self{
            &mut ContentType::SpotLight(ref mut light) => return Some(light),
            _ => None
        }
    }

    ///Returns the either a empty or a None
    pub fn as_empty(&mut self) -> Option<&mut empty::Empty>{
        match self{
            &mut ContentType::Empty(ref mut empty) => return Some(empty),
            _ => None
        }
    }

    ///Returns the either a camera or a None
    pub fn as_camera(&mut self) -> Option<&mut DefaultCamera>{
        match self{
            &mut ContentType::Camera(ref mut cam) => {
                return Some(cam);
            },
            _ => None
        }
    }
}

impl jakar_tree::node::NodeContent for ContentType{
    ///Should return the name of this content
    fn get_name(&self) -> String{
        match self{
            &ContentType::Renderable(ref ra) =>{
                let ra_lock = ra.lock().expect("failed to lock mesh");
                ra_lock.get_name()
            },
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
            &ContentType::Camera(ref _c) => {
                //c.name.clone() TODO add a camera name
                String::from("Camera")
            },
        }
    }
}
