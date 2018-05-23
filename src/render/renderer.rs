use render::pipeline_manager;
use render::uniform_manager;
use core::resource_management::asset_manager;
use core::resources::camera::Camera;
use render::window;
use render::window::Window;
use render::render_helper;
use render::frame_system;
use render::post_progress;
use render::light_system;
use render::render_passes::RenderPasses;
use render::shadow_system;
use render::forward_system::ForwardSystem;

use core::next_tree::{SceneTree, ValueTypeBool, SceneComparer};
use core::engine_settings;
//use core::simple_scene_system::node_helper;
use core::next_tree::content::ContentType;
use tools::engine_state_machine::RenderState;


use winit;
use vulkano;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::swapchain::AcquireError;
use vulkano::sync::GpuFuture;
use vulkano::sync::FenceSignalFuture;
use vulkano::swapchain::PresentFuture;
use vulkano::command_buffer::CommandBufferExecFuture;
use vulkano::command_buffer::AutoCommandBuffer;

use std::sync::{Arc,Mutex};
use std::time::{Instant, Duration};
use std::mem;


///manages some some debug information
struct RenderDebug {
    last_sec_start: Instant,
    current_counter: u32,
    avg_mesh_render_time: Duration,
    first_mesh_time: bool,
}

impl RenderDebug{
    pub fn new() -> Self{
        RenderDebug{
            last_sec_start: Instant::now(),
            current_counter: 0,
            avg_mesh_render_time: Duration::from_secs(0),
            first_mesh_time: true,
        }
    }
    pub fn update(&mut self){
        if self.last_sec_start.elapsed().as_secs() > 0{
            println!("FPS: {}", self.current_counter);
            self.last_sec_start = Instant::now();
            self.current_counter = 1;
        }else{
            self.current_counter += 1;
        }
    }

    pub fn update_avr_mesh(&mut self, dur: Duration){
        if self.first_mesh_time{
            self.avg_mesh_render_time = dur;
            self.first_mesh_time = false;
            return;
        }

        self.avg_mesh_render_time += dur;
        self.avg_mesh_render_time = self.avg_mesh_render_time.checked_div(2).expect("Failed to calc time!");
    }

    pub fn print_stat(&self){
        println!("Average mesh draw time: {:?}", self.avg_mesh_render_time);
    }
}


///Used tp build a render instance
pub trait BuildRender {
    ///Build a renderer based on settings and a window which will recive the iamges
    fn build(
        self,
        window: Window,
    ) -> Result<Renderer, String>;
}

///The main renderer. Should be created through a RenderBuilder
pub struct Renderer {
    ///Holds the renderers pipeline_manager
    pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,


    //window: vulkano_win::Window,
    window: window::Window,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    swapchain: Arc<vulkano::swapchain::Swapchain<winit::Window>>,
    images: Vec<Arc<vulkano::image::SwapchainImage<winit::Window>>>,

    frame_system: frame_system::FrameSystem,
    shadow_system: shadow_system::ShadowSystem,
    forward_system: ForwardSystem,
    light_system: light_system::LightSystem,

    render_passes: Arc<Mutex<RenderPasses>>,

    ///The post progresser
    post_progress: post_progress::PostProgress,

    //Is true if we need to recreate the swap chain
    recreate_swapchain: bool,

    engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,

    state: Arc<Mutex<RenderState>>,
    last_frame_end: Option<Arc<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<Box<vulkano::sync::GpuFuture + Send + Sync>, AutoCommandBuffer>, winit::Window>>>>,

    debug_info: RenderDebug,
}

impl Renderer {
    ///Creates a new renderer from all the systems. However, you should only use the builder to create
    /// a renderer.
    pub fn create_for_builder(
        pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,
        window: window::Window,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
        swapchain: Arc<vulkano::swapchain::Swapchain<winit::Window>>,
        images: Vec<Arc<vulkano::image::SwapchainImage<winit::Window>>>,

        //the used frame system
        frame_system: frame_system::FrameSystem,

        render_passes: Arc<Mutex<RenderPasses>>,
        shadow_system: shadow_system::ShadowSystem,
        forward_system: ForwardSystem,
        light_system: light_system::LightSystem,
        post_progress: post_progress::PostProgress,

        recreate_swapchain: bool,
        engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
        uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
        state: Arc<Mutex<RenderState>>,
    ) -> Renderer{
        Renderer{
            pipeline_manager: pipeline_manager,
            window: window,
            device: device,
            queue: queue,
            swapchain: swapchain,
            images: images,
            //Helper systems, the frame system handles... well a frame, the post progress writes the
            //static post_progress pass.AcquireError
            frame_system: frame_system,
            shadow_system: shadow_system,
            forward_system: forward_system,
            render_passes: render_passes,
            light_system: light_system,
            post_progress: post_progress,

            recreate_swapchain: recreate_swapchain,
            engine_settings: engine_settings,
            uniform_manager: uniform_manager,
            state: state,
            last_frame_end: None,
            debug_info: RenderDebug::new(),
        }
    }

