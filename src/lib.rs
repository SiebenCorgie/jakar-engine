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
extern crate time;
extern crate image;
extern crate gltf;
extern crate gltf_importer;
extern crate gltf_utils;
//the new custom tree crate
extern crate jakar_tree;


///The engine core defines most functions and
///traits needed to feed the renderer and communicate with the physics.
///It also mamanges the scene tree and how to get specific information out of it
pub mod core;
///The engines renderer currently WIP
pub mod render;
use render::renderer::BuildRender;
///A collection of helpfull tools for integration of data with the engine
pub mod tools;
///A small thread who will run and administrate the winit window, as well as its input
///processing
pub mod input;

use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::sync::mpsc;


///Holds possible engine creation errors
pub enum CreationErrors{
    ///Is returned when something went wrong while creating the renderer. The String holds a message.
    FailedToCreateRenderer(String),
    ///Is returned when there was a problem with the asset manager or thread creation
    FailedToCreateAssetManager,
    ///Is returned when the engine couldn't start the input loop.
    FailedToCreateInputManager,
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
    render_thread: thread::JoinHandle<()>,

    pub asset_manager: Arc<Mutex<core::resource_management::asset_manager::AssetManager>>,
    asset_thread: thread::JoinHandle<()>,

    pub input_system_messages: Arc<Mutex<Option<input::InputMessage>>>,
    pub keymap: Arc<Mutex<input::KeyMap>>,
    input_thread: thread::JoinHandle<()>,

    pub engine_settings: Arc<Mutex<core::engine_settings::EngineSettings>>,

    pub engine_status: Arc<Mutex<EngineStatus>>,

}

///Implements the main functions for the engine. Other functionality can be imported in scope
///via traits.
impl JakarEngine {
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
    pub fn start(settings: Option<core::engine_settings::EngineSettings>) -> Result<Self, CreationErrors>{
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

        //=========================================================================================

        //Start the input thread

        //First of all we need two channel. The First one will send the instance of vulkano to the
        // input creation process when the build is at that stage. The instance is needed to build a window.
        // Since we can't send the EventLoop we have to send the Instnace, build the window in the input thread
        // and the send the SendAble Window back to the render builder.
        let (in_instance_send, in_instance_recv) = mpsc::channel();
        let (in_window_send, in_window_recv) = mpsc::channel();
        let (in_keymap_send, in_keymap_recv) = mpsc::channel();
        //This object will be used to send messages to the input thread. Either for registering
        //callbacks or to end the thread at the moment
        let input_messages = Arc::new(Mutex::new(None));
        //This one will end the input thread if needed
        let input_engine_status = engine_status.clone();
        //The settings used in the input thread
        let input_engine_settings = engine_settings.clone();
        //Reserved for the engine struct.
        let engine_in_messages = input_messages.clone();
        //Now start the input
        println!("Starting input thread!", );
        let input_thread_handle = thread::spawn(move || {
            let mut input_system ={
                match input::Input::new(
                    input_engine_settings.clone(),
                    in_instance_recv,
                    in_window_send
                ){
                    Ok(in_sys) => in_sys, //all right
                    Err(er) => {
                        //lock the engine status and add the abord message
                        *(input_engine_status.lock().expect("failed to lock status in input")) = EngineStatus::Aboarding(er);
                        return;
                    }
                }
            };

            println!("Finished input system starting thread now!", );
            //Since we started the input successful we can now enter the loop od updating the input
            //in the right speed
            let max_input_speed = input_engine_settings
            .lock().expect("failed to lock settings").max_input_speed.clone();

            let mut time_step = Instant::now();

            //now send the keymap pointer
            match in_keymap_send.send(input_system.get_key_map()){
                Ok(_) => {},
                Err(_) => {
                    println!("Failed to send keymap to main thread!", );

                }
            }

            'input_loop: loop{
                input_system.update();

                time_step = sleep_rest_time(time_step, max_input_speed);

                if *(input_engine_status.lock().expect("failed to lock status")) == EngineStatus::ENDING{
                    println!("Ending Input thread...", );
                    break;
                }
            }
        });

        println!("Started Input system!", );



        //=========================================================================================

