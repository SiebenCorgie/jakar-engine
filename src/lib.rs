///The engines top level

extern crate cgmath;
extern crate collision;
//extern crate assimp; Put the assimp crate out because of legacy and compile problems

//All thrid party crates
extern crate winit;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate time;
extern crate image;
extern crate gltf;
extern crate gltf_importer;
extern crate gltf_utils;

///The engine core defines most functions and
///traits needed to feed the renderer and communicate with the physics.
///It also mamanges the scene tree and how to get specific information out of it
pub mod core;
///The engines renderer currently WIP
pub mod render;
///A collection of helpfull tools for integration of data with the engine
pub mod tools;
///A small thread who will run and administrate the winit window, as well as its input
///processing
pub mod input;

use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;



///An builder object which can be used to create an engine with specific settings
pub struct EngineBuilder {
    pub settings: core::engine_settings::EngineSettings,
}


impl EngineBuilder {

    //Creates an new `EngineBuilder` if not changed, the engine can be started with default settings
    pub fn new() -> Self{

        let settings = Arc::new(Mutex::new(core::engine_settings::EngineSettings::new()
        .with_dimensions(1600, 900)
        .with_name("jakar Instance")
        .set_vulkan_silent()
        .with_fullscreen_mode(false)
        .with_cursor_state(winit::CursorState::Grab)
        .with_cursor_visibility(winit::MouseCursor::NoneCursor)
        .with_msaa_factor(4)
        ));


        EngineBuilder{
            settings: core::engine_settings::EngineSettings::new(),
        }
    }

    ///Starts the engine which will create the following sub systems in their own threads:
    ///
    /// - Renderer
    ///     - Pipeline manager
    ///     - Uniform manager
    ///     - Window system
    ///
    /// - Asset manager
    ///     - Material manager
    ///     - Mesh manager
    ///     - Scene manager
    ///     - Texture manager
    /// - Input system
    pub fn start(mut self) -> JakarEngine{
        //Input

        //Create the settings
        let arc_settings = Arc::new(Mutex::new(self.settings));

        let mut input_handler = Arc::new(
            Mutex::new(
                input::Input::new(arc_settings.clone()).with_polling_speed(60)
            )
        );



        //Create a renderer with the input system
        let render = {
            let mut input_sys = input_handler.lock().expect("failed to lock input system before start");
            Arc::new(
                Mutex::new(
                    render::renderer::Renderer::new(
                        (*input_sys).get_events_loop(),
                        arc_settings.clone(),
                        (*input_sys).get_key_map(),
                    )
                )
            )
        };

        //Create a asset manager for the renderer
        let mut asset_manager = {
            let input_sys = input_handler.lock().expect("failed to lock input system before start");

            Arc::new(
                Mutex::new(
                    core::resource_management::asset_manager::AssetManager::new(
                        render.clone(),
                        arc_settings.clone(),
                        (*input_sys).key_map.clone()
                    )
                )
            )
        };


        //Start the input thread
        {
            let mut input_sys = input_handler.lock().expect("failed to lock input system before start");

            //Start the input thread
            (*input_sys).start();
        }



        JakarEngine{

            renderer: render,
            asset_manager: asset_manager,
            input_system: input_handler,

            engine_settings: arc_settings,
        }
    }

    ///Set the settings used to create the engine to `settings`
    pub fn with_settings(mut self, settings: core::engine_settings::EngineSettings) -> Self{
        self.settings = settings;
        self
    }

    ///Returns a reference to the current settings instance which then can be manipulated
    pub fn get_settings(&mut self) -> core::engine_settings::EngineSettings{
        self.settings.clone()
    }
}


///An struct representing the top level of this engine
///
///It is responsible for handling all sub systems of the engine as well as providing an API to
/// the user which can be used to manipulate data
pub struct JakarEngine {
    ///The renderer
    pub renderer: Arc<Mutex<render::renderer::Renderer>>,
    pub asset_manager: Arc<Mutex<core::resource_management::asset_manager::AssetManager>>,
    pub input_system: Arc<Mutex<input::Input>>,

    pub engine_settings: Arc<Mutex<core::engine_settings::EngineSettings>>,

}

///Implements the main functions for the engine. Other functionality can be imported in scope
///via traits
impl JakarEngine {
    ///Updates the engine
    pub fn update(&mut self){
        self.get_asset_manager().update();
    }

    ///Returns the asset manager
    pub fn get_asset_manager<'a>(&'a mut self) -> MutexGuard<'a, core::resource_management::asset_manager::AssetManager>{
        let asset_lock = self.asset_manager.lock().expect("failed to lock asset manager");
        asset_lock
    }

    ///Returns the renderer
    pub fn get_renderer<'a>(&'a mut self) -> MutexGuard<'a, render::renderer::Renderer>{
        let render_lock = self.renderer.lock().expect("failed to lock asset manager");
        render_lock
    }
}






/*TODO
3rd Render on main thread, manage materials on event on different thread,
manage objects on secondary thread, manage loading on n-threads (per object?)
5th create a high level fn collection for adding and removing things from the scene tree
6th build a simple forward renderer with vulkano
7th make core, render and later physics independend threads //NOTE Done in 3.1-3.4 ?
10th shadow generation?

9th CREATE A FLUFFY TILED RENDERER WITH ALL THE STUFF
10th PBR ?
11th Editor and templates https://www.youtube.com/watch?v=UWacQrEcMHk , https://www.youtube.com/watch?annotation_id=annotation_661107683&feature=iv&src_vid=UWacQrEcMHk&v=xYiiD-p2q80 , https://www.youtube.com/watch?annotation_id=annotation_2106536565&feature=iv&src_vid=UWacQrEcMHk&v=yIedljapuz0
*/


/*TODO
*/

//Some Helper functions
///Returns an runtime error
pub fn rt_error(location: &str, content: &str){
    println!("ERROR AT: {} FOR: {}", location, content);
}
