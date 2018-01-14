
use cgmath::*;
use collision;


use render::shader_impls::lights;
use core::ReturnBoundInfo;

//use std::sync::{Arc,Mutex};
use std::f64::consts;


///A Generic Point Light
#[derive(Clone)]
pub struct LightPoint {
    pub name: String,
    intensity: f32,
    radius: f32,
    color: Vector3<f32>,

    bound: collision::Aabb3<f32>,
}



///A generic directional light i.e. a sun
#[derive(Clone)]
pub struct LightDirectional {
    pub name: String,
    intensity: f32,
    color: Vector3<f32>,

    bound: collision::Aabb3<f32>,
}



///A generic spot light, like car lights or stage light
#[derive(Clone)]
pub struct LightSpot {
    pub name: String,
    intensity: f32,
    color: Vector3<f32>,

    radius: f32,
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
            radius: 5.0,

            bound: collision::Aabb3::new(min, max),
        }
    }
    ///Returns this lught as its shader-useable instance
    pub fn as_shader_info(&self, location: &Vector3<f32>) -> lights::ty::PointLight{
        //convert to a Vec4 for 128 bit padding in the shader
        let color_type: [f32; 3] = self.color.into();
        let location_type: [f32; 3] = location.clone().into();
        //Return a native vulkano struct
        lights::ty::PointLight{
            color: color_type,
            location: location_type,
            intensity: self.intensity,
            radius: self.radius,
            _dummy0: [0; 4],
            _dummy1: [0; 12],
        }


    }

    ///sets the lights intensity
    #[inline]
    pub fn set_intensity(&mut self, new_itensity: f32){
        //check for under 0 value, if so do nothing
        if new_itensity<=0.0{
            return;
        }
        self.intensity = new_itensity;
    }

    ///returns the refernce to the intensity
    #[inline]
    pub fn get_intensity(&mut self) -> &mut f32{
        &mut self.intensity
    }

    ///sets the lights intensity
    #[inline]
    pub fn set_radius(&mut self, new_radius: f32){
        self.radius = new_radius;
        self.rebuild_bound();
    }

    ///returns the refernce to the radius of this light source
    #[inline]
    pub fn get_radius(&mut self) -> &mut f32{
        &mut self.radius
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

    ///Rebuilds bound based on intensity
    fn rebuild_bound(&mut self){

        //following https://developer.valvesoftware.com/wiki/Constant-Linear-Quadratic_Falloff and UE4 radius + brightness
        let radius = self.radius;
        self.bound = collision::Aabb3::new(
            Point3::new(-radius, -radius, -radius),
            Point3::new(radius, radius, radius)
        );
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

        LightDirectional{
            name: String::from(name),

            intensity: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),

            bound: collision::Aabb3::new(min, max),
        }
    }

    ///Returns this lught as its shader-useable instance
    pub fn as_shader_info(&self, rotation: &Quaternion<f32>) -> lights::ty::DirectionalLight{
        let tmp_color: [f32;3] = self.color.into();
        //Transfere to the shader type [f32;3]
        let tmp_direction: [f32;3] = rotation.rotate_vector(Vector3::new(1.0, 0.0, 0.0)).into();
        //Return a native vulkano struct
        lights::ty::DirectionalLight{
            color: tmp_color,
            direction: tmp_direction,
            intensity: self.intensity,
            _dummy0: [0; 4],
        }
    }


    ///set intensity
    #[inline]
    pub fn set_intensity(&mut self, new_itensity: f32){
        //check for under 0 value, if so do nothing
        if new_itensity<=0.0{
            return;
        }
        self.rebuild_bound()
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

    ///Rebuilds bound, but directional lights have no bound (atm), so do nothing
    fn rebuild_bound(&mut self){
        //nothing
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

        let outer_radius = 50.0;
        let inner_radius = 40.0;

        LightSpot{
            name: String::from(name),
            intensity: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),

            radius: 5.0,
            outer_radius: outer_radius,
            inner_radius: inner_radius,

            bound: collision::Aabb3::new(min, max),
        }
    }

    ///Returns this lught as its shader-useable instance
    pub fn as_shader_info(&self, rotation: &Quaternion<f32>, location: &Vector3<f32>) -> lights::ty::SpotLight{

        let tmp_color: [f32;3] = self.color.into();
        //Transfere to the shader type [f32;3]
        let tmp_direction: [f32;3] = rotation.rotate_vector(Vector3::new(1.0, 0.0, 0.0)).into();
        let location_type: [f32; 3] = location.clone().into();



        lights::ty::SpotLight{
            color: tmp_color,
            direction: tmp_direction,
            location: location_type,
            intensity: self.intensity,
            radius: self.radius,
            //to save some graphics power calculating the cosin directly and using it in the shader

            outer_radius: to_radians(self.outer_radius).cos(),
            inner_radius: to_radians(self.inner_radius).cos(),
            _dummy0: [0; 4],
            _dummy1: [0; 4],
            _dummy2: [0; 4],
        }
    }

    ///set intensity
    #[inline]
    pub fn set_intensity(&mut self, new_itensity: f32){
        //check for under 0 value, if so do nothing
        if new_itensity<=0.0{
            return;
        }
        self.intensity = new_itensity;
    }

    ///returns the refernce to the intensity
    #[inline]
    pub fn get_intensity(&mut self) -> &mut f32{
        &mut self.intensity
    }

    ///sets the lights intensity
    #[inline]
    pub fn set_radius(&mut self, new_radius: f32){
        self.radius = new_radius;
        self.rebuild_bound();
    }

    ///returns the refernce to the radius of this light source
    #[inline]
    pub fn get_radius(&mut self) -> &mut f32{
        &mut self.radius
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

    ///Rebuilds bound based on intensity, but only in the + direction, because its the only direction
    /// a spotlight shines
    fn rebuild_bound(&mut self){
        //following https://developer.valvesoftware.com/wiki/Constant-Linear-Quadratic_Falloff
        //we calculate the max radius of the light for 1/256 as min. intensity


        let radius = self.radius;
        let y_z_extend = self.outer_radius.sin() * radius;
        self.bound = collision::Aabb3::new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(radius, y_z_extend, -y_z_extend)//we can make the assumption that the spot light
            //is always "looking" in x direction because of the way the direction vector is computed in the
            // to_shader_info() //TODO Check for function
        );
    }

}
//Helper function for calculating the view
#[inline]
fn to_radians(degree: f32) -> f32 {
    degree * (consts::PI / 180.0) as f32
}
