use core::ReturnBoundInfo;
use cgmath::*;
use collision;
use collision::Aabb3;

#[derive(Clone)]
pub struct Empty {
    pub name: String,
    bound: Aabb3<f32>,
}

impl Empty{
    ///Returns an Empty with a 1x1x1 bound and `name` as name
    pub fn new(name: &str) -> Self{
        //Creating the box extend from the location, there might be a better way
        let min = Point3::new(0.5, 0.5, 0.5, );
        let max = Point3::new(0.5, 0.5, 0.5, );

        Empty{
            name: String::from(name),
            bound: Aabb3::new(min, max),
        }
    }
}

impl ReturnBoundInfo for Empty{
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

        self.bound = Aabb3::new(min, max);
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

    ///Doesn't change anything because the set bound is always right (an empty has no dimension)
    fn rebuild_bound(&mut self){
        //Does nothing
    }
}

/*
impl NodeMember for Empty{


    ///Returns the name of this node
    fn get_name(&self) -> String{
        self.name.clone()
    }

    ///Returns the type of node this is
    fn get_content_type(&mut self) -> node::ContentTag{
        node::ContentTag::Empty
    }
}
*/