    ///Recreates swapchain for the window size.
    ///Returns true if successfully recreated chain
    pub fn recreate_swapchain(&mut self) -> bool{
        //get new dimmensions etc

        println!("Getting new window dimensions", );

        //Update the widow dimensions in scope to prevent locking
        let new_dimensions = {
            let mut engine_settings_lck = self.engine_settings
            .lock()
            .expect("Faield to lock settings");

            let c_d = self.window.get_current_extend();
            let (new_width, new_height) = (c_d[0], c_d[1]);
            engine_settings_lck.set_dimensions(new_width, new_height);
            engine_settings_lck.get_dimensions()
        };

        println!("Generating new swpachain and images", );

        let (new_swapchain, new_images) =
        match self.swapchain.recreate_with_dimension(new_dimensions) {
            Ok(r) => r,
            // This error tends to happen when the user is manually resizing the window.
            // Simply restarting the loop is the easiest way to fix this issue.
            Err(SwapchainCreationError::UnsupportedDimensions) => {
                return false;
            },
            Err(err) => panic!("{:?}", err)
        };

        println!("Replacing swapchain and images", );
        //Now repace
        mem::replace(&mut self.swapchain, new_swapchain);
        mem::replace(&mut self.images, new_images);

        println!("Recreating image attachments", );
        //with the new dimensions set in the setting, recreate the images of the frame system as well
        self.frame_system.recreate_attachments();

        //Now when can mark the swapchain as "fine" again
        self.recreate_swapchain = false;
        true
    }

