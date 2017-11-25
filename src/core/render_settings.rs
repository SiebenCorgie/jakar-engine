///Descibes settings the renderer can have. Most of the values can't be changed after
/// starting the engine.
///Things to keep in mind:
///
/// - if you turn hdr off you won't have any bloom effects as well.
#[derive(Clone)]
pub struct RenderSettings {
    ///filtering options, should be power of two between 1 and 16
    anisotropic_filtering: f32,
    ///Samples for each pixel, should be power of two between 1 and 16 (but can be higher)
    msaa: u32,
    ///true if the engine should render in hdr mode. This will render the image in hdr
    ///and the perform a converion to a normal 8 bit image.
    hdr: bool,
    ///true if it should be possible to perform a postprogressing shader on the render output
    postprogress: bool,

    //TODO: add things like "max render distance" for objects
}


impl RenderSettings{
    ///Creates a default, medium good set of render settings:
    /// TODO write default settings here
    pub fn default() -> Self{
        RenderSettings{
            anisotropic_filtering: 1.0,
            msaa: 1,
            hdr: true,
            postprogress: true,
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
    ///Returns the current filter size. Is always a power of two.
    #[inline]
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
    ///Returns the current msaa factor. Is always a power of two.
    #[inline]
    pub fn get_msaa_factor(&self) -> u32{
        self.msaa
    }

    ///Set the hdr state. If true the engine will render an 16bit hdr iamge first and then converts
    /// it down to a 8bit image before presenting it to the swapchain. You should leave this on
    /// for good lighing results
    #[inline]
    pub fn with_hdr(mut self, hdr_state: bool) -> Self{
        self.hdr = hdr_state;
        self
    }

    ///Returns the current hdr state.
    #[inline]
    pub fn has_hdr(&self) -> bool{
        self.hdr
    }

    ///If set to true the engine will perform a postprogessing step after rendering the final
    /// image.
    #[inline]
    pub fn with_post_progress(mut self, state: bool) -> Self{
        self.postprogress = state;
        self
    }

    ///Returns the current post progressing state.
    #[inline]
    pub fn has_post_progress(&self) -> bool{
        self.postprogress
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
