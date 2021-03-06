


///Can be used to change between variouse debug views. They are actually applied in the
///post progress stage
#[derive(Clone)]
pub enum DebugView {
    MainDepth,
    DebugGrid,
    ShadowMaps,
    DirectionalDepth,
    Shaded,
}

impl DebugView{
    pub fn as_shader_int(&self) -> i32{
        match self{
            &DebugView::MainDepth => 0,
            &DebugView::DebugGrid => 1,
            &DebugView::ShadowMaps => 2,
            &DebugView::DirectionalDepth => 3,
            &DebugView::Shaded => 4,
        }
    }
}

///Describes several settings which are used while debugging.
#[derive(Clone)]
pub struct DebugSettings {
    ///Can be used to switch between debug views.
    pub debug_view: DebugView,
    ///Describes which level of the scaled ldr image should be shown when in
    pub ldr_debug_view_level: u32,
    ///Is true if you want to let the engine draw the bounds of the objects
    pub draw_bounds: bool,
}

///BloomSettings.
#[derive(Clone)]
pub struct BloomSettings {
    ///more levels means nicer bloom but worse performance1
    pub levels: u32,
    ///Describes what size the initial blur image should be scaled down.
    pub first_bloom_level: u32,
    ///How big the bloom should be. This value should mostly be below 1.0.
    pub size: f32,
    ///How bright the added blure samples are. 1.0 is realy bright. *The values should be between 1.0 and 0.0*
    pub brightness: f32
}

impl BloomSettings{
    pub fn new(levels: u32, scale_down: u32, size: f32, brightness: f32) -> Self{
        BloomSettings{
            levels,
            first_bloom_level: scale_down,
            size,
            brightness
        }
    }
}

///All settings needed for the auto exposure to work. Howevcer, there is an option to use
/// no auto exposure. If it is turned on, the engine will use the min_exposure setting always.
#[derive(Clone)]
pub struct ExposureSettings {
    ///The lower cap for exposure. The exposure can't be lower then this.
    pub min_exposure: f32,
    ///Same as min, but for the upper cap.
    pub max_exposure: f32,
    ///The speed the exposure gets corrected upwards (when the image is too dark). Should be a bit faster then the downb scaling.
    pub scale_up_speed: f32,
    ///Same as up, but for correcting down.
    pub scale_down_speed: f32,
    ///Tells the system which target lumiosity a frame shoudl have
    pub target_lumiosity: f32,
    ///If false, the implementation will use a static value which it gets from `min_exposure`.
    pub use_auto_exposure: bool,
}


impl ExposureSettings{
    pub fn new(min: f32, max: f32, up_speed: f32, down_speed: f32, target_lumiosity: f32, use_auto: bool) -> Self{
        ExposureSettings{
            min_exposure: min,
            max_exposure: max,
            scale_up_speed: up_speed,
            scale_down_speed: down_speed,
            target_lumiosity: target_lumiosity,
            use_auto_exposure: use_auto,
        }
    }
}

///Collects all settings for directional lights, Note that the light currently has set the
/// number of cascades to four (not changeable)
#[derive(Clone)]
pub struct DirectionalLightSettings {
    ///Describes how many samples should be taken in each direction when calculating the shadow
    pcf_samples: u32,
    ///Describes the resolution of a single cascade in the directonal shadow map
    shadow_map_resolution: u32,
    /// number of cascades the shadow can have
    num_cascades: u32,
    /// Controlles at which point cascades are split in the logarithmic function.
    cascade_lambda: f32,
    ///Controles how many percent on the shadowmap a mesh must occupy to be actually rendered.
    /// for high settings 0.1 is a good value. You can controll each cascade.
    occupy_bias: [f32; 4],
    /// Describes how much the different samples are "spread" over the uv. A high numerb means less
    /// noise, but also harder shadows and therefor better visible pixels.
    poisson_spread: f32
}

