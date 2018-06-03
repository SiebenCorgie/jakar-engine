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
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate image;
extern crate gltf;
extern crate gltf_importer;
extern crate gltf_utils;
//the new custom tree crate
extern crate jakar_tree;
//The threadpool implementation
extern crate jakar_threadpool;


///The engine core defines most functions and
///traits needed to feed the renderer and communicate with the physics.
///It also mamanges the scene tree and how to get specific information out of it
pub mod core;
///The engines renderer. Handels the drawin of one frame and manages resources on the gpu as well as
/// data which is needed to feed the gpu.
pub mod render;
use render::renderer::BuildRender;
///A collection of helpfull tools for integration of data with the engine
pub mod tools;
use tools::engine_state_machine::NextStep;
///A small thread who will run and administrate the winit window, as well as its input
///processing
pub mod input;

use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::*;

use std::result::Result;

///Holds possible engine creation errors
pub enum CreationErrors{
    ///Is returned when something went wrong while creating the renderer. The String holds a message.
    FailedToCreateRenderer(String),
    ///Is returned when there was a problem with the asset manager or thread creation
    FailedToCreateAssetManager,
    ///Is returned when the engine couldn't start the input loop.
    FailedToCreateInputManager(String),
    ///Is returned when something else happned.
    UnknownError,
}

///Describes the current status of the engine
#[derive(PartialEq)]
pub enum EngineStatus {
    ///Is used when starting normal
    STARTING,
    RUNNING,
    ///Is used when the engine is waiting for threads to end.
    WAITING,
    ENDING,
    ///Is used when something is going wrong while creating the engine runtime.
    Aboarding(String)
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
    pub engine_status: Arc<Mutex<EngineStatus>>,

    main_loop_thread: Option<JoinHandle<()>>,

    pub thread_pool: Arc<Mutex<jakar_threadpool::ThreadPool>>,

}