        //Start the renderer
        //first create reciver and sender for the render handler
        let (render_t_sender, render_t_reciver) = mpsc::channel();
        //also create an sender and reciver to send the newly created asset manager once it is available
        let (render_asset_sender, render_asset_reciver) = mpsc::channel();

        //copy all relevant infos and move them into the thread
        let render_engine_status = engine_status.clone();
        let render_settings = engine_settings.clone();
        let render_thread = thread::spawn(move ||{
            //Now create the renderer

            //now read the maximum fps the engine should have
            let max_fps = {
                let settings = render_settings.lock().expect("failed to lock render settings");
                (*settings).max_fps
            };
            //Create a renderer with the input system
            let (render, mut gpu_future) = {
                //first we create an engine builder. Then we configure it. Finally we return
                // the renderer and the gpu future. If something went wrong while creating the
                // renderer we set the engine status to Err(message). This way we can ensure
                // that the engine only starts if the renderer is created successfuly.
                let mut render_builder = render::render_builder::RenderBuilder::new();
                //Configure======================================
                {
                    let settings = render_settings.lock().expect("failed to lock render settings");
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

                //===============================================
                println!("Building renderer now!", );
                //now build
                let render_status = render_builder.create(
                    in_instance_send,
                    in_window_recv,
                    render_settings,
                );

                //now we match the craetion status, if sucessful, we can return the renderer
                // and the gpu future. If not, we set the Engine status to
                // CreationErrors::FailedToCreateRenderer(Message)
                match render_status{
                    Ok((render,future)) => {
                        //wrapping the renderer in an Arc
                        let arc_render = Arc::new(Mutex::new(render));
                        let gpu_future_box = future;
                        //return both
                        (arc_render, gpu_future_box)
                    },
                    Err(msg) => {
                        //something went wrong :(
                        (*(render_engine_status
                            .lock()
                            .expect("failed to lock render engine status")
                        )) = EngineStatus::Aboarding(msg.clone());
                        println!("Failed to create renderer: {}\n returning!", msg);
                        return;
                    }
                }

            };

            //Created an renderer, send it to the engine struct
            match render_t_sender.send(render.clone()){
                Ok(_) => {},
                Err(e) => println!("Failed to send renderer to main thread: {}", e),
            }

            //wait till we are reciving an asset manager
                //we are actually cycling between trying to get the asset_manager and testing
                // the status, if there went something wrong while creating any of the sub systems
                // we return as well, without starting the rendering loop
            //create a variable for the asset manager. Will return Some eventually
            let mut asset_manager_inst = None;

            'render_waiting_loop: loop{
                //trying to recive
                match render_asset_reciver.try_recv(){
                    Ok(manager) => {
                        //actually recived something, will overwrite now
                        asset_manager_inst = Some(manager);
                        //now we can break the loop
                        break;
                    }
                    Err(r) =>{
                        //well either the sender is disconnected or has not yet sended
                        //when disconected we can return, when not sended we test if any other
                        //system has crashed
                        match r{
                            mpsc::TryRecvError::Disconnected => return,
                            mpsc::TryRecvError::Empty => {},
                        }
                    }
                }

                //now test, if the messag is "aboard" if yes, we can aboard as well
                {
                    match *(render_engine_status
                        .lock()
                        .expect("failed to lock render engine status")
                    )
                    {
                        EngineStatus::Aboarding(_) => return,
                        _ => {}, //all is nice test again for a return value of the channel
                    }
                }
            }

            //get the asset manager from the asset manager creation thread
            let asset_manager: Arc<Mutex<core::resource_management::asset_manager::AssetManager>> = {
                match asset_manager_inst{
                    None => return,
                    Some(am) => am,
                }
            };


            //Set the thread start time
            let mut last_time = Instant::now();

            let mut fps_time_start = Instant::now();
            println!("Started renderer!", );

            //now start the rendering loop
            'render_thread: loop{
                //lock the renderer and render an image
                //TODO loc in scope
                let mut renderer_lck = render
                .lock().expect("failed to lock renderer");

                //now render a frame and get the new gpu future, this one can be used to stopp the
                //rendering correctly bey joining the gpu future

                //to render a frame we just copy the whole asset manager and submit the copy to the
                //renderer, this might be optimized
                let mut asset_copy = {
                    let asset_manager_lck = asset_manager
                    .lock().expect("failed to lock asset manager");
                    (*asset_manager_lck).clone()
                };
                //gpu_future =
                (*renderer_lck).render(&mut asset_copy); // gpu_future);

                //Tet if the engine should still run
                let engine_is_running = {
                    let status = render_engine_status.lock().expect("failed to lock engine status");
                    match *status{
                        EngineStatus::RUNNING => true,
                        EngineStatus::STARTING => true, //also keeping the loop when still starting, the asset manager should be available because of the .recv() call
                        _ => false,
                    }
                };

                if !engine_is_running{
                    println!("Renderer should end", );
                    //engine is stoping, ending loop
                    //wait a second for the gpu to finish its last work, then clean up the future
                    thread::sleep(Duration::from_millis(60));
                    //end frame on gpu
                    gpu_future.cleanup_finished();
                    break;
                }
                //now sleep the rest if needed
                last_time = sleep_rest_time(last_time, max_fps);


                let fps_time = fps_time_start.elapsed().subsec_nanos();

                let fps = 1.0/ (fps_time as f32 / 1_000_000_000.0);
                println!("This Frame after waiting: {}", fps);

                fps_time_start = Instant::now();

            }
        });

        //last but not least, we try to recive the renderer  as fast as possible
        //if something went wrong, teh status should be aborad.
        //if this is the case we can already return the function the the error messages.
        //else we test if we can recive something.
        //all this happens in the loop
        let mut renderer_isnt = None;
        'main_render_waiting_loop: loop{
            //trying to recive
            match render_t_reciver.try_recv(){
                Ok(renderer) => {
                    //actually recived something, will overwrite now
                    renderer_isnt = Some(renderer);
                    //now we can break the loop
                    println!("Got a Renderer in the asset waiting loop", );
                    break;
                }
                Err(r) =>{
                    //well either the sender is disconnected or has not yet sended
                    //when disconected we can return, when not sended we test if any other
                    //system has crashed
                    match r{
                        mpsc::TryRecvError::Disconnected => {
                            //while we already know that something went wrong, we try to get the message later
                            println!("Renderer crashed, getting message", );
                            return Err(CreationErrors::FailedToCreateRenderer("Renderer Disconnected".to_string()));
                        },
                        mpsc::TryRecvError::Empty => {},
                    }
                }
            }

            //now test, if the messag is "aboard" if yes, we can aboard as well
            {
                match *(engine_status
                    .lock()
                    .expect("failed to lock render engine status")
                )
                {
                    EngineStatus::Aboarding(ref msg) => return Err(
                        CreationErrors::FailedToCreateRenderer(msg.clone())
                    ),
                    _ => {}, //all is nice test again for a return value of the channel
                }
            }
        }


        //now recive the renderer in the main thread
        let renderer = match renderer_isnt{
            Some(renderer) => renderer,
            None => return Err(
                CreationErrors::FailedToCreateRenderer(
                    "Reciving was successful, but returned non".to_string()
                )
            ),
        };

        //=========================================================================================

        //Since renderer and therefore input system should be running now we ca recive the input keymap now
        //Now recive the keymap and store it for the later engine struct
        let key_map = {
            match in_keymap_recv.recv(){
                Ok(km) => km,
                Err(_) => {
                    println!("Failed to recive Key map!", );
                    return Err(CreationErrors::FailedToCreateInputManager)
                },
            }
        };


        //Same as the renderer, crate an reciver and sender for the asset manager,
        //also create clones for needed systems to create the asset manager
        let (asset_t_sender, asset_t_reciver) = mpsc::channel();

        let asset_t_status = engine_status.clone();
        let asset_t_settings = engine_settings.clone();
        let asset_t_keymap = key_map.clone();
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
        //Start the asset manager
        let asset_thread = thread::spawn(move ||{
            //dirst of all read some values we need later for the thread speed etc.
            //read the maximum asset thread speed from the configuration
            let max_speed = {
                let settings = asset_t_settings.lock().expect("failed to locks settings");
                (*settings).max_asset_updates
            };

            //Create a asset manager for the renderer
            let asset_manager = {

                Arc::new(
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
                )
            };

            //now send the asset manager to the main thread
            match asset_t_sender.send(asset_manager.clone()){
                Ok(_) => {},
                Err(e) => println!("Failed to send asset manager to main thread: {}", e),
            }
            //also send a copy to the rendering thread
            match render_asset_sender.send(asset_manager.clone()){
                Ok(_) => {},
                Err(e) => println!("Failed to send asset manager to renderer thread: {}", e),
            }

            //finished with starting the asset manager, now loop till we are told to stop
            //TODO
            //create a time stemp which will be used to calculate the waiting time for each tick
            let mut last_time = Instant::now();

            println!("Started Asset Manager", );
            'asset_loop: loop{

                //now update the asset mananger
                //in scope, because we want to be able to do other stuff while the thread is waiting
                {
                    let mut asset_manager_lck = asset_manager.lock().expect("failed to lock asset manager while updating assets");
                    (*asset_manager_lck).update();
                }

                //now check for the engine status, if we should end, end the loop and therefore return the thread
                let engine_is_running = {
                    let status = asset_t_status.lock().expect("failed to lock engine status");
                    match *status{
                        EngineStatus::RUNNING => true,
                        EngineStatus::STARTING => true, //also keeping the loop when still starting, the asset manager should be available because of the .recv() call
                        _ => false,
                    }
                };

                if !engine_is_running{
                    //engine is stoping, ending loop
                    println!("Ending asset thread", );
                    break;
                }
                //sleep for the rest time to be not too fast
                last_time = sleep_rest_time(last_time, max_speed);
            }
        });

        //=========================================================================================
        //Recive asset manager and store them in the struct
        let asset_manager_inst = asset_t_reciver
        .recv()
        .expect("failed to recive asset manager for jakar struct");

        //Now switch the state to "running"
        {
            let mut engine_status_lck = engine_status.lock().expect("failed to lock engine status");
            (*engine_status_lck) = EngineStatus::RUNNING;
        }


        //now create the engine struct and retun it to the instigator
        Ok(JakarEngine{
            renderer: renderer,
            render_thread: render_thread,

            asset_manager: asset_manager_inst,
            asset_thread: asset_thread,

            input_system_messages: engine_in_messages,
            keymap: key_map,
            input_thread: input_thread_handle,

            engine_settings: engine_settings,

            engine_status: engine_status,
        })
    }

    ///Ends all threads of the engine and then Returns
    pub fn end(self){
        //Setting the engine status to end
        //then close input
        //then wait for the threads to finish
        {
            let mut status_lck = self.engine_status.lock().expect("failed to lock engine status");
            (*status_lck) = EngineStatus::ENDING;
        }
        //wait some milliseconds to give the threads some time as well as the gpu
        thread::sleep(Duration::from_millis(100));

        //now try to join the thread
        match self.render_thread.join(){
            Ok(_) => println!("Ended render thread successfuly!"),
            Err(_) => println!("Failed to end render thread while ending"),
        }

        match self.asset_thread.join(){
            Ok(_) => println!("Ended asset_thread thread successfuly!"),
            Err(_) => println!("Failed to end asset_thread thread while ending"),
        }

        match self.input_thread.join(){
            Ok(_) => println!("Ended input_thread thread successfuly!"),
            Err(_) => println!("Failed to end input_thread thread while ending"),
        }
        println!("Finished ending Engine, returning to main thread", );
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
    pub fn get_current_keymap(&self) -> input::KeyMap{
        let map = {
            (self.keymap.lock().expect("Failed to get current keymap")).clone()
        };
        map
    }

    ///Returns a copy of the current settings. Can be used for instance to change the current graphics
    /// settings like `exposure` or `gamma`.
    pub fn get_settings(&self) -> Arc<Mutex<core::engine_settings::EngineSettings>>{
        self.engine_settings.clone()
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
            Some(time_to_wait) => thread::sleep(time_to_wait),
            None => {println!("Failed to calculate time to wait for thread.", );} //do nothing then... but print a message just in case there is a permanent bug
        }
    }else{
        //DO noting as well
    }

    //Return new "last_time"
    Instant::now()
}
