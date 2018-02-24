
use winit;
use vulkano_win;
use vulkano_win::VkSurfaceBuild;
use vulkano;
use std::sync::{Arc, Mutex};

use core::engine_settings;


///Controlles a window created with the renderer
pub struct Window {
    window: vulkano_win::Window,
}


impl Window{
    pub fn new(instance: &Arc<vulkano::instance::Instance>,
        events_loop: &winit::EventsLoop,
        engine_settings: Arc<Mutex<engine_settings::EngineSettings>>
    )-> Self{

        let mut available_monitors = winit::get_available_monitors();

        let mut engine_settings_lck = engine_settings.lock().expect("Failed to lock engine settings");
        let mut window_builder = winit::WindowBuilder::new();

        //do not specifiy screen dimensions when creating with fullscreen
        //Set fullscreen if needed
        if engine_settings_lck.fullscreen{
            let valid_monitor_id = {
                match available_monitors.nth(engine_settings_lck.main_monitor as usize){
                    Some(monitor) => monitor,
                    None => {
                        //The monitor id in the settings is not valid, trying to get the 0th
                        //one, if this fails then there is no monitor and we have to panic :(
                        match available_monitors.nth(0){
                            Some(monitor_id) => monitor_id,
                            None => panic!("could not find monitor for this system!"),
                        }
                    },
                }
            };
            //After getting a vaild monitor id, returning if for the fullscreen
            window_builder = window_builder.with_fullscreen(valid_monitor_id);
            //beacuse we are in fullscreen, we can overwerite the dimensions in the settings
        }else{
            //is not fullscreen, so we set up a window with dimensions
            window_builder = window_builder.with_dimensions(
                engine_settings_lck.get_dimensions()[0],
                engine_settings_lck.get_dimensions()[1]
            );
        }
        //set some global info for the builder
        window_builder = window_builder
        .with_title(engine_settings_lck.app_name.clone())
        .with_decorations(true);

        //build the vulkano_win window
        let window = window_builder
        .build_vk_surface(events_loop, instance.clone()).expect("failed to create window!");

        //Set the cursor state (can only be done on a already created window)
        window.window().set_cursor(engine_settings_lck.cursor_visible_state);
        window.window().set_cursor_state(engine_settings_lck.cursor_state).ok().expect("could not set cursor");
        //now update the engine settings with the actual size
        match window.window().get_inner_size_pixels(){
            Some(dims) =>{
                engine_settings_lck.window_dimensions = [dims.0, dims.1];
            },
            None => {}, //don't do anything something did'nt work
        }



        Window{
            window: window,
        }
    }

    ///Returns the window surface
    #[inline]
    pub fn surface(&mut self) -> &Arc<vulkano::swapchain::Surface> {
        self.window.surface()
    }

    ///Returns the window component
    #[inline]
    pub fn window(&mut self) -> &winit::Window{
        self.window.window()
    }

    ///Returns the current extend of the window vk_surface, returns [100,100] if something went wrong.
    pub fn get_current_extend(&self) -> [u32; 2]{
        match self.window.window().get_inner_size_pixels(){
            Some(dims) =>{
                [dims.0, dims.1]
            },
            None => {[100, 100]}, //return fallbacks
        }
    }
}