///Implements the main functions for the engine. Other functionality can be imported in scope
///via traits.
impl JakarEngine {
    ///Build the engine which will create the following sub systems:
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
    pub fn build(settings: Option<core::engine_settings::EngineSettings>) -> Result<Self, CreationErrors>{
        //first create the thread save engine settings and the engine status.
        //they are needed to start the input, asset and rendering thread.
        //Thoose will return their main features which will be an Arc<Mutex<T>> of the
        //rendering struct, the asset manager and the input handler. Later there will be an physics
        //handler as well.
        let engine_settings = {
            match settings{
                Some(s_settings) => Arc::new(Mutex::new(s_settings)),
                None =>{
                    //Creating default settings
                    let n_settings = Arc::new(
                        Mutex::new(
                            core::engine_settings::EngineSettings::default()
                        )
                    );
                    n_settings
                }
            }
        };
        let engine_status = Arc::new(Mutex::new(EngineStatus::STARTING));

        //Now, first of all start the rendering builder
        let mut render_builder = render::render_builder::RenderBuilder::new(engine_settings.clone());
        //Configure======================================
        {
            let settings = engine_settings.lock().expect("failed to lock render settings");
            //Check for the debug mode, if we are in debug mode, setup the layers
            match settings.build_mode{
                //Throw all messages
                core::engine_settings::BuildType::Debug => {
                    render_builder.layer_loading = render::render_builder::LayerLoading::All;
                    render_builder.vulkan_messages = vulkano::instance::debug::MessageTypes::errors_and_warnings();
                }
                //Throw only errors
                core::engine_settings::BuildType::ReleaseWithDebugMessages => {
                    render_builder.layer_loading = render::render_builder::LayerLoading::All;
                    render_builder.vulkan_messages = vulkano::instance::debug::MessageTypes::errors();
                }
                //Throw nothing
                core::engine_settings::BuildType::Release => {
                    render_builder.layer_loading = render::render_builder::LayerLoading::NoLayer;
                    render_builder.vulkan_messages = vulkano::instance::debug::MessageTypes::none();
                }
            }
        }
        //now start the instance
        match render_builder.create_instance(){
            Ok(_) => {},
            Err(error) => return Err(CreationErrors::FailedToCreateRenderer(error)),
        }

        //We are now read to start the input system.
        //if will create the system itself as well as an input handler thread, which will poll
        // events in a continues speed.
        let (input_system, window) = {
            let result  = input::Input::new(
                engine_settings.clone(),
                render_builder.get_instance(),
            );

            match result{
                Ok((inp_sy, window)) => (Arc::new(Mutex::new(inp_sy)), window),
                Err(er) => return Err(CreationErrors::FailedToCreateInputManager(er)),
            }
        };

        //Since we go the window now, we can build the renderer
        let renderer = {
            match render_builder.build(window){
                Ok(ren) => Arc::new(Mutex::new(ren)),
                Err(er) => return Err(CreationErrors::FailedToCreateRenderer(er)),
            }
        };


        //Now clone the needed resources and create the asset manager last
        let asset_t_settings = engine_settings.clone();
        let asset_t_pipeline_manager = {
            let mut ren_inst = renderer.lock().expect("failed to lock renderer");
            (*ren_inst).get_pipeline_manager()
        };
        let asset_t_device = {
            let ren_inst = renderer.lock().expect("failed to lock renderer");
            (*ren_inst).get_device()
        };
        let asset_t_queue = {
            let ren_inst = renderer.lock().expect("failed to lock renderer");
            (*ren_inst).get_queue()
        };
        let asset_t_uniform_manager = {
            let ren_inst = renderer.lock().expect("failed to lock renderer");
            (*ren_inst).get_uniform_manager()
        };

        let asset_t_keymap = {
            let inp_sys = input_system.lock().expect("failed to lock input system");
            inp_sys.get_key_map()
        };

        let asset_manager = Arc::new(
            Mutex::new(
                core::resource_management::asset_manager::AssetManager::new(
                    asset_t_pipeline_manager,
                    asset_t_device,
                    asset_t_queue,
                    asset_t_uniform_manager,
                    asset_t_settings,
                    asset_t_keymap
                )
            )
        );

        let thread_pool = Arc::new(Mutex::new(
            jakar_threadpool::ThreadPool::new_hardware_optimal("Jakar_Engine".to_string())
        ));

        //now create the engine struct and retun it to the instigator
        Ok(JakarEngine{
            renderer: renderer,
            asset_manager: asset_manager,
            input_system: input_system,

            engine_settings: engine_settings,
            engine_status: engine_status,
            main_loop_thread: None,
            thread_pool: thread_pool,
        })
    }

