use render::pipeline_manager;
use render::uniform_manager;
use core::resource_management::asset_manager;
use render::window;
use core::engine_settings;
use input::KeyMap;

use rt_error;

use vulkano;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::swapchain::AcquireError;
use vulkano::sync::GpuFuture;
use vulkano_win;
use vulkano::instance::debug::{DebugCallback, MessageTypes};
use vulkano::pipeline::GraphicsPipelineAbstract;

use winit;

use std::sync::{Arc,Mutex};
use std::time::{Duration,Instant};
use std::mem;
use time;

///The main renderer
pub struct Renderer  {
    ///Holds the renderers pipeline_manager
    pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,

    //Vulkano data
    extensions: vulkano::instance::InstanceExtensions,
    instance: Arc<vulkano::instance::Instance>,
    debug_callback: Option<DebugCallback>,
    //window: vulkano_win::Window,
    window: window::Window,
    device: Arc<vulkano::device::Device>,
    queues: vulkano::device::QueuesIter,
    queue: Arc<vulkano::device::Queue>,
    swapchain: Arc<vulkano::swapchain::Swapchain>,
    images: Vec<Arc<vulkano::image::SwapchainImage>>,
    renderpass: Arc<RenderPassAbstract + Send + Sync>,
    depth_buffer: Arc<vulkano::image::AttachmentImage<vulkano::format::D16Unorm>>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,

    previous_frame: Option<Box<GpuFuture>>,

    //Is true if we need to recreate the swap chain
    recreate_swapchain: bool,

    engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
    //A reference to the keymap to create input dependent functions
    key_map: Arc<Mutex<KeyMap>>,

}

