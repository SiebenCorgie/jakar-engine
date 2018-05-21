
use cgmath::*;
use collision;
use std::f64::consts;
use std::sync::{Arc, Mutex};

use render::shader::shader_inputs::default_data;
use core::engine_settings::{EngineSettings,CameraSettings};
use input::keymap::KeyMap;
use core::next_tree::attributes::NodeAttributes;

use std::time::{Instant};


///Camera trait, use this to implement any type of camera
pub trait Camera {
    ///Creates a default camera
    fn new(settings: Arc<Mutex<EngineSettings>>, key_map: Arc<Mutex<KeyMap>>) -> Self;
    ///Initiates a camera with given opetions
    fn from_properties(
        settings: Arc<Mutex<EngineSettings>>,
        key_map: Arc<Mutex<KeyMap>>,
        transform: &Decomposed<Vector3<f32>, Quaternion<f32>>,
    ) -> Self;

    //Calculates / Update the view
    //fn update_view(&mut self);
    ///Updates the internal transform information to return teh correct results.
    fn update(&mut self, transform: &Decomposed<Vector3<f32>, Quaternion<f32>>);
    ///Returns the view matrix if needed
    fn get_view_matrix(&self) -> Matrix4<f32>;
    ///Returns Position used for view matrix calculation
    fn get_position(&self) -> Vector3<f32>;
    ///Sets Fov on this camera
    fn set_fov(&mut self, new_fov: f32);
    ///Returns the perspective matrix based on the window settings
    fn get_perspective(&self) -> Matrix4<f32>;
    ///Returns an view projection matrix which is corrected for vulkans view space
    fn get_view_projection_matrix(&self) -> Matrix4<f32>;
    ///Returns the bound of the view frustum
    fn get_frustum_bound(&self) -> collision::Frustum<f32>;
    ///Returns the current far/near plane settings used
    fn get_near_far(&self) -> CameraSettings;
    ///Returns the uniform data of this camera as an `default_data::ty::Data`. The transform field
    /// has to be set to an identity matrix.
    fn as_uniform_data(&self) -> default_data::ty::Data;
}


///An example implementation
#[derive(Clone)]
pub struct DefaultCamera {

    ///The node infromation used for the view matrix
    node_transform: Decomposed<Vector3<f32>, Quaternion<f32>>,

    direction: Vector3<f32>,
    view: Matrix4<f32>,
    projection: Matrix4<f32>,

    //Setting
    fov: f32,

    current_cam_settings: CameraSettings,

    settings: Arc<Mutex<EngineSettings>>,
}

///The Camera can use the opengl math beacuse be do
///```
///gl_Position.y = -gl_Position.y;
///```
///in every shader.
impl Camera for DefaultCamera{
    fn new(
        settings: Arc<Mutex<EngineSettings>>,
        key_map: Arc<Mutex<KeyMap>>
    ) -> Self {
        let fov = 45.0;

        let current_cam_settings = {
            let set_lck = settings.lock().expect("failed to load settings");
            set_lck.camera.clone()
        };

        DefaultCamera {

            node_transform: Decomposed{
                rot: Quaternion::from_angle_y(Deg(0.0)),
                disp: Vector3::new(0.0,0.0,0.0),
                scale: 0.0,
            },

            direction: Vector3::new(0.0,0.0,1.0),
            view: Matrix4::<f32>::identity(),
            projection: Matrix4::<f32>::identity(),

            fov: fov,

            current_cam_settings: current_cam_settings,
            settings: settings,
        }
    }

    ///Initiates a camera with given opetions
    fn from_properties(
        settings: Arc<Mutex<EngineSettings>>,
        key_map: Arc<Mutex<KeyMap>>,
        transform: &Decomposed<Vector3<f32>, Quaternion<f32>>,
    ) -> Self{

        let fov = 45.0;

        let current_cam_settings = {
            let set_lck = settings.lock().expect("failed to load settings");
            set_lck.camera.clone()
        };

        let mut new_cam = DefaultCamera {

            node_transform: Decomposed{
                rot: Quaternion::from_angle_y(Deg(0.0)),
                disp: Vector3::new(0.0,0.0,0.0),
                scale: 0.0,
            },

            direction: Vector3::new(0.0,0.0,1.0),
            view: Matrix4::<f32>::identity(),
            projection: Matrix4::<f32>::identity(),

            fov: fov,

            current_cam_settings: current_cam_settings,

            settings: settings,

        };
        new_cam.update(transform);
        new_cam
    }

