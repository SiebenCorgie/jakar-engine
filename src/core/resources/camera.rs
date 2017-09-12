
use cgmath::*;
use collision;
use std::f64::consts;
use std::sync::{Arc, Mutex};

use core::engine_settings;
use input::KeyMap;

use std::time::{Instant};

///Camera trait, use this to implement any type of camera
pub trait Camera {
    ///Creates a default camera
    fn new(settings: Arc<Mutex<engine_settings::EngineSettings>>, key_map: Arc<Mutex<KeyMap>>) -> Self;
    ///Initiates a camera with given opetions
    fn from_properties(
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        key_map: Arc<Mutex<KeyMap>>,
        front: Vector3<f32>,
        position: Vector3<f32>,
        up: Vector3<f32>,
        yaw: f32,
        pitch: f32,
        speed: f32,
    ) -> Self;
    ///Calculates / Update the view
    fn update_view(&mut self);
    ///Returns the view matrix if needed
    fn get_view_matrix(&self) -> Matrix4<f32>;
    ///Returns the current direction of the camera
    fn get_direction(&self) -> Vector3<f32>;
    ///Set current direction
    fn set_direction(&mut self, new_direction: Vector3<f32>);
    ///Returns Position
    fn get_position(&self) -> Vector3<f32>;
    ///Set current position
    fn set_position(&mut self, new_pos: Vector3<f32>);
    ///Sets Fov on this camera
    fn set_fov(&mut self, new_fov: f32);
    ///Sets the far, and near planes of the frustum
    fn set_frustum_planes(&mut self, near: f32, far: f32);
    ///Returns the perspective matrix based on the window settings
    fn get_perspective(&self) -> Matrix4<f32>;
    ///Returns the bound of the view frustum
    fn get_frustum_bound(&self) -> collision::Frustum<f32>;
}

///An example implementation
#[derive(Clone)]
pub struct DefaultCamera {
    //camera General
    pub position: Vector3<f32>,
    pub front: Vector3<f32>,
    pub up: Vector3<f32>,
    pub right: Vector3<f32>,
    pub world_up: Vector3<f32>,
    //Camera Rotation
    yaw: f32,
    pitch: f32,

    //Setting
    fov: f32,
    near_plane: f32,
    far_plane: f32,

    speed: f32,

    settings: Arc<Mutex<engine_settings::EngineSettings>>,
    key_map: Arc<Mutex<KeyMap>>,

    last_time: Instant,
}


impl Camera for DefaultCamera{
    fn new(
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        key_map: Arc<Mutex<KeyMap>>
    ) -> Self {
        //camera General
        let position = Vector3::new(0.0, 0.0, 0.0);
        let front = Vector3::new(0.0, 0.0, -1.0);
        let up = Vector3::new(0.0, -1.0, 0.0);
        let world_up = Vector3::new(0.0, -1.0, 0.0);

        //Camera Rotation
        let yaw: f32 = -90.0;
        let pitch: f32 = 0.0;

        let fov = 45.0;
        let near_plane = 0.1;
        let far_plane = 100.0;

        DefaultCamera {
            position: position,
            front: front,
            up: up,
            right: front.cross(up),
            world_up: up,

            yaw: yaw,
            pitch: pitch,
            fov: fov,

            near_plane: near_plane,
            far_plane: far_plane,

            speed: 25.0,

            settings: settings,

            key_map: key_map,

            last_time: Instant::now(),
        }
    }

    ///Initiates a camera with given opetions
    fn from_properties(
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        key_map: Arc<Mutex<KeyMap>>,
        front: Vector3<f32>,
        position: Vector3<f32>,
        up: Vector3<f32>,
        yaw: f32,
        pitch: f32,
        speed: f32,
    ) -> Self{

        let fov = 45.0;
        let near_plane = 0.1;
        let far_plane = 100.0;

        let mut new_cam = DefaultCamera {
            position: position,
            front: front,
            up: up,
            right: front.cross(up),
            world_up: up,

            yaw: yaw,
            pitch: pitch,
            fov: fov,
            near_plane: near_plane,
            far_plane: far_plane,

            speed: speed,

            settings: settings,

            key_map: key_map,

            last_time: Instant::now(),
        };

        new_cam.update_view();
        new_cam
    }