impl DirectionalLightSettings{
    ///Creates a custom set of settings
    pub fn new(pcf_samples: u32, resolution: u32, cascade_lambda: f32, occupy_bias: [f32; 4], poisson_spread: f32) -> Self{
        DirectionalLightSettings{
            pcf_samples: pcf_samples,
            shadow_map_resolution: resolution,
            num_cascades: 4,
            cascade_lambda: cascade_lambda,
            occupy_bias: occupy_bias,
            poisson_spread: poisson_spread,
        }
    }

    ///Creates the default set of settings:
    /// - pcf samples: 9
    /// - shadow map resoltion: 1024 (each cascade)
    /// - num_cascades: 4 (can't be changed)
    /// - cascade_lambda: 0.95
    /// - occupy_bias: 0.1
    pub fn default() -> Self{
        DirectionalLightSettings{
            pcf_samples: 2,
            shadow_map_resolution: 1024,
            num_cascades: 4,
            cascade_lambda: 0.95,
            occupy_bias: [0.1; 4],
            poisson_spread: 800.0,
        }
    }
    /// Always returns 4 for now.
    pub fn get_num_cascades(&self) -> u32{
        self.num_cascades
    }
    /// Controlles the resolution of the shadow map. A single cascaded always has half of that
    /// resolution, since all 4 cascades are stored on the same shadowmap.
    pub fn get_shadow_map_resolution(&self) -> u32{
        self.shadow_map_resolution
    }
    pub fn set_shadow_map_resolution(&mut self, new: u32){
        self.shadow_map_resolution = new;
    }
    /// Describes how many samples are taken in each direction when rendering the shadows
    /// More samples means smoother shadows but also impacts the fps heavily.
    pub fn set_pcf_samples(&mut self, new: u32){
        self.pcf_samples = new;
    }
    pub fn get_pcf_samples(&self) -> u32{
        self.pcf_samples
    }
    /// Controlles at which point cascades are split in the logarithmic function.
    pub fn set_cascade_lambda(&mut self, new: f32){
        self.cascade_lambda = new;
    }
    /// a typical value is 0.95.
    pub fn get_cascade_lambda(&self) -> f32{
        self.cascade_lambda
    }
    ///Controles how many percent on the shadowmap a mesh must occupy to be actually rendered.
    /// for high settings 0.1 is a good value.
    pub fn set_occupy_bias(&mut self, new: [f32; 4]){
        self.occupy_bias = new;
    }

    pub fn get_occupy_bias(&self) -> [f32; 4]{
        self.occupy_bias
    }

    ///Controlles how much the different samples for the shadows a spread over the shadow map.
    ///A higher value means less noise buit also more visible and harder edges on the shadow.
    pub fn set_poisson_spread(&mut self, new: f32){
        self.poisson_spread = new;
    }

    pub fn get_poisson_spread(&self) -> f32{
        self.poisson_spread
    }

}

///Defines variouse settings for point lights
pub struct PointLightSettings {
    pcf_samples: i32,
    shadow_map_resolution: i32,
    occupy_bias: f32,
}

impl PointLightSettings{
    ///Creates a new set of settings from the supplied values
    pub fn new(pcf_samples: i32, shadow_map_resolution: i32, occupy_bias: f32) -> Self{
        PointLightSettings{
            pcf_samples,
            shadow_map_resolution,
            occupy_bias
        }
    }

    ///Creates a default set as follows:
    /// - pcf_samples: 2
    /// - shadow_map_resolution: 1024
    /// - occupy_bias: 0.1
    pub fn default() -> Self{
        PointLightSettings{
            pcf_samples: 2,
            shadow_map_resolution: 1024,
            occupy_bias: 0.1
        }
    }
}

///Defines several settings which will be used to determin how lights and their shadows are rendered
#[derive(Clone)]
pub struct LightSettings {
    pub directional_settings: DirectionalLightSettings
}

impl LightSettings{
    ///Creates a custom set of light settings
    pub fn new(directional_light: DirectionalLightSettings) -> Self{
        LightSettings{
            directional_settings: directional_light,
        }
    }

