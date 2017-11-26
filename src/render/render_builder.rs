use vulkano;
use vulkano::image::attachment::AttachmentImage;
use vulkano::framebuffer::FramebufferAbstract;
use winit;
use vulkano::sync::GpuFuture;
use vulkano_win;
use vulkano::instance::debug::{DebugCallback, MessageTypes};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::format::Format;

use std::sync::{Arc, Mutex};

use render;
use render::pipeline_manager;
use render::uniform_manager;
use render::window;
use render::frame_system;
use core::render_settings;
use core::engine_settings;
use input::KeyMap;

///Describes how the handler should load the layers, by default set to NoLayer
pub enum LayerLoading{
    ///Should try to load all available layers.
    All,
    ///Should not load any layer at all
    NoLayer,
    ///Should try to load the layers in this vector
    Load(Vec<String>),
}


///This struct saves the configuration options for the renderer and rebuilds it when needed.
pub struct RenderBuilder {
    ///Describes the extensions needed to work properly. By **default** its only the extensions needed
    /// To create the window surface. The craetion will fail if those aren't met.
    pub physical_extensions_needed: vulkano::instance::InstanceExtensions,
    ///Describes the extensions needed from the abstract device. **Default: only the swapchain is needed***
    pub logical_extensions_needed: vulkano::device::DeviceExtensions,

    /// Describes the loaded layers. NOTE: First, the builder only tries to load the layer presented
    /// by an enum of the type `LayerLoading::Load`. If it is not present, it doesnt get loaded.
    /// Second: If the engine settings indicate that the engine is in release mode, no layer will be
    /// loaded.
    /// **Default: No layers are loaded**
    pub layer_loading: LayerLoading,
    ///Can be used to set which vulkan messages are printed. NOTE: when the engine is in release mode
    /// this settings will be overwritten by the `none()` function.
    /// **Default: only errors are printed**
    pub vulkan_messages: vulkano::instance::debug::MessageTypes,
    ///Can be set to `Some(String)` where `String` is the name of a preferred physical device.
    /// If this is `None`, a small ranking between the available options will decide.
    /// **DEFAULT: None**
    pub preferred_physical_device: Option<String>,
    ///A set of minimal features the vulkan instance has to have. The creation will fail if this
    /// is not met.
    /// **Default: No Features are needed **
    pub minimal_features: vulkano::instance::Features,
}


impl RenderBuilder {
    ///Creates a new default renderer. For the default values, see the struct documentation.
    /// After the creation you are free to change any parameter.
    pub fn new() -> Self{
        //Init the default values
        let physical_extensions_needed = vulkano_win::required_extensions();
        let logical_extensions_needed = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };
        let layers = LayerLoading::NoLayer;
        let vulkan_messages = vulkano::instance::debug::MessageTypes::errors();
        let mut minimal_features = vulkano::instance::Features::none();
        //test add sampling
        minimal_features.sample_rate_shading = true;


