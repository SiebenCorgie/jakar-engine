use vulkano;
use vulkano_win;
use vulkano::instance::debug::{DebugCallback, MessageTypes};
use vulkano::instance::Instance;

use std::sync::{Arc, Mutex};


use render;
use render::renderer::Renderer;
use render::renderer::BuildRender;
use render::pipeline_manager;
use render::uniform_manager;
use render::frame_system;
use render::pipeline_builder;
use render::post_progress;
use render::light_system;
use render::render_passes::RenderPassConf;
use render::window::Window;
use render::shadow_system::ShadowSystem;

use core::engine_settings;
use tools::engine_state_machine::RenderState;
///Describes how the handler should load the layers, by default set to NoLayer
#[derive(PartialEq, Clone)]
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
    ///The engine settings used to build this renderer.
    pub settings: Arc<Mutex<engine_settings::EngineSettings>>,
    ///Describes the extensions needed to work properly. By **default** its only the extensions needed
    /// To create the window surface. The craetion will fail if those aren't met.
    pub instance_extensions_needed: vulkano::instance::InstanceExtensions,
    ///Describes the extensions needed from the abstract device. **Default: only the swapchain is needed***
    pub device_extensions_needed: vulkano::device::DeviceExtensions,

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
    ///Becomes Some(instance) when calling `start_build`. This intermediate step is neede to make
    ///it possible to start the input thread.
    pub instance: Option<Arc<Instance>>,

}