impl Renderer {
    ///Creates a new renderer with all subsystems
    pub fn new(
            events_loop: Arc<Mutex<winit::EventsLoop>>,
            engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
            key_map: Arc<Mutex<KeyMap>>,
        ) -> Self{
        //Init Vulkan

        //Check for needed extensions
        let mut extensions = vulkano_win::required_extensions();
        //Add the debug extension
        extensions.ext_debug_report = true;

        //Add debuging layer
        //println!("STATUS: RENDER CORE: List of Vulkan debugging layers available to use: ", );
        let mut layers = vulkano::instance::layers_list().expect("failed to get layer list");
        while let Some(l) = layers.next() {
            //println!("STATUS: RENDER: \t{}", l.name());
        }

        // NOTE: To simplify the example code we won't verify these layer(s) are actually in the layers list:
        let layer = "VK_LAYER_LUNARG_standard_validation";
        let layers = vec![&layer];

        //Create a vulkan instance from these extensions
        let instance = vulkano::instance::Instance::new(None, &extensions, layers)
        .expect("failed to create instance");

        let engine_settings_wrk = {
            let engine_settings_lck = engine_settings
            .lock()
            .expect("failed to lock engine settings");

            (*engine_settings_lck).clone()
        };

        //Register debuging messages
        let mut all = MessageTypes {
            error: true,
            warning: true,
            performance_warning: true,
            information: true,
            debug: true,
        };

        //if vulkan is set silent, show no messages
        if engine_settings_wrk.vulkan_silence(){
            all = MessageTypes {
                error: false,
                warning: false,
                performance_warning: false,
                information: false,
                debug: false,
            };
        }

        let _debug_callback = DebugCallback::new(&instance, all, |msg| {
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

        //Get us a graphics card
        let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
                                .next().expect("no device available");
        //println!("STATUS: RENDER: Using device: {} (type: {:?})", physical.name(), physical.ty());
        //copy the events loop for the window creation
        let events_loop_unlck = events_loop
        .lock()
        .expect("Failed to hold lock on events loop");

        //and create a window for it
        let mut window = window::Window::new(
            &instance.clone(), &*events_loop_unlck, engine_settings.clone()
        );

        //Create a queue
        let queue = physical.queue_families().find(
            |&q| q.supports_graphics() &&
            window.surface().is_supported(q).unwrap_or(false)
        )
        .expect("couldn't find a graphical queue family");

        //select needed device extensions
        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };

        //TODO Test for availabe features via a difference check maybe
        //first create a
        let minimal_features = vulkano::instance::Features {
            geometry_shader: true,
            .. vulkano::instance::Features::none()
        };
        //then look if physical.supported_features() doesnt crash the creation, if so, change engine
        //settings to a fallback state where we can create the device
        //ref: https://docs.rs/vulkano/0.5.6/vulkano/instance/struct.Features.html#structfield.sampler_anisotropy

        //Create a artificial device and its queue
        let (device, mut queues) = vulkano::device::Device::new(
            physical, physical.supported_features(),
            &device_ext, [(queue, 0.5)].iter().cloned()
        )
        .expect("failed to create device");

        let queue = queues.next().expect("failed to create queue!");

        //Get the swapchain and its images
        let (swapchain, images) = {

            let caps = window.surface()
            .capabilities(physical).expect("failed to get surface capabilities");

            //lock settings to read fallback settings
            let mut engine_settings_lck = engine_settings
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
                format,
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

        //Create a depth buffer
        let depth_buffer = vulkano::image::attachment::AttachmentImage::transient(
            device.clone(), images[0].dimensions(), vulkano::format::D16Unorm)
            .expect("failed to create depth buffer!");



        let mut uniform_manager_tmp = uniform_manager::UniformManager::new(
            device.clone()
        );


        //TODO, create custom renderpass with different stages (light computing, final shading (how to loop?),
        //postprogress) => Dig through docs.
        //Create a simple renderpass
        println!("Createing Render Pass", );
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
        println!("Finished renderpass", );

        //Create the frame buffers from all images
        let framebuffers = images.iter().map(|image| {
            Arc::new(vulkano::framebuffer::Framebuffer::start(renderpass.clone())
                //The color pass
                .add(image.clone()).expect("failed to add image to frame buffer!")
                //and its depth pass
                .add(depth_buffer.clone()).expect("failed to add depth to frame buffer!")
                .build().expect("failed to build framebuffer!"))
        }).collect::<Vec<_>>();

        let mut store_framebuffer: Vec<Arc<FramebufferAbstract + Send + Sync>> = Vec::new();
        for i in framebuffers{
            store_framebuffer.push(i.clone());
        }

        let previous_frame = Some(Box::new(vulkano::sync::now(device.clone())) as Box<GpuFuture>);

        //Creates the renderers pipeline manager
        let pipeline_manager = Arc::new(
            Mutex::new(
                pipeline_manager::PipelineManager::new(
                    device.clone(), renderpass.clone(),
                )
            )
        );
        println!("Finished Render Setup", );
        //Pas everthing to the struct
        Renderer{
            pipeline_manager: pipeline_manager,

            //Vulkano data
            extensions: extensions,
            instance: instance.clone(),
            debug_callback: _debug_callback,
            window: window,
            device: device,
            queues: queues,
            queue: queue,
            swapchain: swapchain,
            images: images,
            renderpass: renderpass,
            depth_buffer: depth_buffer,
            framebuffers: store_framebuffer,

            previous_frame: previous_frame,

            recreate_swapchain: false,

            engine_settings: engine_settings.clone(),
            uniform_manager: Arc::new(Mutex::new(uniform_manager_tmp)),

            key_map: key_map,
        }
    }



    ///Recreates swapchain for the window size in `engine_settings`
    ///Returns true if successfully recreated chain
    pub fn recreate_swapchain(&mut self) -> bool{
        //get new dimmensions etc
        let mut engine_settings_lck = self.engine_settings
        .lock()
        .expect("Faield to lock settings");

        let (new_width, new_height) = self.window
        .window()
        .get_inner_size_pixels()
        .expect("failed to get hight and width of current window");

        (*engine_settings_lck).set_dimensions(new_width, new_height);

        let (new_swapchain, new_images) =
        match self.swapchain.recreate_with_dimension(engine_settings_lck.get_dimensions()) {
            Ok(r) => r,
            // This error tends to happen when the user is manually resizing the window.
            // Simply restarting the loop is the easiest way to fix this issue.
            Err(SwapchainCreationError::UnsupportedDimensions) => {
                return false;
            },
            Err(err) => panic!("{:?}", err)
        };

        //Now repace
        mem::replace(&mut self.swapchain, new_swapchain);
        mem::replace(&mut self.images, new_images);

        //Recreate depth buffer for new size
        //Create a depth buffer
        self.depth_buffer = vulkano::image::attachment::AttachmentImage::transient(
            self.device.clone(), self.images[0].dimensions(), vulkano::format::D16Unorm)
            .expect("failed to create depth buffer!");


        // Because framebuffers contains an Arc on the old swapchain, we need to
        // recreate framebuffers as well.
        //Create the frame buffers from all images
        let framebuffers = self.images.iter().map(|image| {
            Arc::new(vulkano::framebuffer::Framebuffer::start(self.renderpass.clone())
                //The color pass
                .add(image.clone()).expect("failed to add image to frame buffer!")
                //and its depth pass
                .add(self.depth_buffer.clone()).expect("failed to add depth to frame buffer!")
                .build().expect("failed to build framebuffer!"))
        }).collect::<Vec<_>>();

        let mut store_framebuffer: Vec<Arc<FramebufferAbstract + Send + Sync>> = Vec::new();
        for i in framebuffers{
            store_framebuffer.push(i.clone());
        }


        mem::replace(&mut self.framebuffers, store_framebuffer);

        //Now when can mark the swapchain as "fine" again
        self.recreate_swapchain = false;
        true
    }

    ///Returns the image if the image state is outdated
    ///Panics if another error occures while pulling a new image
    pub fn check_image_state(&self) -> Result<(usize, SwapchainAcquireFuture), AcquireError>{

        match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(r) => return Ok(r),
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                return Err(vulkano::swapchain::AcquireError::OutOfDate);
            },
            Err(err) => panic!("{:?}", err)
        };
    }