        RenderBuilder{
            physical_extensions_needed: physical_extensions_needed,
            logical_extensions_needed: logical_extensions_needed,
            layer_loading: layers,
            vulkan_messages: vulkan_messages,
            preferred_physical_device: None,
            minimal_features: minimal_features,
        }
    }

    ///Creates a render object from this settings.
    /// returns an error if:
    ///
    /// - required physical extensions are not supported
    /// - required logical extensions are not supported
    /// - layers are not supported (if needed)
    /// - no device found
    pub fn create(
        self,
        events_loop: Arc<Mutex<winit::EventsLoop>>,
        engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
        key_map: Arc<Mutex<KeyMap>>,
    ) -> Result<(render::renderer::Renderer, Box<GpuFuture>), String>{
        println!("Starting Vulkan Renderer!", );
        //Init Vulkan
        //Check for needed extensions
        //let mut extensions = vulkano_win::required_extensions();
        //Add the debug extension

        //Generate the list of debuging layers used
        let debuging_layers_string = {
            //Decide based on the engine settings if there should be list. If yes, decide the list
            // based on the builder setting.
            if (*(engine_settings.lock().expect("failed to lock settings in render builder")))
            .build_mode != engine_settings::BuildType::Release
            {
                match self.layer_loading{
                    LayerLoading::All => {
                        let list = vulkano::instance::layers_list().expect("failed to get layer list");
                        let mut ret_list: Vec<String> = Vec::new();
                        for item in list.into_iter(){
                            ret_list.push(item.name().to_string());
                        }
                        ret_list
                    },
                    LayerLoading::NoLayer => {
                        let vec: Vec<String> = Vec::new();
                        //vec.push("".to_string());
                        vec
                    },
                    LayerLoading::Load(try_list) => {
                        let mut ret_vec: Vec<String> = Vec::new();
                        //try out each element
                        for item in vulkano::instance::layers_list().expect("failed to get layer list").into_iter()

                        {
                            for try_item in try_list.clone().into_iter(){
                                if try_item == item.name(){
                                    ret_vec.push(item.name().to_string());
                                }
                            }
                        }
                        ret_vec
                    }
                }
            }else{
                let vec: Vec<String> = Vec::new();
                //vec.push("".to_string());
                vec
            }
        };

        //I don't know a better method which is why we transform the Vec<String> now to a Vec<&str>
        let mut debug_layers = Vec::new();
        for layer in debuging_layers_string.iter(){
            debug_layers.push(layer.as_str());
        }


        //Create an vulkano app info from the settings
        let app_info = {
            use std::borrow::Cow;
            let engine_settings_lck = engine_settings.lock().expect("failed to lock settings");

            let app_name = Some(Cow::Owned((*engine_settings_lck).app_name.clone()));
            let engine_name = Some(Cow::Owned((*engine_settings_lck).engine_name.clone()));

            vulkano::instance::ApplicationInfo{
                application_name: app_name,
                application_version: Some((*engine_settings_lck).app_version.clone()),
                engine_name: engine_name,
                engine_version: Some((*engine_settings_lck).engine_version.clone()),
            }
        };

        println!("Created App Info", );

        //Create a vulkan instance from these extensions
        let try_instance = vulkano::instance::Instance::new(
            Some(&app_info),
            &self.physical_extensions_needed, //TODO verify
            &debug_layers
        );

        //now unwarp our new instance
        let instance = {
            match try_instance {
                Ok(k) => k,
                Err(vkerr) => {
                    println!("Vulkano_err: {}", vkerr);
                    return Err("Failed to create instance!".to_string())
                },
            }
        };

        println!("Created Instance", );

        //now decide for a mesaging service from vulkan, when in release mode, we wont do any
        //if not we read from the builder and construct a callback
        {
            match (*(engine_settings
            .lock()
            .expect("failed to lock engine settings"))).build_mode{
                engine_settings::BuildType::Release => {
                    //we don't call back the errors or warnings
                },
                engine_settings::BuildType::Debug => {
                    //going to print according to the builer
                    //Setup the debug callback for the instance
                    let _debug_callback = DebugCallback::new(&instance, self.vulkan_messages, |msg| {
                        let ty = if msg.ty.error {
                            "error"
                        } else if msg.ty.warning {
                            "warning"
                        } else if msg.ty.performance_warning {
                            "performance_warning"
                        } else if msg.ty.information {
                            "information"
                        } else if msg.ty.debug {
                            "debug"
                        } else {
                            panic!("no-impl");
                        };
                        //println!("STATUS: RENDER: {} {}: {}", msg.layer_prefix, ty, msg.description);
                    }).ok();
                },
                engine_settings::BuildType::ReleaseWithDebugMessages => {
                    //going to print the errors
                    let _debug_callback = DebugCallback::new(
                        &instance, MessageTypes::errors(), |msg|
                        {
                            let ty = if msg.ty.error {
                                "error"
                            } else if msg.ty.warning {
                                "warning"
                            } else if msg.ty.performance_warning {
                                "performance_warning"
                            } else if msg.ty.information {
                                "information"
                            } else if msg.ty.debug {
                                "debug"
                            } else {
                                panic!("no-impl");
                            };
                            //println!("STATUS: RENDER: {} {}: {}", msg.layer_prefix, ty, msg.description);
                        }
                    ).ok();
                }
            }
        }

        //Using a physical device according to the builder settings
        let physical_device_tmp = {
            match self.preferred_physical_device{
                Some(pref_dev) =>{
                    //try to get a device with this name, else rank the devices in the iterator
                    // if the iterator is > 1
                    let mut local_devices = vulkano::instance::PhysicalDevice::enumerate(&instance);
                    if local_devices.len() > 1{
                        rank_devices(local_devices)
                    }else{
                        match local_devices.next(){
                            Some(device) => Some(device),
                            None => return Err("No physical device found!".to_string()),
                        }
                    }
                },
                None => {
                    rank_devices(vulkano::instance::PhysicalDevice::enumerate(&instance))
                }
            }
        };

        //uwnrap the device
        let physical_device = {
            match physical_device_tmp{
                Some(device) => device,
                None => return Err("No physical device found!".to_string()),
            }
        };

        println!("Selected first graphics card", );

        //and create a window for it
        let mut window = {
            let events_loop_unlck = events_loop
            .lock()
            .expect("Failed to hold lock on events loop");
            window::Window::new(
                &instance.clone(), &*events_loop_unlck, engine_settings.clone()
            )
        };
        println!("Opened Window", );

        //Create a queue
        let queue_tmp = physical_device.queue_families().find(
            |&q| q.supports_graphics() &&
            window.surface().is_supported(q).unwrap_or(false)
        );


        let queue = {
            match queue_tmp {
                None => return Err("couldn't find a graphical queue family".to_string()),
                Some(queue) => queue,
            }
        };

        //TODO Test for extensions
        //select needed device extensions
        let device_ext = self.logical_extensions_needed;

        //Ensre that each feature is supported
        if !physical_device.supported_features().superset_of(&self.minimal_features){
            return Err("Not all features are supported!".to_string());
        }

        //Create a artificial device and its queue
        let (device, mut queues) = vulkano::device::Device::new(
            physical_device, &self.minimal_features, //TODO test for needed features and only activate the needed ones
            &device_ext, [(queue, 0.5)].iter().cloned()
        )
        .expect("failed to create device");

        let queue = queues.next().expect("failed to create queue!");

        //Get the swapchain and its images
        let (swapchain, images) = {

            let caps = window.surface()
            .capabilities(physical_device).expect("failed to get surface capabilities");

            //lock settings to read fallback settings
            let engine_settings_lck = engine_settings
            .lock()
            .expect("Failed to lock settings");


            //Set dimensions or fallback to the ones in the settings
            let dimensions = caps.current_extent.unwrap_or((*engine_settings_lck).get_dimensions());
            let usage = caps.supported_usage_flags;
            let format = caps.supported_formats[0].0;

            vulkano::swapchain::Swapchain::new(
                device.clone(),
                window.surface().clone(),
                caps.min_image_count,
                format, //automaticly use the right format for the hardware display
                dimensions,
                1,
                usage,
                &queue,
                vulkano::swapchain::SurfaceTransform::Identity,
                vulkano::swapchain::CompositeAlpha::Opaque,
                vulkano::swapchain::PresentMode::Fifo,
                true,
                None
            )
            .expect("failed to create swapchain")
        };

        for i in images.iter(){
            use vulkano::image::ImageAccess;
            println!("Images have samples: {}", i.samples());
        }

        //Create the uniform manager
        let uniform_manager_tmp = uniform_manager::UniformManager::new(
            device.clone()
        );

        //start the future chain
        let previous_frame = Box::new(vulkano::sync::now(device.clone())) as Box<GpuFuture>;


        println!("Starting frame system", );
        //now create us a default frame system
        let frame_system = frame_system::FrameSystem::new(
            (*(engine_settings.lock().expect("failed to lock settings in render builder"))).get_render_settings_cpy(),
            device.clone(),
            images[0].dimensions(),
            queue.clone(),
            swapchain.format()
        );
        println!("Finished the frame system", );
        //Creates the renderers pipeline manager
        let pipeline_manager = Arc::new(
            Mutex::new(
                pipeline_manager::PipelineManager::new(
                    device.clone(), frame_system.get_renderpass(), 0 //default value atm
                )
            )
        );


        println!("Finished Render Setup", );
        //Pass everthing to the struct
        let renderer = render::renderer::Renderer::create_for_builder(
            pipeline_manager,

            //Vulkano data
            window,
            device,
            queue,
            swapchain,
            images,

            frame_system,

            false,

            engine_settings.clone(),
            Arc::new(Mutex::new(uniform_manager_tmp)),

            Arc::new(Mutex::new(render::renderer::RendererState::WAITING)),
        );
        println!("Finished renderer!", );
        Ok((renderer, previous_frame))
    }
}