    ///Updates the camera view information
    fn update_view(&mut self){

        let delta_time: f32 ={
            //Get the time and / 1_000_000_000 for second
            (self.last_time.elapsed().subsec_nanos()) as f32
            /
            1_000_000_000.0
        };
        //and update "last time" for the next frame
        self.last_time = Instant::now();

        //println!("Delta_Seconds: {}", delta_time.clone() );

        //Corrected Camera Speed
        let camera_speed = 2.0 * delta_time;

        //copy us a easy key map
        let key_map_inst = {
            let glob_key_map_lck = self.key_map
            .lock()
            .expect("failed to lock global key map");

            let return_key_map = (*glob_key_map_lck).clone();
            return_key_map
        };

        //Input processing
        //some are flipped because in vulkan the upper_left corener is -1/-1 not -1/1 like in opengl
        {
            if key_map_inst.a == true {
                self.position -= self.right * camera_speed;
            }
            if key_map_inst.w == true {
                self.position += self.front * camera_speed;
            }
            if key_map_inst.s == true {
                self.position -= self.front * camera_speed;
            }
            if key_map_inst.d == true {
                self.position += self.right * camera_speed;
            }
            if (key_map_inst.ctrl_l == true) | (key_map_inst.q == true) {
                self.position = self.position - Vector3::new(0.0, camera_speed, 0.0);
            }
            if (key_map_inst.shift_l == true) | (key_map_inst.e == true) {
                self.position = self.position + Vector3::new(0.0, camera_speed, 0.0);
            }
        }

        let sensitivity = 20.0;

        //Fixed camera gittering by slowing down so one integer delta = movement of
        // delta * sensitvity * time_delta * slowdown (virtual speed up)
        let virtual_speedup = 1.0; //currently not used because of the new float delta
        let x_offset: f32 = key_map_inst.mouse_delta_x as f32 * sensitivity * delta_time * virtual_speedup;
        let y_offset: f32 = key_map_inst.mouse_delta_y as f32 * sensitivity * delta_time * virtual_speedup;
        //needed to exchange these beacuse of the z-is-up system
        self.yaw -= x_offset;
        self.pitch -= y_offset;

        if self.pitch > 89.0 {
            self.pitch = 89.0;
        }
        if self.pitch < -89.0 {
            self.pitch = -89.0;
        }

        let mut front = Vector3::new(0.0, 0.0, 0.0);
        front.x = to_radians(self.yaw).cos() * to_radians(self.pitch).cos();
        front.y = to_radians(self.pitch).sin();
        front.z =  to_radians(self.yaw).sin() * to_radians(self.pitch).cos();
        self.front = front.normalize();

        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();

    }

    //Return view matrix as [[f32; 4]; 4]
    fn get_view_matrix(&self) -> Matrix4<f32> {

        let tmp_target = self.position + self.front;

        let view = Matrix4::look_at(
            Point3::new(self.position.x, self.position.y, self.position.z),
            Point3::new(tmp_target.x, tmp_target.y, tmp_target.z),
            self.up
        );
        view
    }

    ///Returns the direction the camera is facing
    fn get_direction(&self) -> Vector3<f32> {
        self.front
    }

    ///Sets the direction of the camera to a Vector3<f32>
    fn set_direction(&mut self, new_direction: Vector3<f32>){
        self.front = new_direction.normalize();
    }

    ///Returns the position of the camera as Vector3<f32>
    fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    ///Sets the position
    fn set_position(&mut self, new_pos: Vector3<f32>){
        self.position = new_pos;
    }

    ///Sets the field of view for this camera
    fn set_fov(&mut self, new_fov: f32){
        self.fov = new_fov;
    }

    ///Sets the frustum far and near plane
    fn set_frustum_planes(&mut self, near: f32, far: f32) {
        self.far_plane = far;
        self.near_plane = near;
    }

    //Calculates the perspective based on the engine and camera settings
    fn get_perspective(&self) -> Matrix4<f32>{
        //TODO update the perspective to use current engine settings
        let (width, height) = {
            let engine_settings_lck = self.settings.lock().expect("Faield to lock settings");

            (
                (*engine_settings_lck).get_dimensions()[0],
                (*engine_settings_lck).get_dimensions()[1]
            )
        };

        perspective(Deg(self.fov),
        (width as f32 / height as f32),
        self.near_plane, self.far_plane)
    }

    ///Returns the frustum bound of this camera as a AABB
    fn get_frustum_bound(&self) -> collision::Frustum<f32>{
        let matrix = self.get_perspective() * self.get_view_matrix();
        collision::Frustum::from_matrix4(matrix).expect("failed to create frustum")
    }
}

//Helper function for calculating the view
fn to_radians(degree: f32) -> f32 {
    degree * (consts::PI / 180.0) as f32
}
