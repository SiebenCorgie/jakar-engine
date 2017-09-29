use std::sync::{Arc, Mutex};
use std::thread;

use core::engine_settings;
use input::KeyMap;
use winit;

use std::time::{Duration, Instant};


#[derive(PartialEq, Eq)]
pub enum InputHandlerStates {
    ShouldEnd,
    Running,
}

pub struct InputHandler {
    key_map: Arc<Mutex<KeyMap>>,

    events_loop: Arc<Mutex<winit::EventsLoop>>,
    pub state: Arc<Mutex<InputHandlerStates>>,

    settings: Arc<Mutex<engine_settings::EngineSettings>>,

    max_polling_speed: u32,
}


impl InputHandler{
    ///Creates a new input handler, needs to be started via `start` and ended via `end`
    pub fn new(
        key_map: Arc<Mutex<KeyMap>>,
        events_loop: Arc<Mutex<winit::EventsLoop>>,
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
    ) -> Self{

        //read the max polling speed
        let max_polling_speed = {
            let n_settings = settings.lock().expect("failed to lock settings");
            (*n_settings).max_input_speed
        };

        InputHandler{
            key_map: key_map,
            events_loop: events_loop,

            settings: settings,

            state: Arc::new(Mutex::new(InputHandlerStates::Running)),

            max_polling_speed: max_polling_speed,
        }
    }