///A function to rank a itterator of physical devices. The best one will be returned
fn rank_devices(devices: vulkano::instance::PhysicalDevicesIter)
-> Option<vulkano::instance::PhysicalDevice>
{
    use vulkano::instance::PhysicalDeviceType;
    use std::collections::BTreeMap;
    //save the devices according to the score, at the end pick the last one (highest score);
    let mut ranking = BTreeMap::new();

    for device in devices.into_iter(){
        let mut device_score = 0;

        match device.ty(){
            PhysicalDeviceType::IntegratedGpu => device_score += 10,
            PhysicalDeviceType::DiscreteGpu => device_score += 50,
            PhysicalDeviceType::VirtualGpu => device_score += 20,
            PhysicalDeviceType::Cpu => device_score += 5,
            PhysicalDeviceType::Other => device_score += 0,
        }

        ranking.insert(device_score, device);
    }

    let mut tmp_vec = Vec::new();
    for (_, device) in ranking.into_iter().rev(){
        tmp_vec.push(device);
    }

    if tmp_vec.len()>0{
        Some(tmp_vec[0])
    }else{
        None
    }
}


///A collection of renderpasses. The right pass is choosen based on the render settings which are present
/// while building the renderer
pub enum RenderPassLayout {
    ///Msaa: no, HDR/Bloom: no, Postprogress: no
    NoMsaaNoHdrNoPostProgress,
    ///Msaa: yes, HDR/Bloom: no, Postprogress: no
    MsaaNoHdrNoPostProgress,
    ///Msaa: yes, HDR/Bloom: yes, Postprogress: no
    MsaaHdrNoPostProgress,
    ///Msaa: yes, HDR/Bloom: no, Postprogress: yes
    MsaaNoHdrPostProgress,
    ///Msaa: no, HDR/Bloom: yes, Postprogress: no
    NoMsaaHdrNoPostProgress,
    ///Msaa: no, HDR/Bloom: no, Postprogress: yes
    NoMsaaNoHdrPostProgress,
    ///Msaa: no, HDR/Bloom: yes, Postprogress: yes
    NoMsaaHdrPostProgress,
    ///Msaa: yes, HDR/Bloom: yes, Postprogress: yes
    MsaaHdrPostProgress,
}

