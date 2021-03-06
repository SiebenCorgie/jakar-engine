
///Holds many useful information for different kinds of information
pub mod engine_settings;
///Is a sub category for the engine settings, it handles only the rendering specific settings
/// like msaa sample count, rendering distance etc.
pub mod render_settings;
///the resource manager is a abstraction to group al management systems together, it is
///not an actuall system
pub mod resource_management;
///Resources holds all loadable resources, they should ussually be managed though one
///of the management systems in `core::resource_management`.
pub mod resources;
///A new tree system which will replace the `simple_scene_system`.
pub mod next_tree;

//use std::sync::{Arc, Mutex};
use cgmath::*;
use collision::*;

///If a object implements this trait, it can be used for any bound operation in the engine
pub trait ReturnBoundInfo {
    ///return the max size of its bound
    fn get_bound_max(&self)-> Point3<f32>;
    ///return the min size of its bound
    fn get_bound_min(&self)-> Point3<f32>;
    ///Returns a copy of its bound
    fn get_bound(&self) -> Aabb3<f32>;
    ///Sets the bound to the new values (in mesh space)
    fn set_bound(&mut self, min: Point3<f32>, max: Point3<f32>);
    ///Returns the vertices of the bounding mesh, good for debuging
    fn get_bound_points(&self)-> Vec<Vector3<f32>>;
    ///Rebuilds the bound based on for instance `intensity` of an light.
    fn rebuild_bound(&mut self);
}


///tmp trait for intersction
pub trait AABB3Intersection {
    fn intersects(&self, other: &Aabb3<f32>) -> bool;
    fn half_extend(&self) -> Vector3<f32>;
}

impl AABB3Intersection for Aabb3<f32>{
    fn intersects(&self, other: &Aabb3<f32>) -> bool{
        let (a0, a1) = (self.center(), self.half_extend());
        let (b0, b1) = (other.center(), other.half_extend());

        let x = (a0.x - b0.x) <= (a1.x + b1.x);
        let y = (a0.y - b0.y) <= (a1.y + b1.y);
        let z = (a0.z - b0.z) <= (a1.z + b1.z);

        if x && y && z{
            return true;
        }else{
            return false;
        }

        //a1.x > b0.x && a0.x < b1.x && a1.y > b0.y && a0.y < b1.y
    }

    fn half_extend(&self) -> Vector3<f32>{
        let mins = self.min();
        let maxs = self.max();

        let x = (maxs.x - mins.x) / 2.0;
        let y = (maxs.y - mins.y) / 2.0;
        let z = (maxs.z - mins.z) / 2.0;

        Vector3::new(x, y, z)
    }
}

///A cheating trait to convert points into vectors
pub trait PointToVector<V> {
    fn into_vec(&self) -> V;
}

impl PointToVector<Vector3<f32>> for Point3<f32>{
    fn into_vec(&self) -> Vector3<f32>{
        Vector3::new(
            self.x,
            self.y,
            self.z
        )
    }
}
