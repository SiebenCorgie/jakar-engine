use std::sync::{Mutex, Arc};

use std::sync::mpsc;

use winit;

use core::engine_settings;
use render::window::Window;

use vulkano::instance::Instance;
///a sub mod who will read the input since the last loop
///and store the key values in a struct
pub mod input_handler;

///Contains the state of each key.
pub mod keymap;



//A enum which is used to message differen behavoirs to the Input system/thread. Since we can't just
//keep a Arc<MutexT> of this struct.
pub enum InputMessage {
    ///Has to be implemented but it is there
    RegisterCallback,
}

///Describes the state in which the input manager is currently.
//Can be used to end the system.
pub enum InputState {
    ///Is the normal state while running.
    Running,
    ///If this is the current state the system will try to end as fast as possible.
    End
}

///Manages all input
///TODO implement the message api
pub struct Input {
    input_handler: input_handler::InputHandler,
    //It is not allowed to share this, thats why we have this complicated setup :/
    events_loop: winit::EventsLoop,
    settings: Arc<Mutex<engine_settings::EngineSettings>>,
    pub key_map: Arc<Mutex<keymap::KeyMap>>,

    status: InputState,

}


impl Input{
    ///Creates a new Input instance. It needs to recive the vulkan instance which tagets the window
    /// at some point and it will send the newly created window through the recived `window_sender` after reciving the
    /// vulkano instance.
    pub fn new(
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        instance_reciver: mpsc::Receiver<Arc<Instance>>,
        window_sender: mpsc::Sender<Window>,
    ) -> Result<Self, String>{

        let key_map_inst = Arc::new(Mutex::new(keymap::KeyMap::new()));

        let events_loop = winit::EventsLoop::new();

        //Will be some if everything went right
        let mut instance = None;

        //Try to recive the vulkano instance
        'recive_loop: loop{
            match instance_reciver.try_recv(){
                Ok(instance_recv) => {
                    println!("Got Instance!", );
                    instance = Some(instance_recv);
                    break;
                },
                Err(r) => {
                    //println!("Did not recive Instance", );
                    match r{
                        mpsc::TryRecvError::Disconnected => return Err(String::from("Could not build Input, did not recive Instance!")),
                        mpsc::TryRecvError::Empty => {}, //All is right, try again
                    }
                },
            }
        }

        match instance{
            Some(inst) => {
                //Ready to build and send this window
                let window = Window::new(
                    &inst, &events_loop, settings.clone()
                );

                //Now send back to the render builder
                match window_sender.send(window){
                    Ok(_) => {},
                    Err(_) => println!("Failed to send window to main!", ),
                }
            }
            None => return Err(String::from("Could not unwrap Instance.")),
        }

        //Finally build the input system and return
        Ok(Input{
            input_handler: input_handler::InputHandler::new(key_map_inst.clone(), settings.clone()),
            events_loop: events_loop,
            settings: settings,
            key_map: key_map_inst.clone(),
            status: InputState::Running,
        })
    }

    ///Updates the input state. Currently this only updates the keymap. Later it will call
    /// a list of action callbacks as well.
    pub fn update(&mut self){
        self.input_handler.update_keys(&mut self.events_loop);
    }

    ///Returns the input handler
    #[inline]
    pub fn get_input_handler(&mut self) -> &mut input_handler::InputHandler{
        &mut self.input_handler
    }

    ///Creates a copy of the current key map
    #[inline]
    pub fn get_key_map_copy(&self) -> keymap::KeyMap{
        //get the map
        let key_map = {
            //lock
            let tmp_map = self.key_map.lock().expect("failed to lock keymap for copy return");
            //copy
            (*tmp_map).clone()
        };
        //return it
        key_map
    }

    ///Returns a mutable reference to the current key map copy.
    ///NOTE: This copy will be overwritten from time to time by the input thread.
    #[inline]
    pub fn get_key_map(&self) -> Arc<Mutex<keymap::KeyMap>>{
        self.key_map.clone()
    }


}

//TODO Implement the other keys
/*
F1,
F2,
F3,
F4,
F5,
F6,
F7,
F8,
F9,
F10,
F11,
F12,
F13,
F14,
F15,
Snapshot,
Scroll,
Pause,
Insert,
Home,
Delete,
End,
PageDown,
PageUp,
Compose,
AbntC1,
AbntC2,
Add,
Apostrophe,
Apps,
At,
Ax,
Backslash,
Calculator,
Colon,
Comma,
Convert,
Decimal,
Divide,
Equals,
Grave,
Kana,
Kanji,
LBracket,
LMenu,
Mail,
MediaSelect,
MediaStop,
Minus,
Multiply,
Mute,
MyComputer,
NavigateForward,
NavigateBackward,
NextTrack,
NoConvert,
NumpadComma,
NumpadEnter,
NumpadEquals,
OEM102,
Period,
PlayPause,
Power,
PrevTrack,
RBracket,
RMenu,
Semicolon,
Slash,
Sleep,
Stop,
Subtract,
Sysrq,
Underline,
Unlabeled,
VolumeDown,
VolumeUp,
Wake,
WebBack,
WebFavorites,
WebForward,
WebHome,
WebRefresh,
WebSearch,
WebStop,
Yen,
    */
