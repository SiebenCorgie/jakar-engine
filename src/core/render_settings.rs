///Descibes settings the renderer can have. Most of the values can't be changed after
/// starting the engine.
#[derive(Clone)]
pub struct RenderSettings {
    ///filtering options, should be power of two between 1 and 16
    anisotropic_filtering: f32,
    ///Samples for each pixel, should be power of two between 1 and 16 (but can be higher)
    msaa: u32,

    //TODO: add things like "max render distance" for objects 
}


impl RenderSettings{
    ///Creates a default, medium good set of render settings:
    /// TODO write default settings here
    pub fn default() -> Self{
        RenderSettings{
            anisotropic_filtering: 1.0,
            msaa: 1,
        }
    }

    ///Setts the anisotropical filtering factor, leaves it at the current value if it isn't between
    /// 1-16 and a power of two.
    ///Sets up a custom anisotropical filtering factor
    #[inline]
    pub fn with_anisotropical_filtering(mut self, af_factor: u32)-> Self{

        if !test_for_power_of_two(af_factor) || af_factor > 16{
            return self;
        }

        self.anisotropic_filtering = af_factor as f32;
        self
    }

    pub fn get_anisotropical_filtering(&self) -> f32{
        self.anisotropic_filtering
    }


    ///Sets up a custom anisotropical filtering factor, should be a power of two as well, otherwise
    /// it won't be used
    #[inline]
    pub fn with_msaa_factor(mut self, msaa_factor: u32) -> Self{
        if !test_for_power_of_two(msaa_factor){
            return self;
        }
        self.msaa = msaa_factor;
        self
    }

    pub fn get_msaa_factor(&self) -> u32{
        self.msaa
    }



}

///Tests for power of two
fn test_for_power_of_two(num: u32) -> bool{
    let mut walker: i32 = num as i32;
    //test if we can dev by two and don't get a remainder and if we are > 1
    while walker > 1 && (walker % 2 == 0){
        walker /= 2;
    }
    //if the last walker was 1 it is a power of two otherwise we got something like 0.5 or 1.5;
    (walker == 1)
}