    ///Creates the default settings, see the impls of the different settings to see them.
    pub fn default() -> Self{
        LightSettings{
            directional_settings: DirectionalLightSettings::default(),
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
    exposure: ExposureSettings,

    ///Collects all settings related to light and shadows
    light_settings: LightSettings,

    ///Defines the bloom settings. Mainly strength and scale.
    bloom: BloomSettings,

    ///Describes the several debug settings one cna change
    debug_settings: DebugSettings,

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
            exposure: ExposureSettings::new(
                0.2, 4.0, 0.002, 0.003, 1.0, true
            ),
            light_settings: LightSettings::default(),

            bloom: BloomSettings{
                levels: 8,
                first_bloom_level: 4, //nice performance and look
                size: 0.05,
                brightness: 1.0,
            },

            debug_settings: DebugSettings{
                draw_bounds: false,
                debug_view: DebugView::Shaded,
                ldr_debug_view_level: 0,
            }
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

    ///Returns the current debug settings as mutable.
    #[inline]
    pub fn get_debug_settings(&self) -> &DebugSettings{
        &self.debug_settings
    }

    ///Returns the current debug settings as mutable.
    #[inline]
    pub fn get_debug_settings_mut(&mut self) -> &mut DebugSettings{
        &mut self.debug_settings
    }

    ///sets the view type to be used for the following frames
    #[inline]
    pub fn set_debug_settings(&mut self, new: DebugSettings){
        self.debug_settings = new;
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
    pub fn with_exposure(mut self, exposure: ExposureSettings) -> Self{
        self.exposure = exposure;
        self
    }
    ///Returns the current exposure.
    #[inline]
    pub fn get_exposure(&self) -> ExposureSettings{
        self.exposure.clone()
    }

    ///Sets the current exposure to `new` and marks the settings as unresolved.
    #[inline]
    pub fn get_exposure_mut(&mut self) -> &mut ExposureSettings{
        &mut self.exposure
    }

    ///Returns the current light settings as a clone.
    #[inline]
    pub fn get_light_settings(&self) -> LightSettings{
        self.light_settings.clone()
    }

    ///Returns the current light settings as mutable reference.
    #[inline]
    pub fn get_light_settings_mut(&mut self) -> &mut LightSettings{
        &mut self.light_settings
    }

    ///Sets the current light settings
    #[inline]
    pub fn set_light_settings(&mut self, new: LightSettings){
        self.light_settings = new;
    }

    ///Sets the current light settings when building the rendering settings
    #[inline]
    pub fn with_light_settings(mut self, new: LightSettings) -> Self{
        self.light_settings = new;
        self
    }

    ///Sets the current bloom settings. Don't overdo it or your rendered image will look like a Michael Bay movie.
    ///Also for performance, try to increase the initial scale down
    #[inline]
    pub fn with_bloom(mut self, levels: u32, first_bloom_level: u32, size: f32, brightness: f32) -> Self{
        let mut using_first_level = first_bloom_level;
        if first_bloom_level >= levels{
            using_first_level = levels - 1;
        }

        let mut used_brightness = brightness;
        if brightness > 1.0{
            used_brightness = 1.0;
        }
        if brightness < 0.0 {
            used_brightness = 0.0
        }


        self.bloom = BloomSettings::new(levels, using_first_level, size, used_brightness);
        self
    }

    ///Sets the current bloom settings. Don't overdo it or your rendered image will look like a Michael Bay movie.
    #[inline]
    pub fn set_bloom(&mut self, levels: u32, first_bloom_level: u32, size: f32, brightness: f32){
        let mut using_first_level = first_bloom_level;
        if first_bloom_level >= levels{
            using_first_level = levels - 1;
        }

        let mut used_brightness = brightness;
        if brightness > 1.0{
            used_brightness = 1.0;
        }
        if brightness < 0.0 {
            used_brightness = 0.0
        }
        self.bloom = BloomSettings::new(levels, using_first_level, size, used_brightness);
    }

    ///Returns the current bloom settings. They might change per frame.
    #[inline]
    pub fn get_bloom(&self) -> BloomSettings{
        self.bloom.clone()
    }

    ///Returns the current bloom settings. They might change per frame.
    #[inline]
    pub fn get_bloom_mut(&mut self) -> &mut BloomSettings{
        &mut self.bloom
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
