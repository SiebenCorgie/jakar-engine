use std::sync::{Arc, Mutex};

use core::engine_settings;
use input::KeyMap;
use winit;

pub struct InputHandler {
    key_map: Arc<Mutex<KeyMap>>,
    settings: Arc<Mutex<engine_settings::EngineSettings>>,
}


impl InputHandler{
    ///Creates a new input handler, needs to be started via `start` and ended via `end`
    pub fn new(
        key_map: Arc<Mutex<KeyMap>>,
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
    ) -> Self{

        InputHandler{
            key_map: key_map,
            settings: settings,
        }
    }

    ///Starts the input reading and saves the current key-map for usage in everything input releated
    pub fn update_keys(&mut self, events_loop: &mut winit::EventsLoop){

        //Create a time which keeps track of the lst time to calculate the later

        //Create a tmp keymap which will overwrite the global keymap in `input`
        //for each iteration
        //let mut current_keys = KeyMap::new();
        let mut current_keys = {
            (self.key_map.lock().expect("failed to lock source key map")).clone()
        };

        //Kill the axis motion for now
        current_keys.reset_data();

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

                            //Copy our selfs a settings instance to change settings which ... changed
                            let mut settings_instance = {
                                let lck = self.settings.lock().expect("failed to lock settings in input handler");
                                (*lck).clone()
                            };

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
                        /* For winit 0.11.1
                        CursorMoved {device_id, position, modifiers} =>{

                        },
                        CursorEntered{device_id} =>{

                        },
                        CursorLeft{device_id} =>{

                        },
                        MouseWheel{device_id, delta, phase, modifiers} =>{

                        },
                        MouseInput{device_id, state, button, modifiers} =>{

                        },
                        */
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
            let mut key_map_unlck = self.key_map
            .lock()
            .expect("failed to hold key_map_inst lock while updating key info");
            (*key_map_unlck) = current_keys;
        }
    }
}
