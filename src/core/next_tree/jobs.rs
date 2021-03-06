use cgmath::*;


///The main jobs a scene tree can perform on a node
#[derive(Clone)]
pub enum SceneJobs {
    ///Moves the node along the vector.
    Move(Vector3<f32>),
    ///Rotates the node via euler angles from this vector (x,y,z).
    Rotate(Vector3<f32>),
    ///Adds a quaternion rotation to the current rotation
    RotateQ(Quaternion<f32>),
    ///Rotates the node around a given point.
    ///The first element is the rotation, the second element is the point.
    RotateAroundPoint(Vector3<f32>, Vector3<f32>),
    ///Scales the object by this x,y and z values.
    /// Currently only uniform scale (based on x) is suported
    ///TODO: implement non uniform scale.
    Scale(Vector3<f32>),
    /*TODO
    add setter job to "set_location" etc.
    */
}
