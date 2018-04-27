
use cgmath::*;
use collision;
use collision::Aabb;

use core::resources::camera::DefaultCamera;
use core::resources::camera::Camera;
use render::shader::shader_inputs::lights;
use core::ReturnBoundInfo;
use core::PointToVector;
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

        let mut new_light = LightPoint{
            name: String::from(name),
            intensity: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),
            radius: 5.0,

            bound: collision::Aabb3::new(min, max),
        };

        new_light.rebuild_bound();
        new_light
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

        let mut new_light = LightDirectional{
            name: String::from(name),

            intensity: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),

            bound: collision::Aabb3::new(min, max),
        };

        new_light.rebuild_bound();
        new_light
    }

    ///Returns this light as its shader-useable instance
    ///Needs the node rotation and the camera location to calculate the direction and light space
    pub fn as_shader_info(&self,
        rotation: &Quaternion<f32>,
        camera: &DefaultCamera,
        pcf_samples: u32,
        poisson_spreading: f32,
        shadow_region: [[f32; 4]; 4]
    ) -> lights::ty::DirectionalLight{
        let tmp_color: [f32;3] = self.color.normalize().into();

        //Now we create the light space matrix from the direction of this light;
        let (light_space, depths_slices) = {
            let (matrixes, depths) = self.get_mvp(rotation, camera);
            //the four matrixes
            let mut ret_mat = [[[0.0; 4]; 4]; 4];
            for idx in 0..matrixes.len(){
                ret_mat[idx] = matrixes[idx].into();
            }
            (ret_mat, depths)
        };

        //Return a native vulkano struct
        lights::ty::DirectionalLight{
            shadow_region: shadow_region,
            shadow_depths: depths_slices,
            light_space: light_space,
            color: tmp_color,
            direction: self.get_direction_vector(rotation).into(),
            intensity: self.intensity,
            poisson_spread: poisson_spreading,
            pcf_samples: pcf_samples,
            _dummy0: [0; 4],
            _dummy1: [0; 8]
        }
    }

    pub fn get_direction_vector(&self, rotation: &Quaternion<f32>) -> Vector3<f32>{
        rotation.rotate_vector(Vector3::new(1.0, 0.0, 0.0))
    }

    ///Returns the four mvp matrixes for the cascaded shadow mapping as well as the depth slices
    /// which are used for them.
    pub fn get_mvp(&self, rotation: &Quaternion<f32>, cam: &DefaultCamera) -> ([Matrix4<f32>;4], [f32;4]){

        let mut cascade_splits = [0.0; 4];

        let mut return_depths: [f32;4] = [0.0;4];
        let mut proj_matrix: [Matrix4<f32>; 4] = [Matrix4::<f32>::identity(); 4];

        let near_clip = cam.get_near_far().near_plane;
		let far_clip = cam.get_near_far().far_plane;
		let clip_range = far_clip - near_clip;

		let min_z = near_clip;
		let max_z = near_clip + clip_range;

		let range = max_z - min_z;
		let ratio = max_z / min_z;

        let lambda = 0.95; //TODO get from settings

		// Calculate split depths based on view camera furstum
		// Based on method presentd in https://developer.nvidia.com/gpugems/GPUGems3/gpugems3_ch10.html
		for i in 0..4 {
			let p = (i as f32 + 1.0) / 4.0;
			let log = min_z * ratio.powf(p);
			let uniform = min_z + range * p;
			let d = lambda * (log - uniform) + uniform;
			cascade_splits[i] = (d - near_clip) / clip_range;
		}

		// Calculate orthographic projection matrix for each cascade
		let mut last_split_dist = 0.0;
		for i in 0..4 {
			let split_dist = cascade_splits[i];

			let mut frustum_corners = [
				Vector3::new(-1.0,  1.0, -1.0),
				Vector3::new( 1.0,  1.0, -1.0),
				Vector3::new( 1.0, -1.0, -1.0),
				Vector3::new(-1.0, -1.0, -1.0),
				Vector3::new(-1.0,  1.0,  1.0),
				Vector3::new( 1.0,  1.0,  1.0),
				Vector3::new( 1.0, -1.0,  1.0),
				Vector3::new(-1.0, -1.0,  1.0),
			];

			// Project frustum corners into world space
			let inv_cam = cam.get_view_projection_matrix().invert().expect("failed to invers cam");
			for i in 0..8 {
				let inv_corner = inv_cam * frustum_corners[i].extend(1.0);
				frustum_corners[i] = inv_corner.truncate() / inv_corner.w;
			}

			for i in 0..4 {
				let dist = frustum_corners[i + 4] - frustum_corners[i];
				frustum_corners[i + 4] = frustum_corners[i] + (dist * split_dist);
				frustum_corners[i] = frustum_corners[i] + (dist * last_split_dist);
			}

			// Get frustum center
			let mut frustum_center = Vector3::new(0.0,0.0,0.0);
			for i in 0..8 {
				frustum_center += frustum_corners[i];
			}
			frustum_center /= 8.0;

			let mut radius: f32 = 0.0;
			for i in 0..8 {
				let distance: f32 = (frustum_corners[i] - frustum_center).magnitude();
				radius = radius.max(distance);
			}
			radius = (radius * 16.0).ceil() / 16.0;

			let max_extents = Vector3::new(radius,radius,radius);
			let min_extents = -max_extents;

			let light_dir = self.get_direction_vector(&rotation);
			let light_view_matrix = Matrix4::look_at(
                Point3::from_vec(frustum_center - light_dir * -min_extents.z),
                Point3::from_vec(frustum_center),
                Vector3::new(0.0, 1.0, 0.0)
            );

			let light_ortho_matrix = ortho(
                min_extents.x, max_extents.x,
                min_extents.y, max_extents.y,
                min_extents.z, max_extents.z - min_extents.z
            );

			// Store split distance and matrix in cascade
			return_depths[i] = (near_clip + split_dist * clip_range) * -1.0;
			proj_matrix[i] = light_ortho_matrix * light_view_matrix;

			last_split_dist = cascade_splits[i];
		}

        (proj_matrix, return_depths)
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

        let mut new_light = LightSpot{
            name: String::from(name),
            intensity: 1.0,
            color: Vector3::new(1.0, 1.0, 1.0),

            radius: 5.0,
            outer_radius: outer_radius,
            inner_radius: inner_radius,

            bound: collision::Aabb3::new(min, max),
        };

        new_light.rebuild_bound();
        new_light
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
        //let y_z_extend = self.outer_radius.sin() * radius;
        //TODO go from the max "left" to the max outer right...
        self.bound = collision::Aabb3::new(
            Point3::new(-radius, -radius, -radius),
            Point3::new(radius, radius, radius)//we can make the assumption that the spot light
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
