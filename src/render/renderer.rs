use render::pipeline_manager;
use render::uniform_manager;
use core::resource_management::asset_manager;
use render::window;
use render::window::Window;
use render::render_helper;
use render::frame_system;
use render::post_progress;
use render::light_system;
use render::render_passes::RenderPasses;
use render::shadow_system;

use core::next_tree::{SceneTree, ValueTypeBool, SceneComparer};
use core::engine_settings;
//use core::simple_scene_system::node_helper;
use core::next_tree;
use jakar_tree;

use winit;
use vulkano;
use vulkano::instance::Instance;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::swapchain::AcquireError;
use vulkano::sync::GpuFuture;
use vulkano::sync::FenceSignalFuture;


use std::sync::{Arc,Mutex};
use std::time::{Instant};
use std::mem;
use std::sync::mpsc;

///An enum describing states of the renderer
#[derive(Eq, PartialEq)]
pub enum RendererState {
    RUNNING,
    WAITING,
    ENDED
}

///Used tp build a render instance
pub trait BuildRender {
    ///Build a renderer based on settings and a window which will recive the iamges
    fn build(
        self,
        instance_sender: mpsc::Sender<Arc<Instance>>,
        window_reciver: mpsc::Receiver<Window>,
        engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
    ) -> Result<(Renderer, Box<GpuFuture>), String>;
}

///The main renderer. Should be created through a RenderBuilder
pub struct Renderer  {
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
    light_system: light_system::LightSystem,

    render_passes: RenderPasses,

    ///The post progresser
    post_progress: post_progress::PostProgress,

    //Is true if we need to recreate the swap chain
    recreate_swapchain: bool,

    engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,

    state: Arc<Mutex<RendererState>>,
    //last_frame: Arc<FenceSignalFuture<GpuFuture>>,
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
        //renderpass: Arc<RenderPassAbstract + Send + Sync>,

        //the used frame system
        frame_system: frame_system::FrameSystem,

        render_passes: RenderPasses,
        shadow_system: shadow_system::ShadowSystem,
        light_system: light_system::LightSystem,
        post_progress: post_progress::PostProgress,

        recreate_swapchain: bool,
        engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
        uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,
        state: Arc<Mutex<RendererState>>,
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
            render_passes: render_passes,
            light_system: light_system,
            post_progress: post_progress,