    ///Renders the scene with the parameters supplied by the asset_manager
    pub fn render(&mut self, asset_manager: &mut asset_manager::AssetManager){

        //println!("STATUS: RENDER CORE: Starting render ", );
        //DEBUG
        let start_time = Instant::now();

        //Clean the last frame for starting a new one
        self.previous_frame.as_mut().expect("failed to clean previous frame").cleanup_finished();


        //If found out in last frame that images are out of sync, generate new ones
        if self.recreate_swapchain{
            if !self.recreate_swapchain(){
                //If we got the UnsupportedDimensions Error (and therefor returned false)
                //Abord the frame
                return;
            }
        }

        //Try to get a new image
        //If not possible becuase outdated (result is Err)
        //then return (abort frame)
        //and recreate swapchain
        let (image_num, acquire_future) =
        match self.check_image_state(){
            Ok(r) => r,
            Err(_) => {
                self.recreate_swapchain = true;
                return;
            },
        };


        //TODO have to find a nicer way of doing this... later
        let command_buffer = {


            let dimensions = {
                let engine_settings_lck = self.engine_settings
                .lock()
                .expect("Faield to lock settings");
                (*engine_settings_lck).get_dimensions()
            };

            let mut tmp_cmd_buffer = Some(
                vulkano::command_buffer::AutoCommandBufferBuilder::new(
                    self.device.clone(),
                    self.queue.family()).expect("failed to create tmp buffer!")
                );

            let build_start = tmp_cmd_buffer
            .take()
            .expect("failed to take cmd buffer build for start");

            tmp_cmd_buffer = Some(build_start.begin_render_pass(
                self.framebuffers[image_num].clone(), false,
                vec![
                    [0.01, 0.0, 0.1, 1.0].into(),
                    1f32.into()
                ]).expect("failed to clear"));

            //println!("Trying to get meshes in frustum", );
            //Draw
                //get all meshes, later in view frustum based on camera
            let meshes_in_frustum = asset_manager.get_meshes_in_frustum();
            println!("Rendering {} meshes", meshes_in_frustum.len());

            for mesh_transform in meshes_in_frustum.iter(){

                //let mesh = mesh_transform.0.clone();
                //let transform = mesh_transform.1.clone();

                let mesh_lck = mesh_transform.0
                .lock()
                .expect("could not lock mesh for building command buffer");

                let cb = tmp_cmd_buffer
                .take()
                .expect("Failed to recive command buffer in loop!");

                let material = asset_manager
                .get_material_manager()
                .get_material(&(*mesh_lck).get_material_name());

                let mut unlocked_material = material
                .lock()
                .expect("Failed to lock material");

                //We have to create all the types in advance to prevent a lock
                let pipeline_copy = {
                    //Returning pipeline
                    (*unlocked_material).get_pipeline()
                };

                let set_01 = {
                    //TODO Set the model-matrix from the mesh data
                    //aquirre the tranform matrix and generate the new set_01
                    (*unlocked_material).get_set_01(mesh_transform.1)
                };


                let set_02 = {
                    (*unlocked_material).get_set_02()
                };

                let set_03 = {
                    (*unlocked_material).get_set_03()
                };

                let set_04 = {
                    (*unlocked_material).get_set_04()
                };
                //println!("STATUS: RENDER CORE: Adding to tmp cmd buffer", );

                tmp_cmd_buffer = Some(cb
                    .draw_indexed(
                        pipeline_copy,

                        vulkano::command_buffer::DynamicState{
                            line_width: None,
                            viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                                origin: [0.0, 0.0],
                                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                depth_range: 0.0 .. 1.0,
                            }]),
                            scissors: None,
                        },
                        (*mesh_lck)
                        .get_vertex_buffer(),

                        (*mesh_lck)
                        .get_index_buffer(
                            self.device.clone(), self.queue.clone()
                        ).clone(),

                        (set_01, set_02, set_03, set_04),

                        ()
                    ).expect("Failed to draw in command buffer!")
                );
            }
            //End renderpass
            tmp_cmd_buffer
            .take()
            .expect("failed to return command buffer to main buffer")
        }

        .end_render_pass().expect("failed to end")
        .build().expect("failed to end");;

        //println!("STATUS: RENDER CORE: Trying flush", );

        //TODO find a better methode then Option<Box<GpuFuture>>
        let future = self.previous_frame
        .take()
        .expect("failed to take previous frame")
        .join(acquire_future)
        .then_execute(
            self.queue.clone(), command_buffer
        )
        .expect("failed to execute buffer!")
        .then_swapchain_present(
            self.queue.clone(), self.swapchain.clone(), image_num
        )
        .then_signal_fence_and_flush().expect("failed to flush");

        self.previous_frame = Some(Box::new(future) as Box<_>);

        //DEBUG
        let fps_time = start_time.elapsed().subsec_nanos();
        //println!("STATUS: RENDER: FPS: {}", 1.0/ (fps_time as f32 / 1_000_000_000.0) );
    }

    ///Returns the uniform manager
    pub fn get_uniform_manager(&self) -> Arc<Mutex<uniform_manager::UniformManager>>{
        self.uniform_manager.clone()
    }

    ///Returns the pipeline manager of this renderer
    pub fn get_pipeline_manager(&mut self) -> Arc<Mutex<pipeline_manager::PipelineManager>>{
        self.pipeline_manager.clone()
    }

    ///Starts the rendering loop UNIMPLEMENTED
    pub fn start_loop(){

    }

    ///Returns the device of this renderer
    pub fn get_device(&self) -> Arc<vulkano::device::Device>{
        self.device.clone()
    }

    ///Returns the queue of this renderer
    pub fn get_queue(&self) -> Arc<vulkano::device::Queue>{
        self.queue.clone()
    }

    ///A helper function whicht will creat a tubel of
    ///(`pipeline_manager`, `uniform_manager`, `device`)
    ///This is needed for the material creation
    pub fn get_material_instances(&self) -> (
        Arc<GraphicsPipelineAbstract + Send + Sync>,
        Arc<Mutex<uniform_manager::UniformManager>>,
        Arc<vulkano::device::Device>,
        )
    {
        //Copy a default pipeline currently there is no way to nicly create a pipeline from a
        //shader file without doubling the pipeline code :/
        let pipeline_copy = {
            let pipe_man_inst = self.pipeline_manager.clone();
            let mut pipe_man_lck = pipe_man_inst.lock().expect("failed to hold pipe man lock");
            (*pipe_man_lck).get_default_pipeline()
        };

        let pipe = pipeline_copy;
        let uni_man = self.uniform_manager.clone();
        let device = self.device.clone();

        (pipe, uni_man, device)
    }

    ///Returns an instance of the engine settings
    ///This might be a dublicate, still helpful
    pub fn get_engine_settings(&mut self) -> Arc<Mutex<engine_settings::EngineSettings>>{
        self.engine_settings.clone()
    }

}

/*TODO:
The Functions
Start the renderer
The Renderer is fixed fo now, it will always draw the same frame but will update its content everytime
this will be done via a Arc<content> / clone methode.
For instance the uniform_set 01 will be supplied by the camera system for model and camera info
the set_02 will be supplied by the material system in cooperation with the pipeline system to bind
the correct pipeline and uniform set at the right mesh
the vertex buffer will be copied from each mesh which will be rendered. The scene system will have its own
loop.
Last but not least, at some point the the renderer will calculate the forward+ light pass and give the
info to a ligh handeling system. But this is not implemented yet and won't be so fast. I have
to find out how to calculate this forward pass (ref: https://www.slideshare.net/takahiroharada/forward-34779335
and https://takahiroharada.files.wordpress.com/2015/04/forward_plus.pdf and
https://www.3dgep.com/forward-plus/#Forward)
*/
