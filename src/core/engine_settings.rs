
use winit;
use vulkano;

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
    pub silent_vulkan: bool,

    ///Graphics settings:
    ///filtering options, should be power of two between 1 and 16
    pub anisotropic_filtering: f32,
    ///Samples for each pixel, should be power of two between 1 and 16 (but can be higher)
    pub msaa: u32,


    ///Max input pollings per second (default 60)
    pub max_input_speed: u32,
    ///Max asset updates per second (default 120)
    pub max_asset_updates: u32,
    ///Max frames per second in the rendering thread (default 700 fps)
    pub max_fps: u32,




}

impl EngineSettings{
    /// Creates a `EngineSettings` with default values.
    /// You can change some of them like this at creation time:
    /// # Examples
    ///  ```
    /// use ori-engine::core::engine_settings;
    ///
    /// let settings = core::engine_settings::EngineSettings::new()
    ///     .with_dimensions(800, 600)
    ///     .with_name("Teddy the bear")
    ///     .set_vulkan_silent()
    ///     ));
    ///  ```
    pub fn new() -> Self{



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
            silent_vulkan: false,
            //Graphics settings
            anisotropic_filtering: 1.0,
            msaa: 1,

            max_input_speed: 60,
            max_asset_updates: 120,
            max_fps: 700,
        }
    }

    ///Sets the maximum updates per second value for the asset thread.
    pub fn with_asset_update_speed(mut self, speed: u32) -> Self{
        self.max_asset_updates = speed;
        self
    }

    ///Sets the maximum polls per second value for the input thread.
    pub fn with_input_poll_speed(mut self, speed: u32) -> Self{
        self.max_input_speed = speed;
        self
    }

    ///Sets the maximum frames per second value for the render thread.
    pub fn with_fps(mut self, fps: u32) -> Self{
        self.max_fps = fps;
        self
    }

    ///Sets the main monitor, used to define where the fullscreen mode has to be applied
    pub fn with_main_monitor(mut self, id: i32) -> Self{
        self.main_monitor = id;
        self
    }
    ///Sets the fullscreen mode (true = fullscreen)
    pub fn with_fullscreen_mode(mut self, mode: bool) -> Self{
        self.fullscreen = mode;
        self
    }

    ///Sets a new visibility state for the cursor
    pub fn with_cursor_visibility(mut self, state: winit::MouseCursor) -> Self{
        self.cursor_visible_state = state;
        self
    }

    ///sets the cursor state, most usefull is a free or a crapped cursor
    pub fn with_cursor_state(mut self, state: winit::CursorState) -> Self{
        self.cursor_state = state;
        self
    }

    ///Sets up a custom anisotropical filtering factor
    pub fn with_anisotropical_filtering(mut self, af_factor: f32)-> Self{
        self.anisotropic_filtering = af_factor;
        self
    }

    ///Sets up a custom anisotropical filtering factor
    pub fn with_msaa_factor(mut self, msaa_factor: u32) -> Self{
        self.msaa = msaa_factor;
        self
    }


    /// Sets vulkan silent, vulkan won't print any validation layer infos anymore
    pub fn set_vulkan_silent(mut self) -> Self{
        self.silent_vulkan = true;
        self
    }
    ///returns the silent status of vulkan
    pub fn vulkan_silence(&self) -> bool{
        self.silent_vulkan.clone()
    }
    ///Sets the dimensions of `self` to `width` and `height`
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self{
        self.window_dimensions = [width, height];
        self
    }
    ///Sets the Location of `self` to `width` and `height`
    pub fn at_location(mut self, width: u32, height: u32) -> Self{
        self.window_location = [width, height];
        self
    }
    ///Sets the name of this settings
    pub fn with_name(mut self, name: &str) -> Self{
        self.app_name = String::from(name);
        self
    }

    ///Sets the application version
    pub fn with_app_version(mut self, major: u16, minor: u16, patch: u16) -> Self{
        self.app_version = vulkano::instance::Version{
            major: major,
            minor: minor,
            patch: patch,
        };
        self
    }

    ///Sets the dimensions of a currently used instance of `EngineSettings`
    pub fn set_dimensions(&mut self, width: u32, height: u32){
        self.window_dimensions = [width, height];
    }
    ///Returns the dimensions
    pub fn get_dimensions(&self) -> [u32; 2]{
        self.window_dimensions.clone()
    }
}