            recreate_swapchain: recreate_swapchain,
            engine_settings: engine_settings,
            uniform_manager: uniform_manager,
            state: state,
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
        //previous_frame: Arc<FenceSignalFuture<GpuFuture>>,
    ){

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
                    return; //this_frame;
                }
            }
        };

        if should_capture{
            time_step = Instant::now();
        }

        //now we can actually start the frame
        //get all opaque meshes
        let opaque_meshes = asset_manager.get_meshes_in_frustum(
            Some(next_tree::SceneComparer::new().without_transparency())
        );
        //get all translucent meshes
        let translucent_meshes = asset_manager.get_meshes_in_frustum(
            Some(next_tree::SceneComparer::new().with_transparency())
        );
        //now send the translucent meshes to another thread for ordering
        let trans_recv = render_helper::order_by_distance(
            translucent_meshes, asset_manager.get_camera()
        );

        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("RENDER INFO: ", );
            println!("\tRE: Nedded {} ms to get all opaque meshes", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        //While the cpu is gathering the the translucent meshes based on the distance to the
        //camera, we start to build the command buffer for the opaque meshes, unordered actually.


        //get out selfs the image we want to render to
        //start the frame
        let mut command_buffer = self.frame_system.new_frame(
            self.images[image_number].clone()
        );

        //First of all we compute the light clusters
        //Since we currently have no nice system to track
        self.light_system.update_light_set(&mut self.shadow_system, asset_manager);

        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to update the light set!", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        //now execute the compute shader
        command_buffer = self.light_system.dispatch_compute_shader(
            command_buffer,
        );
        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to dispatch compute shader", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        //Now we can end this stage (Pre compute)
        command_buffer = self.frame_system.next_pass(command_buffer);

        //Its time to render all the shadow maps...
        command_buffer = self.shadow_system.render_shadows(
            command_buffer,
            &self.frame_system,
            asset_manager
        );

        //change to the forward pass
        command_buffer = self.frame_system.next_pass(command_buffer);

        //now we are in the main render pass in the forward pass, using this to draw all meshes
        //add all opaque meshes to the command buffer
        for opaque_mesh in opaque_meshes.iter(){
            command_buffer = self.add_forward_node(opaque_mesh, command_buffer);
        }
        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to draw all opaque meshes", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        //now draw debug data of the meshes if turned on
        if sould_draw_bounds{
            //draw all opaque
            for mesh in opaque_meshes.iter(){
                command_buffer = render_helper::add_bound_draw(
                     command_buffer,
                     self.pipeline_manager.clone(),
                     mesh,
                     self.device.clone(),
                     self.uniform_manager.clone(),
                     &self.frame_system.get_dynamic_state()
                 );
            }
            if should_capture{
                let time_needed = time_step.elapsed().subsec_nanos();
                println!("\tRE: Nedded {} ms to draw all mesh bounds!", time_needed as f32 / 1_000_000.0);
                time_step = Instant::now()
            }
        }


        //now try to get the ordered list of translucent meshes and add them as well
        match trans_recv.recv(){
            Ok(ord_tr) => {

                for translucent_mesh in ord_tr.iter(){
                    command_buffer = self.add_forward_node(
                        translucent_mesh, command_buffer
                    );
                }

                if should_capture{
                    let time_needed = time_step.elapsed().subsec_nanos();
                    println!("\tRE: Nedded {} ms to draw all transparent meshes!", time_needed as f32 / 1_000_000.0);
                    time_step = Instant::now()
                }

                //now draw debug data of the meshes if turned on
                if sould_draw_bounds{
                    //draw the transparent bounds
                    for mesh in ord_tr.iter(){
                        command_buffer = render_helper::add_bound_draw(
                             command_buffer,
                             self.pipeline_manager.clone(),
                             mesh,
                             self.device.clone(),
                             self.uniform_manager.clone(),
                             &self.frame_system.get_dynamic_state()
                         );
                    }

                    if should_capture{
                        let time_needed = time_step.elapsed().subsec_nanos();
                        println!("\tRE: Nedded {} ms to draw all transparent mesh bounds!", time_needed as f32 / 1_000_000.0);
                        time_step = Instant::now()
                    }
                    //also draw the light bounds
                    let all_point_lights = asset_manager.get_active_scene().copy_all_nodes(
                        &Some(SceneComparer::new().with_value_type(
                            ValueTypeBool::none().with_point_light()
                        )));

                    for light in all_point_lights.iter(){
                        command_buffer = render_helper::add_bound_draw(
                             command_buffer,
                             self.pipeline_manager.clone(),
                             light,
                             self.device.clone(),
                             self.uniform_manager.clone(),
                             &self.frame_system.get_dynamic_state()
                         );
                    }

                    let all_spot_lights = asset_manager.get_active_scene().copy_all_nodes(
                        &Some(SceneComparer::new().with_value_type(
                            ValueTypeBool::none().with_spot_light()
                        )));
                        
                    for light in all_spot_lights.iter(){
                        command_buffer = render_helper::add_bound_draw(
                             command_buffer,
                             self.pipeline_manager.clone(),
                             light,
                             self.device.clone(),
                             self.uniform_manager.clone(),
                             &self.frame_system.get_dynamic_state()
                         );
                    }
                    if should_capture{
                        let time_needed = time_step.elapsed().subsec_nanos();
                        println!("\tRE: Nedded {} ms to draw light bounds!", time_needed as f32 / 1_000_000.0);
                        time_step = Instant::now()
                    }
                }

            },
            Err(er) => {
                println!("Something went wrong while ordering the translucent meshes: {}", er);
            }
        }



        if should_capture{
            println!("\tRE: Finished adding meshes", );
        }


        //finished the forward pass, change to the postprogressing pass
        command_buffer = self.frame_system.next_pass(command_buffer);

        if should_capture{
            println!("\tRE: Changed to subpass", );
        }

        //Sort HDR's
        command_buffer = self.post_progress.sort_hdr(
            command_buffer,
            &self.frame_system
        );

        //Performe next pass
        command_buffer = self.frame_system.next_pass(command_buffer);

        //Do BlurH
        command_buffer = self.post_progress.execute_blur(
            command_buffer,
            &self.frame_system,
        );

        //Change to BlurV
        command_buffer = self.frame_system.next_pass(command_buffer);
        //Do BlurV
        command_buffer = self.post_progress.execute_blur(
            command_buffer,
            &self.frame_system,
        );

        //Change to compute average lumiosity stage
        command_buffer = self.frame_system.next_pass(command_buffer);
        //Compute the lumiosity
        //We only do the compute thing if we really wanna use the compute shader
        let use_auto_exposure = {
            self.engine_settings
            .lock()
            .expect("failed to lock settings")
            .get_render_settings()
            .get_exposure().use_auto_exposure
        };


        if use_auto_exposure{
            command_buffer = self.post_progress.compute_lumiosity(
                command_buffer,
                &self.frame_system,
            );
        }


        //Change to assamble stage
        command_buffer = self.frame_system.next_pass(command_buffer);

        //perform the post progressing
        command_buffer = self.post_progress.assemble_image(
            command_buffer,
            &self.frame_system
        );


        if should_capture{
            println!("\tRE: Added postprogress thingy", );
        }


        //now finish the frame
        command_buffer = self.frame_system.next_pass(command_buffer);
        //And retrieve the ended command buffer
        let finished_command_buffer = {
            match self.frame_system.finish_frame(command_buffer){
                Ok(cb) => cb,
                Err(er) =>{
                    println!("{}", er);
                    return;// this_frame;
                }
            }
        };

        if should_capture{
            println!("\tRE: Ending frame", );
        }

        //now since we did everything we can in this frame, wait for the last frame and start the new one
        //previous_frame.wait(None).expect_err("Failed to wait for last frame !");
        let this_frame = Box::new(vulkano::sync::now(self.device.clone()));

        //thanks firewater
        let real_cb = finished_command_buffer.build().expect("failed to build command buffer");

        let after_prev_and_aq = this_frame.join(acquire_future);

        let before_present_frame = after_prev_and_aq.then_execute(self.queue.clone(), real_cb)
        .expect("failed to add execute to the frame");


        //now present to the image
        let after_present_frame = vulkano::swapchain::present(
            self.swapchain.clone(),
            before_present_frame, self.queue.clone(),
            image_number
        );
        //now signal fences
        let mut after_frame = after_present_frame.then_signal_fence_and_flush().expect("failed to signal and flush");

        //while the gpu is working, clean the old data
        //clean old frame
        after_frame.cleanup_finished();

        //now wait for the graphics card to finish. However, I might implement some better way.
        // It would be nice if the engine could record the next command buffer while the last frame is executing.
        after_frame.wait(None).expect("Could not wait for gpu");

        //Resetting debug options
        if should_capture{
            //ait for the gput to mesure frame time


            let frame_time = start_time.elapsed().subsec_nanos();
            println!("\t RE: FrameTime: {}ms", frame_time as f32/1_000_000.0);
            println!("\t RE: Which is {}fps", 1.0/(frame_time as f32/1_000_000_000.0));
            self.engine_settings.lock().expect("failed to lock settings").stop_capture();

            //Since we waited for the gpu, we can return a now GpuFuture
            return; // Box::new(vulkano::sync::now(self.device.clone()));
        }

        //Box::new(after_frame)
    }

    ///adds a `node` to the `command_buffer` if possible to be rendered.
    fn add_forward_node(
        &mut self,
        node: &jakar_tree::node::Node<
            next_tree::content::ContentType,
            next_tree::jobs::SceneJobs,
            next_tree::attributes::NodeAttributes
        >,
        frame_stage: frame_system::FrameStage
    )
    -> frame_system::FrameStage
    where AutoCommandBufferBuilder: Sized + 'static
    {
        match frame_stage{
            frame_system::FrameStage::Forward(cb) => {
                //get the actual mesh as well as its pipeline an create the descriptor sets
                let mesh_locked = match node.value{
                    next_tree::content::ContentType::Mesh(ref mesh) => mesh.clone(),
                    _ => return frame_system::FrameStage::Forward(cb), //is no mesh :(
                };

                //println!("Needed {}ms to get mesh lock", time_step.elapsed().subsec_nanos() as f32 / 1_000_000.0);
                //time_step = Instant::now();

                let mesh = mesh_locked.lock().expect("failed to lock mesh in cb creation");

                let mesh_transform = node.attributes.get_matrix();

                let material_locked = mesh.get_material();
                let mut material = material_locked
                .lock()
                .expect("failed to lock mesh for command buffer generation");

                let pipeline = material.get_vulkano_pipeline();

                //println!("Needed {}ms Till sets", time_step.elapsed().subsec_nanos() as f32 / 1_000_000.0);
                //time_step = Instant::now();

                let set_01 = {
                    //aquirre the tranform matrix and generate the new set_01
                    material.get_set_01(mesh_transform)
                };

                //println!("Needed {}ms set 01", time_step.elapsed().subsec_nanos() as f32 / 1_000_000.0);
                //time_step = Instant::now();

                let set_02 = {
                    material.get_set_02()
                };

                //println!("Needed {}ms set 02", time_step.elapsed().subsec_nanos() as f32 / 1_000_000.0);
                //time_step = Instant::now();

                let set_03 = {
                    material.get_set_03()
                };

                //println!("Needed {}ms set 03", time_step.elapsed().subsec_nanos() as f32 / 1_000_000.0);
                //time_step = Instant::now();

                let set_04 = {
                    material.get_set_04(&mut self.light_system, &self.frame_system)
                };

                //println!("Needed {}ms set 04", time_step.elapsed().subsec_nanos() as f32 / 1_000_000.0);
                //time_step = Instant::now();

                //extend the current command buffer by this mesh
                let new_cb = cb.draw_indexed(
                    pipeline,
                    self.frame_system.get_dynamic_state().clone(),
                    mesh.get_vertex_buffer(), //vertex buffer (static usually)
                    mesh.get_index_buffer(), //index buffer
                    (set_01, set_02, set_03, set_04), //descriptor sets (currently static)
                    ()
                )
                .expect("Failed to draw in command buffer!");


                //println!("Needed {}ms to draw", time_step.elapsed().subsec_nanos() as f32 / 1_000_000.0);
                //time_step = Instant::now();

                return frame_system::FrameStage::Forward(new_cb);
            },
            _ => {
                println!("Tried to draw mesh in wrong stage!", );
            }
        }

        return frame_stage;

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
    pub fn get_render_passes(&self) -> RenderPasses{
        self.render_passes.clone()
    }

    ///Returns the current engine state
    pub fn get_engine_state(&self) -> Arc<Mutex<RendererState>>{
        self.state.clone()
    }
}
