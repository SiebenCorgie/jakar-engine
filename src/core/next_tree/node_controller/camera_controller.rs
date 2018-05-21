use input::keymap::KeyMap;
use jakar_tree::node::{NodeController, Node};

use core::next_tree::content::ContentType;
use core::next_tree::jobs::SceneJobs;
use core::next_tree::attributes::NodeAttributes;
use core::resources::camera::Camera;
use tools::math::time_tools::*;
use cgmath::*;

use std::sync::{Arc, Mutex};
use std::time::Instant;

///Reads the current keymap and rotates the node based on the input. Will also call the
///Update function of an camera if one is set.
pub struct CameraController {
    key_map: Arc<Mutex<KeyMap>>,
    //Camera_speed
    camera_speed: f32,
    sensitivity: f32,
    //last update time
    last_update: Instant,
}


impl CameraController{
    pub fn new(key_map: Arc<Mutex<KeyMap>>) -> Self{
        let front = Vector3::new(0.0, 0.0, -1.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        CameraController{
            key_map,
            camera_speed: 2.0,
            sensitivity: 20.0,
            last_update: Instant::now()

        }
    }
}

impl NodeController<ContentType,SceneJobs,NodeAttributes> for CameraController
{
    fn update(&mut self, node: &mut Node<ContentType,SceneJobs,NodeAttributes>){
        let current_keys = {
            let map_lck = self.key_map.lock().expect("failed to lock keymap");
            (*map_lck).clone()
        };

        let old_rot = node.get_attrib().transform.rot.clone();
        let current_front = old_rot.rotate_vector(Vector3::new(0.0,0.0,1.0));
        let current_right = old_rot.rotate_vector(Vector3::new(1.0,0.0,0.0));

        //first calculate the offset of the position
        let mut position: Vector3<f32> = Vector3::new(0.0,0.0,0.0);
        let delta = dur_as_f32(self.last_update.elapsed());
        let this_speed = self.camera_speed * delta;

        if current_keys.a == true {
            position += current_right * this_speed;
        }
        if current_keys.w == true {
            position += current_front * this_speed;
        }
        if current_keys.s == true {
            position -= current_front * this_speed;
        }
        if current_keys.d == true {
            position -= current_right * this_speed;
        }
        if current_keys.q == true {
            position = position - Vector3::new(0.0, this_speed, 0.0);
        }
        if current_keys.e == true {
            position = position + Vector3::new(0.0, this_speed, 0.0);
        }

        //Fixed camera gittering by slowing down so one integer delta = movement of
        // delta * sensitvity * time_delta * slowdown (virtual speed up)
        let yaw_x_offset: f32 = -1.0 * current_keys.mouse_delta_x as f32 * self.sensitivity * delta;
        let pitch_y_offset: f32 =  current_keys.mouse_delta_y as f32 * self.sensitivity * delta; //reversed because of opengl style calculation

        //First rotate around the world up axis (y)
        let rot_yaw = Quaternion::from_angle_y(Deg(yaw_x_offset));
        //now rotate around the current pitch axis which is the transformed pitch
        let rot_pitch = Quaternion::from_axis_angle(current_right, Deg(pitch_y_offset));

        //now construct a quaterinion from both rotations
        let rot = rot_pitch * rot_yaw;

        //we got position change and rotation change, lets rotate our node and move it
        node.add_job(SceneJobs::RotateQ(rot));
        node.add_job(SceneJobs::Move(position));

        //if the node has an camera object, update it
        let tranform = node.get_attrib().transform;

        if let ContentType::Camera(ref mut camera) = node.get_value_mut(){
            camera.update(&tranform);
        }

        self.last_update = Instant::now();
    }
}
