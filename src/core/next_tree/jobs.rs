use cgmath::*;


///The main jobs a scene tree can perform on a node
pub enum SceneJobs {
    Move(Vector3<f32>),
    Rotate(Vector3<f32>),
    Scale(Vector3<f32>),
    
}
