


///Can be used to change between variouse debug views. They are actually applied in the
///post progress stage
#[derive(Clone)]
pub enum DebugView {
    ClusterId,
    HeatMap,
    MainDepth,
    Shaded,
}

impl DebugView{
    pub fn as_shader_int(&self) -> i32{
        match self{
            &DebugView::ClusterId => 0,
            &DebugView::HeatMap => 1,
            &DebugView::MainDepth => 2,
            &DebugView::Shaded => 3,
        }
    }
}

///BlurSettings.
#[derive(Clone)]
pub struct BlurSettings {
    pub strength: f32,
    pub scale: f32,
}

impl BlurSettings{
    pub fn new(scale: f32, strength: f32) -> Self{
        BlurSettings{
            strength: strength,
            scale: scale
        }
    }
}

///Descibes settings the renderer can have. Most of the values can't be changed after
/// starting the engine.
///Things to keep in mind:
///
/// - if you turn hdr off you won't have any bloom effects as well.
#[derive(Clone)]
pub struct RenderSettings {
    ///Keeps track of the render settings status. Is true if something changed
    has_changed: bool,
    ///filtering options, should be power of two between 1 and 16
    anisotropic_filtering: f32,
    ///Samples for each pixel, should be power of two between 1 and 16 (but can be higher)
    msaa: u32,
    ///Is true if a "fifo" presentmode of the swapchain should be forced.
    v_sync: bool,

    ///Defines the current gamma correction on the output
    gamma: f32,
    ///Defines the exposure used to correct the HDR image down to LDR
    exposure: f32,

    ///Defines the blur settings. Mainly strength and scale.
    blur: BlurSettings,

    ///The engine should render the bounds
    debug_bounds: bool,

    ///Describes what the post_progress should render
    debug_view: DebugView,

    //TODO: add things like "max render distance" for objects

}


impl RenderSettings{
    ///Creates a default, medium good set of render settings:
    /// Defaults:
    ///
    /// - anisotropic_filtering: 1.0,
    /// - msaa: 1,
    /// - v_sync: false,
    /// - gamma: 2.2,
    /// - exposure: 1.0
    /// - max_point_lights: 512,
    /// - max_dir_lights: 6,
    /// - max_spot_lights: 512,

    /// *No debug turned on*
    pub fn default() -> Self{
        RenderSettings{
            has_changed: false,
            anisotropic_filtering: 1.0,
            msaa: 1,
            v_sync: false,
            gamma: 2.2,
            exposure: 1.0,
            blur: BlurSettings{
                strength: 1.5,
                scale: 1.0,
            },

            debug_bounds: false,
            debug_view: DebugView::Shaded,

        }
    }

    ///Returns true if the render settings have changed but the renderer did not mark the changes
    /// as resolved yet.
    #[inline]
    pub fn render_settings_changed(&self) -> bool{
        self.has_changed
    }

    ///Sets the changes as resolved
    #[inline]
    pub fn resolved_settings(&mut self){
        self.has_changed = false;
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

    ///Returns the current view type.
    #[inline]
    pub fn get_debug_view(&self) -> &DebugView{
        &self.debug_view
    }

    ///sets the view type to be used for the following frames
    #[inline]
    pub fn set_debug_view(&mut self, new: DebugView){
        self.debug_view = new;
    }


    ///Sets up a custom anisotropical filtering factor, should be a power of two as well, otherwise
    /// it won't be used
    #[inline]
    pub fn with_msaa_factor(mut self, msaa_factor: u32) -> Self{
        if !test_for_power_of_two(msaa_factor) || msaa_factor > 16{ //TODO verfy the size
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

    ///set the v_sync falue to either true or false. However the engine will check if
    /// Vulkans Immidiate present mode is supported, if v_sync is turned of. If it is not, V_Sync
    /// will be used (always supported).
    #[inline]
    pub fn with_vsync(mut self, value: bool) -> Self{
        self.v_sync = value;
        self
    }

    ///Sets the vsync mode at runtime. However this won't have an effect if the renderer has already
    /// been started.
    pub fn set_vsync(&mut self, value: bool){
        self.v_sync = value;
    }

    ///Returns the current v_sync status. Will be changed to false at runtime if non-vsync presenting
    /// is not supported.
    #[inline]
    pub fn get_vsync(&self) -> bool{
        self.v_sync
    }



    ///Sets up a custom gamma value.
    #[inline]
    pub fn with_gamma(mut self, gamma: f32) -> Self{
        self.gamma = gamma;
        self
    }
    ///Returns the current gamma factor.
    #[inline]
    pub fn get_gamma(&self) -> f32{
        self.gamma
    }

    ///Sets the current gamma factor to `new` and marks the settings as unresolved.
    #[inline]
    pub fn set_gamma(&mut self, new: f32){
        self.gamma = new;
        self.has_changed = true;
    }

    ///Adds an ammount to gamma. NOTE: the gamma can't be below 0.0 all values beleow will
    /// be clamped to 0.0
    #[inline]
    pub fn add_gamma(&mut self, offset: f32){

        if self.gamma - offset >= 0.0{
            self.gamma -= offset;
        }else{
            self.gamma = 0.0
        }
        self.has_changed = true;
    }

    ///Sets up a custom exposure value. When below 1.0 bright areas will be even brighter but dark
    /// areas will be brighter as well.
    /// When above 1.0 the dark areas might seam black but the bright areas are more defined.
    #[inline]
    pub fn with_exposure(mut self, exposure: f32) -> Self{
        self.exposure = exposure;
        self
    }
    ///Returns the current exposure.
    #[inline]
    pub fn get_exposure(&self) -> f32{
        self.exposure
    }

    ///Sets the current exposure to `new` and marks the settings as unresolved.
    #[inline]
    pub fn set_exposure(&mut self, new: f32){
        self.exposure = new;
        self.has_changed = true;
    }

    ///Adds an ammount to exposure. NOTE: the exposure can't be below 0.0 all values beleow will
    /// be clamped to 0.0

    pub fn add_exposure(&mut self, offset: f32){

        if self.exposure - offset >= 0.0{
            self.exposure -= offset;
        }else{
            self.exposure = 0.0
        }
        self.has_changed = true;
    }


    ///Sets the current blur settings. Don't overdo it or your rendered image will look like a Michael Bay movie.
    #[inline]
    pub fn with_blur(mut self, scale: f32, strength: f32) -> Self{
        self.blur = BlurSettings::new(scale, strength);
        self
    }

    ///Sets the current blur settings. Don't overdo it or your rendered image will look like a Michael Bay movie.
    #[inline]
    pub fn set_blur(&mut self, scale: f32, strength: f32){
        self.blur = BlurSettings::new(scale, strength);
    }

    ///Returns the current blur settings. They might change per frame.
    #[inline]
    pub fn get_blur(&self,) -> BlurSettings{
        self.blur.clone()
    }


    ///If `new` is `true`, the bounds of each renderable object are drawn.
    #[inline]
    pub fn set_debug_bound(&mut self, new: bool){
        self.debug_bounds = new;
    }

    ///Returns true if the bounds should be drawn
    pub fn draw_bounds(&self) -> bool{
        self.debug_bounds
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
