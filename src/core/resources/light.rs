
use cgmath::*;
use collision;


use render::shader_impls::pbr_vertex;
use render::shader_impls::pbr_fragment;
use core::ReturnBoundInfo;
use core::simple_scene_system::node;

use std::sync::{Arc,Mutex};
use std::f64::consts;

use core::resources;

///A Generic Point Light
#[derive(Clone)]
pub struct LightPoint {
    pub name: String,
    intensity: f32,
    color: Vector3<f32>,
    location: Vector3<f32>,

    bound: collision::Aabb3<f32>,
}



///A generic directional light i.e. a sun
#[derive(Clone)]
pub struct LightDirectional {
    pub name: String,
    intensity: f32,
    color: Vector3<f32>,
    location: Vector3<f32>,
    direction: Vector3<f32>,

    bound: collision::Aabb3<f32>,
}



///A generic spot light, like car lights or stage light
#[derive(Clone)]
pub struct LightSpot {
    pub name: String,
    intensity: f32,
    color: Vector3<f32>,
    location: Vector3<f32>,

    direction: Vector3<f32>,

    outer_radius: f32,
    inner_radius: f32,

    bound: collision::Aabb3<f32>,
}



///Custom PointLight implementation
impl LightPoint{
    ///Returns the Member with the passed `name`
    ///Special parameters light radius or color will have to be set later
    pub fn new(name: &str)->Self{
        //Creating the box extend from the location, there might be a better way
        let min = Point3::new(-0.5, -0.5, -0.5, );
        let max = Point3::new(0.5, 0.5, 0.5, );

        LightPoint{
            name: String::from(name),
            intensity: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),
            location: Vector3::new(1.0, 1.0, 1.0),


            bound: collision::Aabb3::new(min, max),
        }
    }
    ///Returns this lught as its shader-useable instance
    pub fn as_shader_info(&self) -> pbr_fragment::ty::PointLight{
        //convert to a Vec4 for 128 bit padding in the shader
        let color_type: [f32; 3] = self.color.into();
        let location_type: [f32; 3] = self.location.into();
        //Return a native vulkano struct
        pbr_fragment::ty::PointLight{
            color: color_type,
            location: location_type,
            intensity: self.intensity,
            _dummy0: [0; 4],
        }


    }

    ///sets the lights intensity
    #[inline]
    pub fn set_intensity(&mut self, new_itensity: f32){
        self.intensity = new_itensity;
    }

    ///returns the refernce to the intensity
    #[inline]
    pub fn get_intensity(&mut self) -> &mut f32{
        &mut self.intensity
    }

    ///Returns the location reference
    #[inline]
    pub fn get_location(&mut self) -> &mut Vector3<f32>{
        &mut self.location
    }

    ///Returns the location reference
    #[inline]
    pub fn set_location(&mut self, new_location: Vector3<f32>){
        self.location = new_location
    }

    ///Sets its color, the value gets normalized, set the intensity via `set_intensity`
    #[inline]
    pub fn set_color(&mut self, new_color: Vector3<f32>){
        self.color = new_color;
    }

    ///Returns the reference to its color
    #[inline]
    pub fn get_color(&mut self) -> &mut Vector3<f32>{
        &mut self.color
    }
}

impl ReturnBoundInfo for LightPoint{
    ///return the max size of its bound
    #[inline]
    fn get_bound_max(&self)-> Point3<f32>{
        self.bound.max.clone()
    }
    ///return the min size of its bound
    #[inline]
    fn get_bound_min(&self)-> Point3<f32>{
        self.bound.min.clone()
    }
    ///Sets the bound to the new values (in mesh space)
    fn set_bound(&mut self, min: Point3<f32>, max: Point3<f32>){
        let min = Point3::new(
            min[0],
            min[1],
            min[2]
        );

        let max = Point3::new(
            max[0],
            max[1],
            max[2]
        );

        self.bound = collision::Aabb3::new(min, max);
    }

    ///Returns its bound
    #[inline]
    fn get_bound(&self) -> collision::Aabb3<f32>{
        self.bound.clone()
    }

    ///Returns the vertices of the bounding mesh, good for debuging
    fn get_bound_points(&self)-> Vec<Vector3<f32>>{
        let mut return_vector = Vec::new();

        let b_min = self.bound.min.clone();
        let b_max = self.bound.max.clone();

        //low
        return_vector.push(Vector3::new(b_min[0], b_min[1], b_min[2])); //Low
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1], b_min[2])); //+x
        return_vector.push(Vector3::new(b_min[0], b_min[1] + b_max[1], b_min[2])); //+y
        return_vector.push(Vector3::new(b_min[0],  b_min[1], b_min[2] + b_max[2])); // +z
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1] + b_max[1], b_min[2])); //+xy
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1], b_min[2] + b_max[2])); //+xz
        return_vector.push(Vector3::new(b_min[0] , b_min[1] + b_max[1], b_min[2] + b_max[1])); //+yz
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1] + b_max[1], b_min[2] + b_max[2])); //+xyz

        return_vector
    }
}