    ///Returns the image if the image state is outdated
    ///Panics if another error occures while pulling a new image
    pub fn check_image_state(&self) -> Result<(usize, SwapchainAcquireFuture<winit::Window>), AcquireError>{

        match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(r) => {
                return Ok(r);
            },
            Err(vulkano::swapchain::AcquireError::OutOfDate) => {
                return Err(vulkano::swapchain::AcquireError::OutOfDate);
            },
            Err(err) => panic!("{:?}", err)
        };
    }

    ///checks the pipeline. If not up to date (return is AcquireError), recreates it.
    fn check_swapchain(&mut self) -> Result<(usize, SwapchainAcquireFuture<winit::Window>), AcquireError>{
        //If found out in last frame that images are out of sync, generate new ones
        if self.recreate_swapchain{
            if !self.recreate_swapchain(){
                //If we got the UnsupportedDimensions Error (and therefor returned false)
                //Abord the frame
                println!("Fucked up while recreating new swapchain", );
                return Err(AcquireError::SurfaceLost);
            }
        }
        //Try to get a new image
        //If not possible becuase outdated (result is Err)
        //then return (abort frame)
        //and recreate swapchain
        match self.check_image_state(){
            Ok(r) => {
                return Ok(r)
            },
            Err(er) => {
                self.recreate_swapchain = true;
                return Err(er);
            },
        };
    }

    ///Renders the scene with the parameters supplied by the asset_manager
    ///and returns the future of this frame. The future can be joint to wait for the gpu
    ///or be supplied to the next update();
    pub fn render(
        &mut self,
        asset_manager: &mut asset_manager::AssetManager,
    ){

        //Show the other system that we are working
        self.set_working_cpu();

        //First of all we get info if we should debug anything, if so this bool will be true
        let (should_capture, mut time_step, start_time, sould_draw_bounds) = {
            let (cap_bool, should_draw_bounds) = {
                let mut lck_set = self.engine_settings.lock().expect("failed to lock settings");
                (lck_set.capture_frame, lck_set.get_render_settings().get_debug_settings().draw_bounds)
            };
            let time_step = Instant::now();
            let start_time = Instant::now();
            (cap_bool, time_step, start_time, should_draw_bounds)
        };
        let (image_number, acquire_future) = {
            match self.check_swapchain(){
                Ok(k) => {
                    k
                },
                Err(e) => {
                    println!("Could not get next swapchain image: {}", e);
                    //early return to restart the frame
                    return;
                }
            }
        };

        //Update the camera data for this frame
        {
            let mut uniform_manager_lck = self.uniform_manager.lock().expect("failed to lock uniform_man.");
            //Finally upadte the MVP data as well
            uniform_manager_lck.update(asset_manager.get_camera().as_uniform_data());
        }

        if should_capture{
            time_step = Instant::now();
        }

        //start the frame
        let mut command_buffer = self.frame_system.new_frame();

        //First of all we compute the light clusters
        let light_buffer_future = self.light_system.update_light_set(
            &mut self.shadow_system, asset_manager
        );

        //we can now join the acquire future and the light_set_future representing the moment
        // were we successfully created the light buffers
        let after_light_future: Box<GpuFuture + Send + Sync> = Box::new(acquire_future.join(light_buffer_future));

        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to update the light set!", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        //now execute the compute shader for generating the lights
        command_buffer = self.light_system.dispatch_compute_shader(
            command_buffer,
        );

        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to dispatch compute shader", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }


        //Its time to render all the shadow maps.
        command_buffer = self.shadow_system.render_shadows(
            command_buffer,
            &self.frame_system,
            asset_manager,
            self.light_system.get_light_store()
        );

        //Now we render all the forward stuff
        command_buffer = self.forward_system.do_forward_shading(
            &self.frame_system,
            &self.light_system,
            &self.post_progress,
            asset_manager,
            command_buffer
        );

        //Since we fininshed the primary work on the asset manager, change to gpu working state
        self.set_working_gpu();

        if should_capture{
            println!("\tRE: Finished adding meshes", );
        }

        //Do all post progressing and finally write the whole frame to the swapchain image
        command_buffer = self.post_progress.do_post_progress(
            command_buffer,
            &self.frame_system,
            self.images[image_number].clone()
        );


        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to do final postprogress!", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }





        //thanks firewater
        let real_cb = command_buffer.build().expect("failed to build command buffer");

        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to finish!", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        //now we submitt the frame, this will add the command buffer to the command queue
        //we then tell the gpu/cpu to present the new image and signal the fence for this frame as well as flush all
        //the operations

        let mut this_frame =
        match self.last_frame_end{
            Some(ref end) => Box::new(after_light_future.join(end.clone())) as Box<GpuFuture + Send + Sync>,
            None => Box::new(after_light_future) as Box<GpuFuture  + Send + Sync>
        }
        .then_execute(self.queue.clone(), real_cb)
        .expect("failed to add execute to the frame")
        .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_number)
        .then_signal_fence_and_flush()
        .expect("failed to signal fences and flush");


        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to present and flush!", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }


        //while the gpu is working, clean the old data
        //clean old frame
        this_frame.cleanup_finished();

        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to cleanup!", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        if should_capture{
            //also wait for the graphics card to end
            this_frame.wait(None).expect("failed to wait for graphics to debug");
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to wait for gpu!", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        //now we overwrite the internal "last_frame_end" with the finish future of this frame
        self.last_frame_end = Some(Arc::new(this_frame));

        //Resetting debug options
        if should_capture{
            //ait for the gput to mesure frame time
            let frame_time = start_time.elapsed().subsec_nanos();
            println!("\t RE: FrameTime: {}ms", frame_time as f32/1_000_000.0);
            println!("\t RE: Which is {}fps", 1.0/(frame_time as f32/1_000_000_000.0));
            self.debug_info.print_stat();
            self.engine_settings.lock().expect("failed to lock settings").stop_capture();
        }

        //update the debug info with this frame
        self.debug_info.update();
        //Box::new(after_frame)
        //now overwrite the current future
        //self.last_frame_end = Some(this_frame)
    }

    fn execute_cb_async(&self, cb: AutoCommandBuffer){
        //IMPLEMENT
    }

    ///Returns the uniform manager
    pub fn get_uniform_manager(&self) -> Arc<Mutex<uniform_manager::UniformManager>>{
        self.uniform_manager.clone()
    }

    ///Returns the pipeline manager of this renderer
    pub fn get_pipeline_manager(&mut self) -> Arc<Mutex<pipeline_manager::PipelineManager>>{
        self.pipeline_manager.clone()
    }

    ///Returns the device of this renderer
    pub fn get_device(&self) -> Arc<vulkano::device::Device>{
        self.device.clone()
    }

    ///Returns the queue of this renderer
    pub fn get_queue(&self) -> Arc<vulkano::device::Queue>{
        self.queue.clone()
    }

    ///Returns an instance of the engine settings
    ///This might be a dublicate, still helpful
    pub fn get_engine_settings(&mut self) -> Arc<Mutex<engine_settings::EngineSettings>>{
        self.engine_settings.clone()
    }

    ///Returns the available renderpasses
    pub fn get_render_passes(&self) -> Arc<Mutex<RenderPasses>>{
        self.render_passes.clone()
    }

    ///Returns the current engine state
    pub fn get_render_state(&self) -> Arc<Mutex<RenderState>>{
        self.state.clone()
    }

    fn set_working_cpu(&mut self){
        let mut state_lck = self.state.lock().expect("failed to lock render state");
        *state_lck = RenderState::work_cpu();
    }

    fn set_working_gpu(&mut self){
        let mut state_lck = self.state.lock().expect("failed to lock render state");
        *state_lck = RenderState::work_gpu();
    }
}