    ///Starts an infinite loop which updates the assets, physics and graphics till `end()` is called
    /// or the struct is droped.
    pub fn start(&mut self){
        let renderer_ref = self.renderer.clone();
        let render_state = {
            let render_lck = renderer_ref.lock().expect("failed to lock renderer");
            render_lck.get_render_state()
        };

        let asset_manager_ref = self.asset_manager.clone();
        let asset_state = {
            let asset_lck = asset_manager_ref.lock().expect("Failed to lock asset manager");
            asset_lck.get_asset_manager_state()
        };


        let engine_state_ref = self.engine_status.clone();
        let thread_pool_ref = self.thread_pool.clone();

        //We got all the info we need. Let's start the loop
        let engine_thread = Builder::new().name("EngineMainLoop".to_string()).spawn(move||{
            let renderer = renderer_ref;
            let engine_state = engine_state_ref;
            let asset_manager = asset_manager_ref;
            let thread_pool = thread_pool_ref;

            let mut state_machine = tools::engine_state_machine::EngineStateMachine::new(
                render_state,
                asset_state,
            );

            'main_loop: loop{
                //Check if we should end
                let should_end = {
                    let state_lck = engine_state.lock().expect("failed to lock engine state");
                    match *state_lck{
                        EngineStatus::ENDING => true,
                        EngineStatus::Aboarding(_) => true,
                        _ => false,
                    }
                };

                if should_end{
                    break;
                }

                //Now get the next step from the state_machine
                let next_step = state_machine.update();

                match next_step{
                    NextStep::Render => {
                        //println!("Rendering!", );
                        let mut thread_pool_lck = thread_pool.lock().expect("failed to lock thread pool");
                        let asset_man_loc = asset_manager.clone();
                        let render_loc = renderer.clone();
                        //DEBUG Set render_state
                        state_machine.render_on_cpu();
                        thread_pool_lck.execute(move ||{
                            let mut asset_copy = {
                                let mut asset_manager_lck = asset_man_loc
                                .lock().expect("failed to lock asset manager");
                                (*asset_manager_lck).clone()
                            };

                            //now render the frame
                            let mut render_lck = render_loc.lock().expect("failed to lock renderer");
                            render_lck.render(&mut asset_copy);
                        });
                    },
                    NextStep::UpdateAssets => {
                        //println!("UpdateingAssets!", );
                        let mut thread_pool_lck = thread_pool.lock().expect("failed to lock thread pool");
                        let asset_man_loc = asset_manager.clone();
                        //DEBUG
                        state_machine.asset_working();
                        thread_pool_lck.execute(move||{
                            let mut asset_manager_lck = asset_man_loc
                            .lock().expect("failed to lock asset manager");
                            asset_manager_lck.update();
                        });
                    },
                    NextStep::UpdatePhysics => {
                        //println!("Doing Physics", );
                        //TODO IMPLEMENT PHYSICS
                    },
                    NextStep::Nothing(_) => {
                        //println!("EmptyCycle! {:?}", remaining);
                        //sleep(remaining)
                    }
                }
            }
        }).expect("Failed to start main engine loop");