    ///Starts the input reading and saves the current key-map for usage in everything input releated
    pub fn start(&mut self) -> thread::JoinHandle<()>{

        let key_map_inst = self.key_map.clone();
        let events_loop_inst = self.events_loop.clone();
        let state_instance = self.state.clone();
        let settings_ins = self.settings.clone();
        let max_polling_speed_inst = self.max_polling_speed.clone();


        //Start the continues input polling
        let input_thread = thread::spawn(move ||{

            //Create a time which keeps track of the lst time to calculate the later
            // `thread::sleep() duartion to keep steady `self.max_polling_speed`
            let mut last_time = Instant::now();
            //Create a tmp keymap which will overwrite the global keymap in `input`
            //for each iteration
            let mut current_keys = KeyMap::new();

            loop{
                //Polling all events TODO make a variable input cap for polling
                //Copy our selfs a settings instance to change settings which ... changed
                let mut settings_instance = {
                    let lck = settings_ins.lock().expect("failed to lock settings in input handler");

                    (*lck).clone()
                };

                // And a small flag to prevent to much locking
                let mut b_engine_settings_changed = false;

                //lock the events loop for polling
                let mut events_loop = (*events_loop_inst).lock().expect("Failed to hold lock on eventsloop");

                //Check if the thread should end alread, return
                {
                    let mut state_lck = state_instance.lock().expect("failed to lock thread state");
                    if *state_lck == InputHandlerStates::ShouldEnd{
                        //println!("STATUS: INPUT HANDLER: ending input thread", );
                        break;
                    }
                }

                //Kill the axis motion for now
                current_keys.mouse_delta_x = 0.0;
                current_keys.mouse_delta_y = 0.0;


                //Now do the events polling
                events_loop.poll_events(|ev| {
                    match ev {
                        //Check the event type
                        //window
                        winit::Event::WindowEvent{window_id, event} =>{
                            //Make life easier and temporarly import all the events
                            use winit::WindowEvent::*;

                            match event{
                                Resized(width , height) =>{

                                    b_engine_settings_changed = true;
                                    settings_instance.set_dimensions(
                                        width.clone() as u32,
                                        height.clone() as u32
                                    );
                                    //println!("Resized to {} / {}", width, height );
                                },
                                Moved(width, height) =>{

                                },
                                Closed => {
                                    current_keys.closed = true;
                                    //println!("STATUS: INPUT HANDLER: closing", );
                                },
                                DroppedFile(file_path) =>{
                                    //println!("Droped file with path: {:?}", file_path );
                                },
                                HoveredFile(file_path) => {

                                },
                                HoveredFileCancelled => {

                                },
                                ReceivedCharacter(character) =>{

                                },
                                Focused(b_state) =>{

                                },
                                KeyboardInput {device_id, input} =>{
                                    use winit::VirtualKeyCode;

                                    //Match the type of input
                                    match input.state{
                                        //if pressed set true, else leave false
                                        winit::ElementState::Pressed => {
                                            match input.virtual_keycode{
                                                //main keys
                                                Some(VirtualKeyCode::A) => current_keys.a = true,
                                                Some(VirtualKeyCode::B) => current_keys.b = true,
                                                Some(VirtualKeyCode::C) => current_keys.c = true,
                                                Some(VirtualKeyCode::D) => current_keys.d = true,
                                                Some(VirtualKeyCode::E) => current_keys.e = true,
                                                Some(VirtualKeyCode::F) => current_keys.f = true,
                                                Some(VirtualKeyCode::G) => current_keys.g = true,
                                                Some(VirtualKeyCode::H) => current_keys.h = true,
                                                Some(VirtualKeyCode::I) => current_keys.i = true,
                                                Some(VirtualKeyCode::J) => current_keys.j = true,
                                                Some(VirtualKeyCode::K) => current_keys.k = true,
                                                Some(VirtualKeyCode::L) => current_keys.l = true,
                                                Some(VirtualKeyCode::M) => current_keys.m = true,
                                                Some(VirtualKeyCode::N) => current_keys.n = true,
                                                Some(VirtualKeyCode::O) => current_keys.o = true,
                                                Some(VirtualKeyCode::P) => current_keys.p = true,
                                                Some(VirtualKeyCode::Q) => current_keys.q = true,
                                                Some(VirtualKeyCode::R) => current_keys.r = true,
                                                Some(VirtualKeyCode::S) => current_keys.s = true,
                                                Some(VirtualKeyCode::T) => current_keys.t = true,
                                                Some(VirtualKeyCode::U) => current_keys.u = true,
                                                Some(VirtualKeyCode::V) => current_keys.v = true,
                                                Some(VirtualKeyCode::W) => current_keys.w = true,
                                                Some(VirtualKeyCode::X) => current_keys.x = true,
                                                Some(VirtualKeyCode::Y) => current_keys.y = true,
                                                Some(VirtualKeyCode::Z) => current_keys.z = true,
                                                //top numbers
                                                Some(VirtualKeyCode::Key1) => current_keys.t_1 = true,
                                                Some(VirtualKeyCode::Key2) => current_keys.t_2 = true,
                                                Some(VirtualKeyCode::Key3) => current_keys.t_3 = true,
                                                Some(VirtualKeyCode::Key4) => current_keys.t_4 = true,
                                                Some(VirtualKeyCode::Key5) => current_keys.t_5 = true,
                                                Some(VirtualKeyCode::Key6) => current_keys.t_6 = true,
                                                Some(VirtualKeyCode::Key7) => current_keys.t_7 = true,
                                                Some(VirtualKeyCode::Key8) => current_keys.t_8 = true,
                                                Some(VirtualKeyCode::Key9) => current_keys.t_9 = true,
                                                Some(VirtualKeyCode::Key0) => current_keys.t_0 = true,
                                                //num pad
                                                Some(VirtualKeyCode::Numpad0) => current_keys.num_0 = true,
                                                Some(VirtualKeyCode::Numpad1) => current_keys.num_1 = true,
                                                Some(VirtualKeyCode::Numpad2) => current_keys.num_2 = true,
                                                Some(VirtualKeyCode::Numpad3) => current_keys.num_3 = true,
                                                Some(VirtualKeyCode::Numpad4) => current_keys.num_4 = true,
                                                Some(VirtualKeyCode::Numpad5) => current_keys.num_5 = true,
                                                Some(VirtualKeyCode::Numpad6) => current_keys.num_6 = true,
                                                Some(VirtualKeyCode::Numpad7) => current_keys.num_7 = true,
                                                Some(VirtualKeyCode::Numpad8) => current_keys.num_8 = true,
                                                Some(VirtualKeyCode::Numpad9) => current_keys.num_9 = true,
                                                //special keys
                                                Some(VirtualKeyCode::LControl) => current_keys.ctrl_l = true,
                                                Some(VirtualKeyCode::RControl) => current_keys.ctrl_r = true,
                                                Some(VirtualKeyCode::LAlt) => current_keys.alt_l = true,
                                                Some(VirtualKeyCode::RAlt) => current_keys.alt_r = true,
                                                Some(VirtualKeyCode::LWin) => current_keys.super_l = true,
                                                Some(VirtualKeyCode::RWin) => current_keys.super_r = true,
                                                Some(VirtualKeyCode::Capital) => current_keys.caps_lock = true,
                                                Some(VirtualKeyCode::LShift) => current_keys.shift_l = true,
                                                Some(VirtualKeyCode::RShift) => current_keys.shift_r = true,
                                                Some(VirtualKeyCode::Tab) => current_keys.tab = true,
                                                Some(VirtualKeyCode::Space) => current_keys.space = true,
                                                Some(VirtualKeyCode::Return) => current_keys.enter = true,
                                                Some(VirtualKeyCode::NumpadEnter) => current_keys.nume_enter = true,
                                                Some(VirtualKeyCode::Escape) => current_keys.escape = true,
                                                //arrows
                                                Some(VirtualKeyCode::Up) => current_keys.up = true,
                                                Some(VirtualKeyCode::Down) => current_keys.down = true,
                                                Some(VirtualKeyCode::Left) => current_keys.left = true,
                                                Some(VirtualKeyCode::Right) => current_keys.right = true,
                                                _ => {},
                                            }
                                        },
                                        winit::ElementState::Released => {
                                            //leave state to false
                                            match input.virtual_keycode{
                                                Some(VirtualKeyCode::A) => current_keys.a = false,
                                                Some(VirtualKeyCode::B) => current_keys.b = false,
                                                Some(VirtualKeyCode::C) => current_keys.c = false,
                                                Some(VirtualKeyCode::D) => current_keys.d = false,
                                                Some(VirtualKeyCode::E) => current_keys.e = false,
                                                Some(VirtualKeyCode::F) => current_keys.f = false,
                                                Some(VirtualKeyCode::G) => current_keys.g = false,
                                                Some(VirtualKeyCode::H) => current_keys.h = false,
                                                Some(VirtualKeyCode::I) => current_keys.i = false,
                                                Some(VirtualKeyCode::J) => current_keys.j = false,
                                                Some(VirtualKeyCode::K) => current_keys.k = false,
                                                Some(VirtualKeyCode::L) => current_keys.l = false,
                                                Some(VirtualKeyCode::M) => current_keys.m = false,
                                                Some(VirtualKeyCode::N) => current_keys.n = false,
                                                Some(VirtualKeyCode::O) => current_keys.o = false,
                                                Some(VirtualKeyCode::P) => current_keys.p = false,
                                                Some(VirtualKeyCode::Q) => current_keys.q = false,
                                                Some(VirtualKeyCode::R) => current_keys.r = false,
                                                Some(VirtualKeyCode::S) => current_keys.s = false,
                                                Some(VirtualKeyCode::T) => current_keys.t = false,
                                                Some(VirtualKeyCode::U) => current_keys.u = false,
                                                Some(VirtualKeyCode::V) => current_keys.v = false,
                                                Some(VirtualKeyCode::W) => current_keys.w = false,
                                                Some(VirtualKeyCode::X) => current_keys.x = false,
                                                Some(VirtualKeyCode::Y) => current_keys.y = false,
                                                Some(VirtualKeyCode::Z) => current_keys.z = false,
                                                //top numbers
                                                Some(VirtualKeyCode::Key1) => current_keys.t_1 = false,
                                                Some(VirtualKeyCode::Key2) => current_keys.t_2 = false,
                                                Some(VirtualKeyCode::Key3) => current_keys.t_3 = false,
                                                Some(VirtualKeyCode::Key4) => current_keys.t_4 = false,
                                                Some(VirtualKeyCode::Key5) => current_keys.t_5 = false,
                                                Some(VirtualKeyCode::Key6) => current_keys.t_6 = false,
                                                Some(VirtualKeyCode::Key7) => current_keys.t_7 = false,
                                                Some(VirtualKeyCode::Key8) => current_keys.t_8 = false,
                                                Some(VirtualKeyCode::Key9) => current_keys.t_9 = false,
                                                Some(VirtualKeyCode::Key0) => current_keys.t_0 = false,
                                                //num pad
                                                Some(VirtualKeyCode::Numpad0) => current_keys.num_0 = false,
                                                Some(VirtualKeyCode::Numpad1) => current_keys.num_1 = false,
                                                Some(VirtualKeyCode::Numpad2) => current_keys.num_2 = false,
                                                Some(VirtualKeyCode::Numpad3) => current_keys.num_3 = false,
                                                Some(VirtualKeyCode::Numpad4) => current_keys.num_4 = false,
                                                Some(VirtualKeyCode::Numpad5) => current_keys.num_5 = false,
                                                Some(VirtualKeyCode::Numpad6) => current_keys.num_6 = false,
                                                Some(VirtualKeyCode::Numpad7) => current_keys.num_7 = false,
                                                Some(VirtualKeyCode::Numpad8) => current_keys.num_8 = false,
                                                Some(VirtualKeyCode::Numpad9) => current_keys.num_9 = false,
                                                //special keys
                                                Some(VirtualKeyCode::LControl) => current_keys.ctrl_l = false,
                                                Some(VirtualKeyCode::RControl) => current_keys.ctrl_r = false,
                                                Some(VirtualKeyCode::LAlt) => current_keys.alt_l = false,
                                                Some(VirtualKeyCode::RAlt) => current_keys.alt_r = false,
                                                Some(VirtualKeyCode::LWin) => current_keys.super_l = false,
                                                Some(VirtualKeyCode::RWin) => current_keys.super_r = false,
                                                Some(VirtualKeyCode::Capital) => current_keys.caps_lock = false,
                                                Some(VirtualKeyCode::LShift) => current_keys.shift_l = false,
                                                Some(VirtualKeyCode::RShift) => current_keys.shift_r = false,
                                                Some(VirtualKeyCode::Tab) => current_keys.tab = false,
                                                Some(VirtualKeyCode::Space) => current_keys.space = false,
                                                Some(VirtualKeyCode::Return) => current_keys.enter = false,
                                                Some(VirtualKeyCode::NumpadEnter) => current_keys.nume_enter = false,
                                                Some(VirtualKeyCode::Escape) => current_keys.escape = false,
                                                //arrows
                                                Some(VirtualKeyCode::Up) => current_keys.up = false,
                                                Some(VirtualKeyCode::Down) => current_keys.down = false,
                                                Some(VirtualKeyCode::Left) => current_keys.left = false,
                                                Some(VirtualKeyCode::Right) => current_keys.right = false,




                                                _ => {},
                                            }
                                        },
                                    }

                                },
                                MouseMoved {device_id, position} =>{

                                },
                                MouseEntered{device_id} =>{

                                },
                                MouseLeft{device_id} =>{

                                },
                                MouseWheel{device_id, delta, phase} =>{

                                },
                                MouseInput{device_id, state, button} =>{

                                },
                                TouchpadPressure{device_id, pressure, stage} =>{

                                },
                                AxisMotion{device_id, axis, value} =>{

                                },
                                Refresh =>{

                                },
                                Suspended(b_state) =>{

                                },
                                Touch(touch) =>{

                                },
                            }
                        },
                        //Device
                        winit::Event::DeviceEvent{device_id, event} => {
                            //Using raw events for the mouse movement
                            //One could, potentually use a 3rd axis for a 3d controller movement
                            //I think
                            match event{
                                winit::DeviceEvent::Motion{axis, value} => {
                                    //since winit 0.7.6
                                    match axis {
                                        0 => current_keys.mouse_delta_x = value, //Mouse x
                                        1 => current_keys.mouse_delta_y = value, //mouse y
                                        2 => {}, //Currently doing nothing, I guess this is mouse wheel

                                        _ => {
                                            //don't do anything
                                        },
                                    }
                                }
                                //This could register raw device events, however, not used atm
                                _ => {},
                            }
                        },
                        //Awake (not implemented)
                        winit::Event::Awakened => {},

                    }
                });

                //Overwrite the Arc<Mutex<KeyMap>> with the new capture
                {
                    let mut key_map_unlck = key_map_inst
                    .lock()
                    .expect("failed to hold key_map_inst lock while updating key info");
                    (*key_map_unlck) = current_keys;
                }

                // If some global settings changed, we can push them to the engine_settings instance
                // of this engine run
                if b_engine_settings_changed{
                    //println!("STATUS: INPUT_HANDLER: Settings changed in Input handler", );
                    let l_settings_ins = settings_ins.clone();
                    let mut settings_lck = l_settings_ins
                    .lock()
                    .expect("failed to lock settings for overwrite");

                    (*settings_lck) = settings_instance;
                }


                //println!("TESTING TIME", );
                //Calculate the time to wait
                //get difference between last time and now
                let difference = last_time.elapsed();

                //test if the difference is smaller then the max_polling_speed
                //if yes the thread was too fast and we need to sleep for the rest of time till
                //we get the time to compleate the polling
                let compare_time = Duration::new(0, ((1.0 / max_polling_speed_inst as f32) * 1_000_000_000.0) as u32);
                //println!("Max_speed: {:?}", compare_time.clone());
                //println!("Difference: {:?}", difference.clone());

                if
                    (difference.subsec_nanos() as f64) <
                    (compare_time.subsec_nanos() as f64) {

                    //Sleep the rest time till we finish the max time in f64
                    let time_to_sleep =
                    compare_time.subsec_nanos() as f64 - difference.subsec_nanos() as f64;
                    //calc a duration
                    let sleep_duration = Duration::new(0, time_to_sleep as u32);
                    //and sleep it
                    thread::sleep(sleep_duration);
                }

                //Reset the last time for next frame
                last_time = Instant::now();

            }
        });

        input_thread
    }

    ///Ends the input thread via a end flag
    #[inline]
    pub fn end(&mut self){
        {
            let mut state_lck = self.state
            .lock()
            .expect("Failed to lock input thread state for ending");

            *state_lck = InputHandlerStates::ShouldEnd;
        }
    }
}
