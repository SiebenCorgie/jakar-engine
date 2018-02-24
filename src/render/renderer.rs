use render::pipeline_manager;
use render::uniform_manager;
use core::resource_management::asset_manager;
use render::window;
use render::render_helper;
use render::frame_system;
use render::post_progress;
use render::light_culling_system;
use render::render_passes::RenderPasses;

use core::next_tree::SceneTree;
use core::engine_settings;
use core::resources::camera::Camera;
//use core::simple_scene_system::node_helper;
use core::next_tree;
use jakar_tree;

use vulkano;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::swapchain::AcquireError;
use vulkano::sync::GpuFuture;
use vulkano::instance::debug::{DebugCallback, MessageTypes};


use std::sync::{Arc,Mutex};
use std::time::{Instant,Duration};
use std::mem;

///An enum describing states of the renderer
#[derive(Eq, PartialEq)]
pub enum RendererState {
    RUNNING,
    WAITING,
    SHOULD_END,
    ENDED
}

///The main renderer. Should be created through a RenderBuilder
pub struct Renderer  {
    ///Holds the renderers pipeline_manager
    pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,


    //window: vulkano_win::Window,
    window: window::Window,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    swapchain: Arc<vulkano::swapchain::Swapchain>,
    images: Vec<Arc<vulkano::image::SwapchainImage>>,

    frame_system: frame_system::FrameSystem,
    light_culling_system: light_culling_system::PreDpethSystem,

    render_passes: RenderPasses,

    ///The post progresser
    post_progress: post_progress::PostProgress,


    //Is true if we need to recreate the swap chain
    recreate_swapchain: bool,

    engine_settings: Arc<Mutex<engine_settings::EngineSettings>>,
    uniform_manager: Arc<Mutex<uniform_manager::UniformManager>>,

    state: Arc<Mutex<RendererState>>,
}

impl Renderer {
    ///Creates a new renderer from all the systems. However, you should only use the builder to create
    /// a renderer.
    pub fn create_for_builder(
        pipeline_manager: Arc<Mutex<pipeline_manager::PipelineManager>>,
        window: window::Window,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
        swapchain: Arc<vulkano::swapchain::Swapchain>,
        images: Vec<Arc<vulkano::image::SwapchainImage>>,
        //renderpass: Arc<RenderPassAbstract + Send + Sync>,

        //the used frame system
        frame_system: frame_system::FrameSystem,

        post_progress: post_progress::PostProgress,
        render_passes: RenderPasses,
        light_culling_system: light_culling_system::PreDpethSystem,

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
            render_passes: render_passes,
            light_culling_system: light_culling_system,
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
    pub fn check_image_state(&self) -> Result<(usize, SwapchainAcquireFuture), AcquireError>{

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
    fn check_pipeline(&mut self) -> Result<(usize, SwapchainAcquireFuture), AcquireError>{
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
        //previous_frame: Box<GpuFuture>,
    ){

        //Sync gput
        let this_frame: Box<GpuFuture> = Box::new(vulkano::sync::now(self.device.clone()));

        //First of all we get info if we should debug anything, if so this bool will be true
        let (should_capture, mut time_step, start_time) = {
            let cap_bool = self.engine_settings.lock().expect("failed to lock settings").capture_frame;
            let time_step = Instant::now();
            let start_time = Instant::now();
            (cap_bool, time_step, start_time)
        };
        let (image_number, acquire_future) = {
            match self.check_pipeline(){
                Ok(k) => {
                    k
                },
                Err(e) => {
                    println!("Could not get next swapchain image: {}", e);
                    //early return to restart the frame
                    return; // this_frame;
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
        self.light_culling_system.update_light_set();

        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to update the light set!", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        //now execute the compute shader
        command_buffer = self.light_culling_system.dispatch_compute_shader(
            command_buffer,
        );
        if should_capture{
            let time_needed = time_step.elapsed().subsec_nanos();
            println!("\tRE: Nedded {} ms to dispatch compute shader", time_needed as f32 / 1_000_000.0);
            time_step = Instant::now()
        }

        //Now we can end this stage (Pre compute)
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
        if (self.engine_settings.lock().expect("failed to lock settings")).get_render_settings().draw_bounds(){
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
                if (self.engine_settings.lock().expect("failed to lock settings")).get_render_settings().draw_bounds(){
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
                    let all_point_lights = asset_manager.get_active_scene().copy_all_point_lights(&None);
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

                    let all_spot_lights = asset_manager.get_active_scene().copy_all_spot_lights(&None);
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

        //perform the post progressing
        command_buffer = self.post_progress.execute(
            command_buffer,
            &self.frame_system
        );


        if should_capture{
            println!("\tRE: Added postprogress thingy", );
        }


        //now finish the frame
        command_buffer = self.frame_system.next_pass(command_buffer);
        let finished_command_buffer = {
            match self.frame_system.finish_frame(command_buffer){
                Ok(cb) => cb,
                Err(er) =>{
                    println!("{}", er);
                    return; // previous_frame;
                }
            }
        };

        if should_capture{
            println!("\tRE: Ending frame", );
        }


        //thanks firewater
        let real_cb = finished_command_buffer
        .end_render_pass().expect("failed to end command buffer")
        .build().expect("failed to build command buffer");


        let after_prev_and_aq = this_frame.join(acquire_future);

        let before_present_frame = after_prev_and_aq.then_execute(self.queue.clone(), real_cb)
        .expect("failed to add execute to the frame");

        //test copy the depth buffer as the show image

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


        //Resetting debug options
        if should_capture{
            let frame_time = start_time.elapsed().subsec_nanos();
            println!("\t RE: FrameTime: {}ms", frame_time as f32/1_000_000.0);
            println!("\t RE: Which is {}fps", 1.0/(frame_time as f32/1_000_000_000.0));
            self.engine_settings.lock().expect("failed to lock settings").stop_capture();
        }

        //Box::new(after_frame)
    }

    ///adds a `node` to the `command_buffer` if possible to be rendered.
    fn add_forward_node(
        &mut self, node: &jakar_tree::node::Node<
            next_tree::content::ContentType,
            next_tree::jobs::SceneJobs,
            next_tree::attributes::NodeAttributes>,
        frame_stage: frame_system::FrameStage)
    -> frame_system::FrameStage
    where AutoCommandBufferBuilder: Sized + 'static
    {

        let mut time_step = Instant::now();


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
                    material.get_set_04(&mut self.light_culling_system)
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