    ///Updates the camera internal node transform to return the correct values for the view matrix.
    fn update(&mut self, transform: &Decomposed<Vector3<f32>, Quaternion<f32>>){
        //first update the view matrix
        let front = transform.rot.rotate_vector(Vector3::new(0.0,0.0,1.0));
        let tmp_target: Vector3<f32> = transform.disp + front;
        let view = Matrix4::look_at(
            Point3::new(transform.disp.x, transform.disp.y, transform.disp.z),
            Point3::new(tmp_target.x, tmp_target.y, tmp_target.z),
            Vector3::new(0.0,1.0,0.0)
        );

        self.view = view;
        self.direction = front;

        //now update the perspective as well
        let (width, height, near_plane, far_plane) = {
            let engine_settings_lck = self.settings.lock().expect("Faield to lock settings");
            (
                engine_settings_lck.get_dimensions()[0],
                engine_settings_lck.get_dimensions()[1],
                engine_settings_lck.camera.near_plane,
                engine_settings_lck.camera.far_plane
            )
        };

        //from https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
        /*
        let bias: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, -1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.5,
            0.0, 0.0, 0.0, 1.0
        );
        */
        //bias has to be multiplied to comply with the opengl -> vulkan coorinate system
        //(+y is down and depth is -1.0 - 1.0)
        //bias *
        let mut pers = perspective(
            Deg(self.fov),
            (width as f32 / height as f32),
            near_plane,
            far_plane
        );
        pers[1][1] *= -1.0;
        //pers = bias * pers;
        self.projection = pers;
    }


/*
    ///Updates the camera view information
    fn update_view(&mut self){

        //first check the current extends... TODO don't lock so often
        self.current_cam_settings = {
            let engine_settings_lck = self.settings.lock().expect("Faield to lock settings");
            engine_settings_lck.camera.clone()
        };

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
            if key_map_inst.q == true {
                self.position = self.position - Vector3::new(0.0, camera_speed, 0.0);
            }
            if key_map_inst.e == true {
                self.position = self.position + Vector3::new(0.0, camera_speed, 0.0);
            }
        }

        let sensitivity = 20.0;

        //Fixed camera gittering by slowing down so one integer delta = movement of
        // delta * sensitvity * time_delta * slowdown (virtual speed up)
        let virtual_speedup = 1.0; //currently not used because of the new float delta
        let x_offset: f32 = key_map_inst.mouse_delta_x as f32 * sensitivity * delta_time * virtual_speedup;
        let y_offset: f32 = -1.0 * key_map_inst.mouse_delta_y as f32 * sensitivity * delta_time * virtual_speedup; //reversed because of opengl style calculation
        //needed to exchange these beacuse of the z-is-up system
        self.yaw += x_offset;
        self.pitch += y_offset;

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
*/
    //Return view matrix as [[f32; 4]; 4]
    fn get_view_matrix(&self) -> Matrix4<f32> {
        self.view
    }

    ///Returns an 4x4 matrix containing view and projection
    fn get_view_projection_matrix(&self) -> Matrix4<f32>{
        let view = self.get_view_matrix();
        let projection = self.get_perspective();
        projection * view
    }

    ///Returns the position of the camera as Vector3<f32>
    #[inline]
    fn get_position(&self) -> Vector3<f32> {
        self.node_transform.disp
    }

    ///Sets the field of view for this camera
    #[inline]
    fn set_fov(&mut self, new_fov: f32){
        self.fov = new_fov;
    }

    //Calculates the perspective based on the engine and camera settings
    fn get_perspective(&self) -> Matrix4<f32>{
        self.projection
    }

    ///Returns the frustum bound of this camera as a AABB
    #[inline]
    fn get_frustum_bound(&self) -> collision::Frustum<f32>{
        let matrix = self.get_perspective() * self.get_view_matrix();
        collision::Frustum::from_matrix4(matrix).expect("failed to create frustum")
    }

    fn get_near_far(&self) -> CameraSettings{
        self.current_cam_settings.clone()
    }

    fn as_uniform_data(&self) -> default_data::ty::Data{

        let cam_near_far = self.get_near_far();

        let uniform_data = default_data::ty::Data {
            //Updating camera from camera transform
            camera_position: self.node_transform.disp.clone().into(),
            _dummy0: [0; 4],
            //This is getting a dummy value which is updated right bevore set creation via the new
            //model provided transform matrix. There might be a better way though.
            model: Matrix4::<f32>::identity().into(),
            view: self.get_view_matrix().into(),
            proj: self.get_perspective().into(),
            near: cam_near_far.near_plane,
            far: cam_near_far.far_plane,
        };

        uniform_data
    }

}
