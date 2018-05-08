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
    //f-keys
    pub f1: bool,
    pub f2: bool,
    pub f3: bool,
    pub f4: bool,
    pub f5: bool,
    pub f6: bool,
    pub f7: bool,
    pub f8: bool,
    pub f9: bool,
    pub f10: bool,
    pub f11: bool,
    pub f12: bool,
    pub f13: bool,
    pub f14: bool,
    pub f15: bool,
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
            //f-keys
            f1: false,
            f2: false,
            f3: false,
            f4: false,
            f5: false,
            f6: false,
            f7: false,
            f8: false,
            f9: false,
            f10: false,
            f11: false,
            f12: false,
            f13: false,
            f14: false,
            f15: false,
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
    ///Resets data which has only a singel callback value
    pub fn reset_data(&mut self){
        self.mouse_location = [0; 2];
        self.mouse_delta_x = 0.0;
        self.mouse_delta_y = 0.0;
    }
}