///Special functions for directional lights
impl LightDirectional{
    ///Returns the Member with the passed `name`
    ///Special parameters light radius or color will have to be set later
    pub fn new(name: &str)->Self{
        //Creating the box extend from the location, there might be a better way
        let min = Point3::new(-0.5, -0.5, -0.5, );
        let max = Point3::new(0.5, 0.5, 0.5, );
        let direction = Vector3::new(1.0, 1.0, 1.0);

        LightDirectional{
            name: String::from(name),

            intensity: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),
            location: Vector3::new(1.0, 1.0, 1.0),

            direction: direction,

            bound: collision::Aabb3::new(min, max),
        }
    }

    ///Returns this lught as its shader-useable instance
    pub fn as_shader_info(&self) -> pbr_fragment::ty::DirectionalLight{

        let tmp_color: [f32;3] = self.color.into();
        let tmp_direction: [f32;3] = self.direction.into();
        let location_type: [f32; 3] = self.location.into();

        //Return a native vulkano struct
        pbr_fragment::ty::DirectionalLight{
            color: tmp_color,
            direction: tmp_direction,
            location: location_type,
            intensity: self.intensity,
            _dummy0: [0; 4],
            _dummy1: [0; 4],
        }
    }

    ///Change the direction
    #[inline]
    pub fn set_direction(&mut self, new_direction: Vector3<f32>){
        self.direction = new_direction;
    }

    ///Returns the direction reference
    #[inline]
    pub fn get_direction(&mut self) -> &mut Vector3<f32>{
        &mut self.direction
    }

    ///Returns the location reference
    #[inline]
    pub fn get_location(&mut self) -> &mut Vector3<f32>{
        &mut self.location
    }

    ///Returns the location reference
    #[inline]
    pub fn set_location(&mut self, new_location: Vector3<f32>){
        self.location = new_location
    }

    ///set intensity
    #[inline]
    pub fn set_intensity(&mut self, new_itensity: f32){
        self.intensity = new_itensity;
    }

    ///returns the refernce to the intensity
    #[inline]
    pub fn get_intensity(&mut self) -> &mut f32{
        &mut self.intensity
    }

    ///Sets its color, the value gets normalized, set the intensity via `set_intensity`
    #[inline]
    pub fn set_color(&mut self, new_color: Vector3<f32>){
        self.color = new_color;
    }

    ///Returns the reference to its color
    #[inline]
    pub fn get_color(&mut self) -> &mut Vector3<f32>{
        &mut self.color
    }


}

impl ReturnBoundInfo for LightDirectional{
    ///return the max size of its bound
    fn get_bound_max(&self)-> Point3<f32>{
        self.bound.max.clone()
    }
    ///return the min size of its bound
    fn get_bound_min(&self)-> Point3<f32>{
        self.bound.min.clone()
    }
    ///Sets the bound to the new values (in mesh space)
    fn set_bound(&mut self, min: Point3<f32>, max: Point3<f32>){
        let min = Point3::new(
            min[0],
            min[1],
            min[2]
        );

        let max = Point3::new(
            max[0],
            max[1],
            max[2]
        );
        self.bound = collision::Aabb3::new(min, max);
    }

    ///Returns it' bound
    #[inline]
    fn get_bound(&self) -> collision::Aabb3<f32>{
        self.bound.clone()
    }

    ///Returns the vertices of the bounding mesh, good for debuging
    fn get_bound_points(& self)-> Vec<Vector3<f32>>{
        let mut return_vector = Vec::new();

        let b_min = self.bound.min.clone();
        let b_max = self.bound.max.clone();

        //low
        return_vector.push(Vector3::new(b_min[0], b_min[1], b_min[2])); //Low
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1], b_min[2])); //+x
        return_vector.push(Vector3::new(b_min[0], b_min[1] + b_max[1], b_min[2])); //+y
        return_vector.push(Vector3::new(b_min[0],  b_min[1], b_min[2] + b_max[2])); // +z
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1] + b_max[1], b_min[2])); //+xy
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1], b_min[2] + b_max[2])); //+xz
        return_vector.push(Vector3::new(b_min[0] , b_min[1] + b_max[1], b_min[2] + b_max[1])); //+yz
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1] + b_max[1], b_min[2] + b_max[2])); //+xyz

        return_vector
    }
}

