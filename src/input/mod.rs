
use std::time::{Instant, Duration};
use std::result::Result;

use std::sync::{Mutex, Arc};
use std::sync::mpsc::*;
use std::thread::*;

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

///Describes messages which can be send to the input thread. They are used to controll mostly
/// the shutdown of this system.
enum InputThreadMessages {
    ///Tells the thread to sleep for a certain ammount of milliseconds.
    Sleep(Duration),
    ///Changes the max polling speed to the ammount/second.
    ChangePollingSpeed(u32),
    End,
}

///Manages all input
///TODO implement the message api
pub struct Input {
    ///The handle of the input thread.
    settings: Arc<Mutex<engine_settings::EngineSettings>>,
    pub key_map: Arc<Mutex<keymap::KeyMap>>,
    msg_send: Sender<InputThreadMessages>,
    input_thread: Option<JoinHandle<()>>,

    status: InputState,

}


impl Input{
    ///Creates a new Input instance. It needs to recive the vulkan instance which will target the
    /// resulting window.
    pub fn new(
        settings: Arc<Mutex<engine_settings::EngineSettings>>,
        instance: Arc<Instance>,
    ) -> Result<(Self, Window), String>{


        //Create the global keymap which gets updated from the input loop
        let key_map = Arc::new(Mutex::new(keymap::KeyMap::new()));
        let key_map_inst = key_map.clone();
        let settings_inst = settings.clone();
        //now spawn the actual input loop as well as the communication channels.
        let (msg_sender, msg_reciver) = channel::<InputThreadMessages>();
        let (window_sender, window_reciver) = channel::<Window>();

        let initial_speed = {
            let settings_lck = settings.lock().expect("failed to lock settings for input loop");
            settings_lck.max_input_speed
        };

        let input_thread = spawn(move||{

            let mut input_handler = input_handler::InputHandler::new(
                key_map_inst.clone(), settings_inst.clone()
            );
            //now create a window for this loop and send it back
            let window = Window::new(
                &instance, &input_handler.get_events_loop(), settings_inst
            );
            //now send the window back
            window_sender.send(window).expect("failed to send window to main thread!");

            //Set initial variables
            let mut polling_speed = initial_speed;
            let mut last_time = Instant::now();

            'input_loop: loop {
                //Have a look for new events
                match msg_reciver.try_recv(){
                    Ok(msg) => {
                        match msg{
                            InputThreadMessages::End => break, //need to end this thread
                            InputThreadMessages::ChangePollingSpeed(new_speed) => polling_speed = new_speed,
                            InputThreadMessages::Sleep(time) => sleep(time),
                        }
                    },
                    Err(err) => {
                        match err {
                            TryRecvError::Empty => {} //all right
                            TryRecvError::Disconnected => {
                                //Lost our input (theirfore the main thread)
                                break;
                            }
                        }
                    }
                }

                //We handled all messages, time to poll the events and then wait the rest time
                input_handler.update_keys();
                //Sleep and update timer
                last_time = super::sleep_rest_time(last_time, polling_speed);
            }
        });

        //recive the window at some time
        let window = {
            match window_reciver.recv(){
                Ok(win) => win,
                Err(er) => return Err(String::from("failed to recive window!")),
            }
        };

        //Finally build the input system and return
        Ok(
            (
                Input{
                    settings: settings,
                    key_map: key_map,
                    status: InputState::Running,
                    msg_send: msg_sender,
                    input_thread: Some(input_thread),
                },
                window
            )
        )
    }

    ///Tells the render loop to sleep for `duration`
    pub fn input_sleep(&mut self, duration: Duration){
        self.msg_send.send(
            InputThreadMessages::Sleep(duration)
        ).expect("failed to send sleep message to input thread");
    }

    ///Changes the pollinspeed of the input thread
    pub fn change_polling_speed(&mut self, new_speed: u32){
        self.msg_send.send(
            InputThreadMessages::ChangePollingSpeed(new_speed)
        ).expect("failed to change polling speed");
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

impl Drop for Input{
    fn drop(&mut self){
        //End the input thread then join it
        self.msg_send.send(
            InputThreadMessages::End
        ).expect("failed to end input thread, could'nt send end");
        let thread = self.input_thread.take();
        if let Some(thr) = thread{
            thr.join().expect("Failed to end input thread, no join.");
        }else{
            println!("Input thread handle was already invalide!", );
        }
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