impl BuildRender for RenderBuilder{
    ///Builds a renderer for a specified window
    fn build(
        mut self,
        mut window: Window,
    ) -> Result<Renderer, String>{
        //now decide for a mesaging service from vulkan, when in release mode, we wont do any
        //if not we read from the builder and construct a callback

        let instance = {
            match self.instance{
                Some(ref inst) => inst.clone(),
                None => return Err(String::from("Tried to build without instance!")),
            }
        };

        {
            match (*(self.settings
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
                        println!("STATUS: RENDER: {} {}: {}", msg.layer_prefix, ty, msg.description);
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
                            println!("STATUS: RENDER: {} {}: {}", msg.layer_prefix, ty, msg.description);
                        }
                    ).ok();
                }
            }
        }

        //Using a physical device according to the builder settings
        //TODO use the name
        let physical_device_tmp = {
            match self.preferred_physical_device{
                Some(_) =>{
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

        println!("Selected best graphics card", );

        let should_enable_all = {
            if self.layer_loading == LayerLoading::All{
                true
            }else{
                false
            }
        };

        if should_enable_all{
            let extensions = vulkano::device::DeviceExtensions::supported_by_device(
                physical_device.clone()
            );
            println!("Enabling all extensions to make debug layers work: ", );
            println!("\t {:?}", extensions);
            self.device_extensions_needed = self.device_extensions_needed.intersection(&extensions);
            println!("Enabled all logical layers", );
        }

        println!("QUEUEINFO:\n==========", );
        //Create a queue
        for queue in physical_device.queue_families(){

            print!("Queue {}, graph: {}, comp: {}, count: {}", queue.id(), queue.supports_graphics(), queue.supports_compute(), queue.queues_count());
        }
        println!("==========", );

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
        let device_ext = self.device_extensions_needed;

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
            let mut engine_settings_lck = self.settings
            .lock()
            .expect("Failed to lock settings");


            //Set dimensions or fallback to the ones in the settings
            let dimensions = caps.current_extent.unwrap_or((*engine_settings_lck).get_dimensions());
            let usage = caps.supported_usage_flags;
            let format = caps.supported_formats[0].0;

            //Check if we can get more then 60fps
            let present_mode = {
                if engine_settings_lck.get_render_settings().get_vsync(){
                    //We should vsync, and it is always supported... returning
                    vulkano::swapchain::PresentMode::Fifo
                }else{
                    //Test if it is supported, if not turn it off in the settings
                    if caps.present_modes.immediate{
                        //suppoorted and enabled
                        vulkano::swapchain::PresentMode::Immediate
                    }else{
                        //Turn it of and set to fifo
                        use ::rt_error;
                        rt_error("RenderBuilder", "Immediate mode is not supported, using v_sync");
                        engine_settings_lck.get_render_settings_mut().set_vsync(false);
                        vulkano::swapchain::PresentMode::Fifo
                    }
                }
            };


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
                present_mode,
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

        let uniform_manager = Arc::new(Mutex::new(uniform_manager_tmp));

        println!("Starting frame passes", );
        let passes = render::render_passes::RenderPasses::new(
            device.clone(),
            swapchain.format(),
            self.settings.clone(),
        );

        println!("Starting frame system", );
        //now create us a default frame system
        let frame_system = frame_system::FrameSystem::new(
            self.settings.clone(),
            device.clone(),
            passes.clone(),
            queue.clone(),

        );
        println!("Finished the frame system", );
        //Creates the renderers pipeline manager , will be packed into the arc mutex later
        let pipeline_manager_arc = Arc::new(
            Mutex::new(
                pipeline_manager::PipelineManager::new(
                    device.clone(),
                    passes.clone(),
                )
            )
        );

        //After creating the pipeline manager, we can create the post progressing system with
        // a currently static set of shader
        //TODO make shader dynamic
        println!("Getting post progress pipeline", );
        let post_progress_pipeline = pipeline_manager_arc.lock()
        .expect("failed to lock new pipeline manager")
        .get_pipeline_by_config(
            pipeline_builder::PipelineConfig::default()
                .with_subpass_id(super::SubPassType::PostProgress.get_id())
                .with_shader("PpExposure".to_string())
                .with_render_pass(RenderPassConf::AssemblePass)
                .with_depth_and_stencil_settings(
                    pipeline_builder::DepthStencilConfig::NoDepthNoStencil
                ),
        );

        let resolve_pipeline = pipeline_manager_arc.lock()
        .expect("failed to lock new pipeline manager")
        .get_pipeline_by_config(
            pipeline_builder::PipelineConfig::default()
                .with_subpass_id(super::SubPassType::HdrSorting.get_id())
                .with_shader("PpResolveHdr".to_string())
                .with_render_pass(RenderPassConf::ObjectPass)
                .with_depth_and_stencil_settings(
                    pipeline_builder::DepthStencilConfig::NoDepthNoStencil
                ),
        );

        let blur_pipeline = pipeline_manager_arc.lock()
        .expect("failed to lock new pipeline manager")
        .get_pipeline_by_config(
            pipeline_builder::PipelineConfig::default()
                .with_subpass_id(super::SubPassType::Blur.get_id())
                .with_shader("PpBlur".to_string())
                .with_render_pass(RenderPassConf::BlurPass)
                .with_depth_and_stencil_settings(
                    pipeline_builder::DepthStencilConfig::NoDepthNoStencil
                ),
        );

        println!("Starting post progress framework", );
        let post_progress = post_progress::PostProgress::new(
            self.settings.clone(),
            post_progress_pipeline,
            resolve_pipeline,
            blur_pipeline,
            device.clone(),
            queue.clone(),
            //&passes,
        );

        println!("Creating light culling system", );
        let light_system = light_system::LightSystem::new(
            uniform_manager.clone(),
            device.clone(),
            queue.clone()
        );

        let shadow_system = ShadowSystem::new(
            device.clone(), self.settings.clone(), pipeline_manager_arc.clone()
        );

        println!("Finished Render Setup", );
        //Pass everthing to the struct

        let renderer = render::renderer::Renderer::create_for_builder(
            pipeline_manager_arc,
            window,
            device,
            queue,
            swapchain,
            images,

            frame_system,
            passes,
            shadow_system,
            light_system,
            post_progress,

            false,
            self.settings,
            uniform_manager,
            Arc::new(Mutex::new(RenderState::Idle)),
        );
        Ok(renderer)
    }
}

impl RenderBuilder {
    ///Creates a new default renderer. For the default values, see the struct documentation.
    /// After the creation you are free to change any parameter.
    pub fn new(engine_settings: Arc<Mutex<engine_settings::EngineSettings>>) -> Self{


        //Init the default values
        let instance_extensions_needed = vulkano_win::required_extensions();
        println!("Starting render builder", );
        let device_extensions_needed = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        let layers = LayerLoading::NoLayer;
        let vulkan_messages = vulkano::instance::debug::MessageTypes::errors();
        //Setup the features needed for the engine to run properly
        let minimal_features = vulkano::instance::Features {
            sampler_anisotropy: true,
            sample_rate_shading: true,
            logic_op: true, //needed for custom blending
            depth_clamp: true, //needed for correct shadow mapping
            .. vulkano::instance::Features::none()
        };
        RenderBuilder{
            settings: engine_settings,
            instance_extensions_needed: instance_extensions_needed,
            device_extensions_needed: device_extensions_needed,
            layer_loading: layers,
            vulkan_messages: vulkan_messages,
            preferred_physical_device: None,
            minimal_features: minimal_features,
            instance: None,
        }
    }

    ///Creates an instance from the current settings. Returns an error string if something went wrong,
    ///else stores the instance and returns `Ok()`.
    pub fn create_instance(&mut self) -> Result<(), String>{
        //=========================================================
        //Now we start to create an instance of vulkan which is needed to build a window
        //Init Vulkan
        //Check for needed extensions
        //let mut extensions = vulkano_win::required_extensions();
        //Add the debug extension

        //Generate the list of debuging layers used
        let debuging_layers_string = {
            //Decide based on the engine settings if there should be list. If yes, decide the list
            // based on the builder setting.
            if (*(self.settings.lock().expect("failed to lock settings in render builder")))
            .build_mode != engine_settings::BuildType::Release
            {
                match self.layer_loading.clone(){
                    LayerLoading::All => {

                        let list = vulkano::instance::layers_list().expect("failed to get layer list");
                        let mut ret_list: Vec<String> = Vec::new();
                        println!("LoadingAllDebugLayers", );
                        for item in list.into_iter(){
                            if item.name().to_string() == "VK_LAYER_RENDERDOC_Capture".to_string(){
                                println!("\t{}", item.name().to_string());
                                ret_list.push(item.name().to_string());
                            }

                            if item.name().to_string() == "VK_LAYER_LUNARG_standard_validation".to_string(){
                                println!("\t{}", item.name().to_string());
                                ret_list.push(item.name().to_string());
                            }

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
            let engine_settings_lck = self.settings.lock().expect("failed to lock settings");

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
        //Since we need some more logical extension if we want to use all debug layer, we query
        //all possible layers from the physical device and register them for the run
        let should_enable_all = {
            if self.layer_loading == LayerLoading::All{
                true
            }else{
                false
            }
        };

        if should_enable_all{
            self.instance_extensions_needed = self.instance_extensions_needed.intersection(
                &vulkano::instance::InstanceExtensions{
                    ext_debug_report: true,
                    khr_surface: true,
                    khr_xcb_surface: true,
                    ..vulkano::instance::InstanceExtensions::none()
                }
            );
            println!("InstanceExtensions: ", );
            println!("\t{:?}", self.instance_extensions_needed);
            println!("Loaded all core extensions", );
        }

        //Create a vulkan instance from these extensions
        let try_instance = vulkano::instance::Instance::new(
            Some(&app_info),
            &self.instance_extensions_needed, //TODO verify
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

        self.instance = Some(instance);

        println!("Created Instance", );
        Ok({})
    }

    ///Returns an instance if there is already one, or takes the current information of the builder
    /// to create one and returns this instead.
    /// #Panic If this doesn't work it will panic.
    pub fn get_instance(&mut self) -> Arc<Instance>{
        match self.instance{
            Some(ref inst) => inst.clone(),
            None => {
                match self.create_instance(){
                    Ok(_) => {},
                    Err(_) => panic!("Failed to create an instance"),
                }
                //now return the instance which should be there now.
                self.instance.clone().expect("there was no instance, but there should be one!")
            }
        }
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
