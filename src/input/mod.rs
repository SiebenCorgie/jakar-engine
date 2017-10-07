use std::sync::{Mutex, Arc};
use std::thread;
use std::time::Duration;

use winit;

use core::engine_settings;

///a sub mod who will read the input since the last loop
///and store the key values in a struct
pub mod input_handler;



///The struct stores the current pressed keys
#[derive(Debug, Copy, Clone)]
pub struct KeyMap {

    //WINDOW
    ///Window info (usually not needed recreation is handled by renderer)
    pub window_dimensions: [u32; 2],

    //GLOBAL
    ///Global States
    pub closed: bool,

    //MOUSE
    //moving
    ///Represents the current location of the mouse
    pub mouse_location: [i32; 2],
    ///represents the current active delta of mouse mouement, this can be used to implement mouse
    ///speed dependent movement like camera-rotation
    pub mouse_delta_x: f64,
    //same as `mouse_delta_x` for axis-y
    pub mouse_delta_y: f64,


    //KEYBOARD
    //normal keys
    pub a: bool,
    pub b: bool,
    pub c: bool,
    pub d: bool,
    pub e: bool,
    pub f: bool,
    pub g: bool,
    pub h: bool,
    pub i: bool,
    pub j: bool,
    pub k: bool,
    pub l: bool,
    pub m: bool,
    pub n: bool,
    pub o: bool,
    pub p: bool,
    pub q: bool,
    pub r: bool,
    pub s: bool,
    pub t: bool,
    pub u: bool,
    pub v: bool,
    pub w: bool,
    pub x: bool,
    pub y: bool,
    pub z: bool,
    //numbers on the top
    pub t_1: bool,
    pub t_2: bool,
    pub t_3: bool,
    pub t_4: bool,
    pub t_5: bool,
    pub t_6: bool,
    pub t_7: bool,
    pub t_8: bool,
    pub t_9: bool,
    pub t_0: bool,
    //numblock
    pub num_1: bool,
    pub num_2: bool,
    pub num_3: bool,
    pub num_4: bool,
    pub num_5: bool,
    pub num_6: bool,
    pub num_7: bool,
    pub num_8: bool,
    pub num_9: bool,
    pub num_0: bool,
    //Main controll keys
    pub ctrl_l: bool,
    pub ctrl_r: bool,
    pub alt_l: bool,
    pub alt_r: bool,
    pub super_l: bool,
    pub super_r: bool,
    pub caps_lock: bool,
    pub shift_l: bool,
    pub shift_r: bool,
    pub tab: bool,
    pub space: bool,
    pub enter: bool,
    pub nume_enter: bool,
    pub escape: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,

    //todo addrest
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


}

impl KeyMap{
    pub fn new() -> Self{
        KeyMap{
            //window info
            window_dimensions: [100, 100],
            //state
            closed: false,

            mouse_location: [0; 2],
            mouse_delta_x: 0.0,
            mouse_delta_y: 0.0,

            //normal keys
            a: false,
            b: false,
            c: false,
            d: false,
            e: false,
            f: false,
            g: false,
            h: false,
            i: false,
            j: false,
            k: false,
            l: false,
            m: false,
            n: false,
            o: false,
            p: false,
            q: false,
            r: false,
            s: false,
            t: false,
            u: false,
            v: false,
            w: false,
            x: false,
            y: false,
            z: false,
            //numbers on the top
            t_1: false,
            t_2: false,
            t_3: false,
            t_4: false,
            t_5: false,
            t_6: false,
            t_7: false,
            t_8: false,
            t_9: false,
            t_0: false,
            //numblock
            num_1: false,
            num_2: false,
            num_3: false,
            num_4: false,
            num_5: false,
            num_6: false,
            num_7: false,
            num_8: false,
            num_9: false,
            num_0: false,
            //Main controll keys
            ctrl_l: false,
            ctrl_r: false,
            alt_l: false,
            alt_r: false,
            super_l: false,
            super_r: false,
            caps_lock: false,
            shift_l: false,
            shift_r: false,
            tab: false,
            space: false,
            enter: false,
            nume_enter: false,
            escape: false,
            //arrows
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }
}


///Manages all input
pub struct Input {
    input_handler: input_handler::InputHandler,
    events_loop: Arc<Mutex<winit::EventsLoop>>,
    settings: Arc<Mutex<engine_settings::EngineSettings>>,
    pub key_map: Arc<Mutex<KeyMap>>,
}


impl Input{
    ///Creates a new Input instance
    pub fn new(settings: Arc<Mutex<engine_settings::EngineSettings>>) -> Self{

        let key_map_inst = Arc::new(Mutex::new(KeyMap::new()));

        let events_loop = Arc::new(Mutex::new(winit::EventsLoop::new()));

        Input{
            input_handler: input_handler::InputHandler::new(key_map_inst.clone(), events_loop.clone(), settings.clone()),
            events_loop: events_loop,
            settings: settings,
            key_map: key_map_inst.clone(),
        }
    }

    ///Starts the input polling thread
    #[inline]
    pub fn start(&mut self) -> thread::JoinHandle<()> {
        self.input_handler.start()
    }

    ///Ends the input polling thread, should be done when exiting the the main loop
    #[inline]
    pub fn end(&mut self){
        self.input_handler.end();

        //Wait some mil seconds so the thread has time to end
        thread::sleep(Duration::from_millis(1000));
    }

    ///Returns the Events loop, used for renderer creation
    #[inline]
    pub fn get_events_loop(&mut self) -> Arc<Mutex<winit::EventsLoop>>{
        self.events_loop.clone()
    }

    ///Returns the input handler
    #[inline]
    pub fn get_input_handler(&mut self) -> &mut input_handler::InputHandler{
        &mut self.input_handler
    }

    ///Creates a copy of the current key map
    #[inline]
    pub fn get_key_map_copy(&self) -> KeyMap{
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
    pub fn get_key_map(&self) -> Arc<Mutex<KeyMap>>{
        self.key_map.clone()
    }


}