///Special functions for the spot light
impl LightSpot{
    ///Returns the Member with the passed `name`
    ///Special parameters light radius or color will have to be set later
    pub fn new(name: &str)->Self{
        //Creating the box extend from the location, there might be a better way
        let min = Point3::new(-0.5, -0.5, -0.5, );
        let max = Point3::new(0.5, 0.5, 0.5, );

        let direction = Vector3::new(1.0, 1.0, 1.0);
        let outer_radius = 50.0;
        let inner_radius = 40.0;

        LightSpot{
            name: String::from(name),
            intensity: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),
            location: Vector3::new(1.0, 1.0, 1.0),

            direction: direction,

            outer_radius: outer_radius,
            inner_radius: inner_radius,

            bound: collision::Aabb3::new(min, max),
        }
    }

    ///Returns this lught as its shader-useable instance
    pub fn as_shader_info(&self) -> pbr_fragment::ty::SpotLight{

        let tmp_color: [f32;3] = self.color.into();
        let tmp_direction: [f32;3] = self.direction.into();
        let location_type: [f32; 3] = self.location.into();

        pbr_fragment::ty::SpotLight{
            color: tmp_color,
            direction: tmp_direction,
            location: location_type,
            intensity: self.intensity,
            //to save some graphics power calculating the cosin directly and using it in the shader

            outer_radius: to_radians(self.outer_radius).cos(),
            inner_radius: to_radians(self.inner_radius).cos(),
            _dummy0: [0; 4],
            _dummy1: [0; 4],
            _dummy2: [0; 8],
        }
    }

    ///Change the direction
    #[inline]
    pub fn set_direction(&mut self, new_direction: Vector3<f32>){
        self.direction = new_direction;
    }

    ///Returns the direction reference
    #[inline]
    pub fn get_direction(&mut self) -> &mut Vector3<f32>{
        &mut self.direction
    }

    ///Returns the location reference
    #[inline]
    pub fn get_location(&mut self) -> &mut Vector3<f32>{
        &mut self.location
    }

    ///Returns the location reference
    #[inline]
    pub fn set_location(&mut self, new_location: Vector3<f32>){
        self.location = new_location
    }

    ///set intensity
    #[inline]
    pub fn set_intensity(&mut self, new_itensity: f32){
        self.intensity = new_itensity;
    }

    ///returns the refernce to the intensity
    #[inline]
    pub fn get_intensity(&mut self) -> &mut f32{
        &mut self.intensity
    }

    ///Sets its color, the value gets normalized, set the intensity via `set_intensity`
    #[inline]
    pub fn set_color(&mut self, new_color: Vector3<f32>){
        self.color = new_color;
    }

    ///Returns the reference to its color
    #[inline]
    pub fn get_color(&mut self) -> &mut Vector3<f32>{
        &mut self.color
    }

    ///Sets the outer radius (point where the fallof ends) of this spot light
    #[inline]
    pub fn set_outer_radius(&mut self, new_radius: f32){
        self.outer_radius = new_radius;
    }

    ///Returns the reference to the outer radius
    #[inline]
    pub fn get_outer_radius(&mut self) -> &mut f32{
        &mut self.outer_radius
    }

    ///Sets the inner radius (point where the fallof starts) of this spot light
    #[inline]
    pub fn set_inner_radius(&mut self, new_radius: f32){
        self.inner_radius = new_radius;
    }

    ///Returns the reference to the inner radius
    #[inline]
    pub fn get_inner_radius(&mut self) -> &mut f32{
        &mut self.inner_radius
    }
}

impl ReturnBoundInfo for LightSpot{
    ///return the max size of its bound
    #[inline]
    fn get_bound_max(&self)-> Point3<f32>{
        self.bound.max.clone()
    }
    ///return the min size of its bound
    #[inline]
    fn get_bound_min(&self)-> Point3<f32>{
        self.bound.min.clone()
    }
    ///Sets the bound to the new values (in mesh space)
    fn set_bound(&mut self, min: Point3<f32>, max: Point3<f32>){
        let min = Point3::new(
            min[0],
            min[1],
            min[2]
        );

        let max = Point3::new(
            max[0],
            max[1],
            max[2]
        );

        self.bound = collision::Aabb3::new(min, max);
    }

    ///Returns it' bound
    #[inline]
    fn get_bound(&self) -> collision::Aabb3<f32>{
        self.bound.clone()
    }

    ///Returns the vertices of the bounding mesh, good for debuging
    fn get_bound_points(&self)-> Vec<Vector3<f32>>{
        let mut return_vector = Vec::new();

        let b_min = self.bound.min.clone();
        let b_max = self.bound.max.clone();

        //low
        return_vector.push(Vector3::new(b_min[0], b_min[1], b_min[2])); //Low
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1], b_min[2])); //+x
        return_vector.push(Vector3::new(b_min[0], b_min[1] + b_max[1], b_min[2])); //+y
        return_vector.push(Vector3::new(b_min[0],  b_min[1], b_min[2] + b_max[2])); // +z
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1] + b_max[1], b_min[2])); //+xy
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1], b_min[2] + b_max[2])); //+xz
        return_vector.push(Vector3::new(b_min[0] , b_min[1] + b_max[1], b_min[2] + b_max[1])); //+yz
        return_vector.push(Vector3::new(b_min[0] + b_max[0], b_min[1] + b_max[1], b_min[2] + b_max[2])); //+xyz

        return_vector
    }

}
//Helper function for calculating the view
#[inline]
fn to_radians(degree: f32) -> f32 {
    degree * (consts::PI / 180.0) as f32
}