        self.main_loop_thread = Some(engine_thread);

    }


    ///Ends all threads of the engine and then Returns. **NOTE** When the engine struct is dropped
    /// the same happens.
    pub fn end(mut self){
        //Setting the engine status to end
        //then close input
        //then wait for the threads to finish
        {
            let mut status_lck = self.engine_status.lock().expect("failed to lock engine status");
            (*status_lck) = EngineStatus::ENDING;
        }
        //wait some milliseconds to give the threads some time as well as the gpu
        sleep(Duration::from_millis(100));
        if let Some(thread) = self.main_loop_thread.take(){
            thread.join().expect("failed to join main thread")
        }else{
            println!("Mainthread as already paniced!", );
        }
    }

    ///Returns the asset manager as mutex guard.
    /// **WARNING** the asset manager will be locked withing you function as long as the variable
    /// is in scope. This can potentually lock up the engine. So dont do the following:
    /// #Example
    /// ```
    /// 'game_loop: loop{
    ///     let something_else = engine.get_asset_manager().get_something_else();
    ///     something_else.do_something();
    ///     thread::sleep_ms(100);  //the engine won't render a second frame for 100 ms because a
    ///                             // "engine" is still borrowd in "something_else"
    /// }
    /// ```
    ///
    /// Instead do something like this:
    ///
    /// ```
    /// 'game_loop: loop{
    ///     {
    ///         let something_else = engine.get_asset_manager().get_something_else();
    ///         something_else.do_something();
    ///     }
    ///     thread::sleep_ms(100);  //the engine block
    /// }
    /// ```
    ///
    /// Or if you want to get a value
    ///
    /// ```
    /// 'game_loop: loop{
    ///     let value = {
    ///         let something_else = engine.get_asset_manager().get_something_else();
    ///         something_else.get_something()
    ///     }
    ///     thread::sleep_ms(100);  //the engine won't block
    /// }
    /// ```
    ///
    /// This value can be of cause something `Arc<Mutex<T>>` for instance the engine_settings ;)
    pub fn get_asset_manager<'a>(&'a mut self) -> MutexGuard<'a, core::resource_management::asset_manager::AssetManager>{
        let asset_lock = self.asset_manager.lock().expect("failed to lock asset manager");
        asset_lock
    }

    ///Returns the renderer, for usage have a look at `get_asset_manager()`
    pub fn get_renderer<'a>(&'a mut self) -> MutexGuard<'a, render::renderer::Renderer>{
        let render_lock = self.renderer.lock().expect("failed to lock asset manager");
        render_lock
    }

    ///Returns the input handler, for usage have a look at `get_asset_manager()`
    pub fn get_current_keymap(&self) -> input::keymap::KeyMap{
        let map = {
            let inp_sys = self.input_system.lock().expect("failed to lock input system");
            inp_sys.get_key_map_copy()
        };
        map
    }

    ///Returns the key map with its Mutex guard.
    pub fn get_key_map(&self) -> Arc<Mutex<input::keymap::KeyMap>>{
        let map = {
            let inp_sys = self.input_system.lock().expect("failed to lock input system");
            inp_sys.get_key_map()
        };
        map
    }

    ///Returns the unlocked settings for easy changing. However the engine won't do anything as long as the
    //Mutex is unlocked, so use with care.
    pub fn get_engine_settings_unlocked<'a>(&'a mut self) -> MutexGuard<'a, core::engine_settings::EngineSettings>{
        self.engine_settings.lock().expect("failed to lock engine settings for user")
    }

    ///Returns a copy of the current settings. Can be used for instance to change the current graphics
    /// settings like `exposure` or `gamma`. **Note**: This are the locked once. It is save to store
    //this object somewhere in your gameplay code and only unlock it when you need it.
    pub fn get_settings(&self) -> Arc<Mutex<core::engine_settings::EngineSettings>>{
        self.engine_settings.clone()
    }

    ///Can be used to execute a sendable function on the internal threadpool
    pub fn execute_async<T>(&mut self, fct: T) where T: FnOnce() + Send + 'static {
        let mut thread_pool_lck = self.thread_pool.lock().expect("failed to lock thread_pool");
        thread_pool_lck.execute(fct);
    }

}

impl Drop for JakarEngine{
    fn drop(&mut self){
        {
            let mut status_lck = self.engine_status.lock().expect("failed to lock engine status");
            (*status_lck) = EngineStatus::ENDING;
        }
        //wait some milliseconds to give the threads some time as well as the gpu
        sleep(Duration::from_millis(100));
        if let Some(thread) = self.main_loop_thread.take(){
            thread.join().expect("failed to join main thread")
        }else{
            println!("Mainthread as already paniced!", );
        }
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

///calculate the time a thread must sleep to be not too fast based on
/// the `last_time` the thread was active and the `current_time`
///then actually returns after this time with a new `last_time`
fn sleep_rest_time(last_time: Instant, max_speed: u32) -> Instant{
    //The max speed is given in "iterations per second" so we create a duration from this
    //by deviding it by one and multiplying it by 1_000.0 this way we computed a duration in milliseconds
    //we then test if we still have to wait some time. If yes, we do so, otherwise we just return the
    //current time
    let min_mills_per_iter = Duration::from_millis(((1.0 / max_speed as f64) * 1_000.0) as u64);
    //Now match what happens if we test the duration agains the duration since we started the loop
    //(last_time)

    let time_since_start = last_time.elapsed();

    if time_since_start < min_mills_per_iter{
        //We have to wait the rest time till we pass the min time
        match min_mills_per_iter.checked_sub(time_since_start){
            Some(time_to_wait) => sleep(time_to_wait),
            None => {println!("Failed to calculate time to wait for thread.", );} //do nothing then... but print a message just in case there is a permanent bug
        }
    }else{
        //DO noting as well
    }

    //Return new "last_time"
    Instant::now()
}
