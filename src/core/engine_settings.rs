
use winit;
use vulkano;
use core::render_settings;

///Describes how the engine should handle debuging messages and vulkan settings
#[derive(Clone, PartialEq)]
pub enum BuildType {
    ///all debuging code is used.
    Debug,
    ///No debuging code is used.
    Release,
    ///Some special debuging messages are printed.
    ReleaseWithDebugMessages,
}

///Some global camera settings which are applyied to the currently active camera.
#[derive(Clone)]
pub struct CameraSettings{
    ///The max distance of the frustum from pov
    pub far_plane: f32,
    ///The min distance from pov
    pub near_plane: f32,
}


///The struc with the information
#[derive(Clone)]
pub struct EngineSettings {
    ///Displayed name
    pub app_name: String,
    ///The version of this application
    pub app_version: vulkano::instance::Version,

    ///Engine name
    pub engine_name: String,
    ///The version of the engine
    pub engine_version: vulkano::instance::Version,

    ///Dimensions in pixel
    pub window_dimensions: [u32; 2],
    ///location in pixel
    pub window_location: [u32; 2],
    ///Sets the visibility state of the cursor
    pub cursor_visible_state: winit::MouseCursor,
    ///Cursor state (i.e. normal, hidden "catched" etc.)
    pub cursor_state: winit::CursorState,
    ///flag to determin if the window should be created fullscreen
    pub fullscreen: bool,
    ///represents the monitor used for fullscreen mode
    pub main_monitor: i32,


    ///Debug settings:
    pub build_mode: BuildType,

    pub capture_frame: bool,

    ///Graphics settings:
    pub render_settings: render_settings::RenderSettings,

    ///Max iterations for the input polling per second
    pub max_input_speed: u32,

    ///The camera settings
    pub camera: CameraSettings,

}

impl EngineSettings{
    /// Creates a `EngineSettings` with default values.
    /// You can change some of them like this at creation time:
    /// # Examples
    ///  ```
    /// use jakar-engine::core::engine_settings;
    ///
    /// let settings = core::engine_settings::EngineSettings::new()
    ///     .with_dimensions(800, 600)
    ///     .with_name("Teddy the bear")
    ///     ;
    ///  ```
    pub fn default() -> Self{
        EngineSettings{
            //main
            app_name: String::from("Jakar-Engine"),
            ///The version of this application
            app_version: vulkano::instance::Version{
                major: 0,
                minor: 1,
                patch: 0,
            },

            ///Engine name
            engine_name: String::from("Jakar-Engine"),
            ///The version of the engine
            engine_version: vulkano::instance::Version{
                major: 0,
                minor: 1,
                patch: 0,
            },

            //window
            window_dimensions: [800, 600],
            window_location: [100, 100],
            cursor_visible_state: winit::MouseCursor::NoneCursor,
            cursor_state: winit::CursorState::Grab,
            fullscreen: false,
            main_monitor: 0,
            //graphics debuging
            build_mode: BuildType::Debug,
            //should usally not capture the frame
            capture_frame: false,
            //Graphics settings
            render_settings: render_settings::RenderSettings::default(),

            max_input_speed: 200,

            camera: CameraSettings{
                far_plane: 100.0,
                near_plane: 1.0,
            }
        }
    }



    ///Adds custom render settings to self
    pub fn with_render_settings(mut self, settings: render_settings::RenderSettings) -> Self{
        self.render_settings = settings;
        self
    }

    ///Returns the current render settings
    pub fn get_render_settings(&self) -> &render_settings::RenderSettings {
        &self.render_settings
    }

    ///Returns the current render settings, but mutable
    pub fn get_render_settings_mut(&mut self) -> &mut render_settings::RenderSettings {
        &mut self.render_settings
    }

    ///Sets the camera settings to `new`. Keep in mind that you can get "z-fighting" if the difference
    /// between near and far plane is too big.
    pub fn with_camera_settings(mut self, new: CameraSettings) -> Self{
        self.camera = new;
        self
    }

    ///see `with_camera_settings()`
    pub fn set_camera_settings(mut self, new: CameraSettings){
        self.camera = new;
    }


    ///Can be turned on, if so, the engine prints render infos, like time needed for ... for the next frame
    pub fn capture_next_frame(&mut self){
        self.capture_frame = true;
    }

    ///Can be used to turn of the capturing, is used anyways after caturing one frame
    pub fn stop_capture(&mut self){
        self.capture_frame = false;
    }

    ///Sets the main monitor, used to define where the fullscreen mode has to be applied
    #[inline]
    pub fn with_main_monitor(mut self, id: i32) -> Self{
        self.main_monitor = id;
        self
    }
    ///Sets the fullscreen mode (true = fullscreen)
    #[inline]
    pub fn with_fullscreen_mode(mut self, mode: bool) -> Self{
        self.fullscreen = mode;
        self
    }

    ///Sets a new visibility state for the cursor
    #[inline]
    pub fn with_cursor_visibility(mut self, state: winit::MouseCursor) -> Self{
        self.cursor_visible_state = state;
        self
    }

    ///sets the cursor state, most usefull is a free or a crapped cursor
    #[inline]
    pub fn with_cursor_state(mut self, state: winit::CursorState) -> Self{
        self.cursor_state = state;
        self
    }

    ///Sets the dimensions of `self` to `width` and `height`
    #[inline]
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self{
        self.window_dimensions = [width, height];
        self
    }

    ///Sets max iterations per second of the input polling.
    #[inline]
    pub fn with_max_input_polling_speed(mut self, new: u32) -> Self{
        self.max_input_speed = new;
        self
    }

    ///Sets the Location of `self` to `width` and `height`
    #[inline]
    pub fn at_location(mut self, width: u32, height: u32) -> Self{
        self.window_location = [width, height];
        self
    }

    ///Sets the engine mode to "release"
    #[inline]
    pub fn in_release_mode(mut self) -> Self{
        self.build_mode = BuildType::Release;
        self
    }

    ///Returns true if vulkan should be silent
    #[inline]
    pub fn vulkan_silence(&self) -> bool{
        match self.build_mode{
            BuildType::Release => true,
            _ => false,
        }
    }

    ///Sets the name of this settings
    #[inline]
    pub fn with_name(mut self, name: &str) -> Self{
        self.app_name = String::from(name);
        self
    }

    ///Sets the application version
    #[inline]
    pub fn with_app_version(mut self, major: u16, minor: u16, patch: u16) -> Self{
        self.app_version = vulkano::instance::Version{
            major: major,
            minor: minor,
            patch: patch,
        };
        self
    }

    ///Sets the dimensions of a currently used instance of `EngineSettings`
    #[inline]
    pub fn set_dimensions(&mut self, width: u32, height: u32){
        self.window_dimensions = [width, height];
    }

    ///Returns the dimensions
    #[inline]
    pub fn get_dimensions(&self) -> [u32; 2]{
        self.window_dimensions.clone()
    }



}