impl RenderPassLayout{

    ///Returns a RenderPass, based on the render settings to give to it.
    pub fn get_render_pass(settings: &render_settings::RenderSettings) -> Self{
        //get the msaa settings
        if settings.get_msaa_factor() > 1{
            //save the msaa factor
            let msaa_factor = settings.get_msaa_factor();
            //get the hdr setting
            if settings.has_hdr(){
                //has postprogress?
                if settings.has_post_progress(){
                    RenderPassLayout::MsaaHdrPostProgress
                }else{
                    RenderPassLayout::MsaaHdrNoPostProgress
                }
            }else{
                //has postprogress?
                if settings.has_post_progress(){
                    RenderPassLayout::MsaaNoHdrPostProgress
                }else{
                    RenderPassLayout::MsaaNoHdrNoPostProgress
                }
            }
        }else{
            //get the hdr setting
            if settings.has_hdr(){
                //has postprogress?
                if settings.has_post_progress(){
                    RenderPassLayout::NoMsaaHdrPostProgress
                }else{
                    RenderPassLayout::NoMsaaHdrNoPostProgress
                }
            }else{
                //has postprogress?
                if settings.has_post_progress(){
                    RenderPassLayout::NoMsaaNoHdrPostProgress
                }else{
                    RenderPassLayout::NoMsaaNoHdrNoPostProgress
                }
            }
        }
    }

    pub fn has_msaa(&self) -> bool{
        match self{
            &RenderPassLayout::MsaaNoHdrNoPostProgress => true,
            &RenderPassLayout::MsaaHdrNoPostProgress => true,
            &RenderPassLayout::MsaaNoHdrPostProgress => true,
            &RenderPassLayout::MsaaHdrPostProgress => true,
            _ => false,
        }
    }

    pub fn has_hdr(&self) -> bool{
        match self{
            &RenderPassLayout::MsaaHdrNoPostProgress => true,
            &RenderPassLayout::NoMsaaHdrNoPostProgress => true,
            &RenderPassLayout::NoMsaaHdrPostProgress => true,
            &RenderPassLayout::MsaaHdrPostProgress => true,
            _ => false,
        }
    }

    pub fn has_post_progress(&self) -> bool{
        match self{
            &RenderPassLayout::MsaaNoHdrPostProgress => true,
            &RenderPassLayout::NoMsaaNoHdrPostProgress => true,
            &RenderPassLayout::NoMsaaHdrPostProgress => true,
            &RenderPassLayout::MsaaHdrNoPostProgress => true,
            _ => false,
        }
    }
}

/*
///returns a simple render pass
pub fn get_simple_rendepass() -> Arc<RenderPassAbstract + Send + Sync>{
    let renderpass = Arc::new(
        ordered_passes_renderpass!(queue.device().clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1, //TODO msaa samples based on settings
            },
            depth: {
                load: Clear,
                store: DontCare,
                format: vulkano::format::Format::D16Unorm,
                samples: 1,
            }
        },
        passes:[
            {
                color: [color],
                depth_stencil: {depth},
                input: []   //has no input, might get the light vec as input and the pre rendered light depth infos
            }
        ]
    ).expect("failed to create render_pass")
    );

    renderpass
}

*/
