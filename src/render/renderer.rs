use render::pipeline_manager;
use render::uniform_manager;
use core::resource_management::asset_manager;
use render::window;
use render::render_helper;
use core::engine_settings;
use core::resources::camera::Camera;
//use core::simple_scene_system::node_helper;
use core::next_tree;
use jakar_tree;

use vulkano;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::swapchain::SwapchainCreationError;
use vulkano::swapchain::SwapchainAcquireFuture;
use vulkano::swapchain::AcquireError;
use vulkano::sync::GpuFuture;
use vulkano::instance::debug::{DebugCallback, MessageTypes};


use std::sync::{Arc,Mutex};
use std::time::{Instant};
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
    renderpass: Arc<RenderPassAbstract + Send + Sync>,
    depth_buffer: Arc<vulkano::image::AttachmentImage<vulkano::format::D16Unorm>>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,

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
        renderpass: Arc<RenderPassAbstract + Send + Sync>,
        depth_buffer: Arc<vulkano::image::AttachmentImage<vulkano::format::D16Unorm>>,
        framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
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
            renderpass: renderpass,
            depth_buffer: depth_buffer,
            framebuffers: framebuffers,
            recreate_swapchain: recreate_swapchain,
            engine_settings: engine_settings,
            uniform_manager: uniform_manager,
            state: state,
        }
    }

    ///Recreates swapchain for the window size in `engine_settings`
    ///Returns true if successfully recreated chain
    pub fn recreate_swapchain(&mut self) -> bool{
        //get new dimmensions etc
        let mut engine_settings_lck = self.engine_settings
        .lock()
        .expect("Faield to lock settings");

        let c_d = self.window.get_current_extend();
        let (new_width, new_height) = (c_d[0], c_d[1]);
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
        use std::time::Duration;

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

    ///Renders the scene with the parameters supplied by the asset_manager
    ///and returns the future of this frame. The future can be joint to wait for the gpu
    ///or be supplied to the next update();
    pub fn render(
        &mut self,
        asset_manager: &mut asset_manager::AssetManager,
        previous_frame: Box<GpuFuture>,
    ) -> Box<GpuFuture>{

        let (image_number, acquire_future) = {
            match self.check_pipeline(){
                Ok(k) => {
                    k
                },
                Err(e) => {
                    println!("Could not get next swapchain image: {}", e);
                    //early return to restart the frame
                    return previous_frame;
                }
            }
        };

        //now we can actually start the frame
        //get all opaque meshes
        let opaque_meshes = asset_manager.get_all_meshes(
            Some(next_tree::SceneComparer::new().without_transparency())
        );
        //get all translucent meshes
        let translucent_meshes = asset_manager.get_all_meshes(
            Some(next_tree::SceneComparer::new().with_transparency())
        );
        //now send the translucent meshes to another thread for ordering
        let trans_recv = render_helper::order_by_distance(
            translucent_meshes, asset_manager.get_camera()
        );

        //While the cpu is gathering the the translucent meshes based on the distance to the
        //camera, we start to build the command buffer for the opaque meshes, unordered actually.
        //1st.:get the dimensions of the current image and start a command buffer builder for it
        //Get the dimensions to fill the dynamic vieport setting per mesh.
        let dimensions = self.window.get_current_extend();

        //start the command buffer
        let mut command_buffer: AutoCommandBufferBuilder =
            vulkano::command_buffer::AutoCommandBufferBuilder::new(
                self.device.clone(),
                self.queue.family()
            )
            .expect("failed to create tmp buffer!")
            .begin_render_pass(
                self.framebuffers[image_number].clone(), false,
                vec![
                    [0.1, 0.1, 0.1, 1.0].into(),
                    1f32.into()
                ]
            ).expect("failed to clear");

        //we now have the start for a command buffer. In a later engine
        //stage at this point a compute command is added for the forward+ early depth pass.
        // we are currently rendering only in a normal forward manor.

        //add all opaque meshes to the command buffer

        for opaque_mesh in opaque_meshes.iter(){
            command_buffer = self.add_node_to_command_buffer(opaque_mesh, command_buffer, dimensions);
        }


        //now try to get the ordered list of translucent meshes and add them as well
        match trans_recv.recv(){
            Ok(ord_tr) => {

                for translucent_mesh in ord_tr.iter(){
                    command_buffer = self.add_node_to_command_buffer(translucent_mesh, command_buffer, dimensions);
                }

            },
            Err(er) => {
                println!("Something went wrong while ordering the translucent meshes: {}", er);
            }
        }

        //now we can end the frame. In the future this might be the time to add post processing
        //etc. but not for now ;)

        //thanks firewater
        let real_cb = command_buffer
        .end_render_pass().expect("failed to end command buffer")
        .build().expect("failed to build command buffer");


        let after_prev_and_aq = previous_frame.join(acquire_future);

        let before_present_frame = after_prev_and_aq.then_execute(self.queue.clone(), real_cb)
        .unwrap();

        //now present to the image
        let after_present_frame = vulkano::swapchain::present(
            self.swapchain.clone(),
            before_present_frame, self.queue.clone(),
            image_number
        );
        //now signal fences
        let mut after_frame = after_present_frame.then_signal_fence_and_flush().unwrap();

        //while the gpu is working, clean the old data
        //clean old frame
        after_frame.cleanup_finished();

        Box::new(after_frame)


    }

    ///adds a `node` to the `command_buffer` if possible to be rendered.
    fn add_node_to_command_buffer(
        &self, node: &jakar_tree::node::Node<
            next_tree::content::ContentType,
            next_tree::jobs::SceneJobs,
            next_tree::attributes::NodeAttributes>,
        command_buffer: AutoCommandBufferBuilder,
        dimensions: [u32; 2])
    -> AutoCommandBufferBuilder
    where AutoCommandBufferBuilder: Sized + 'static
    {

        //get the actual mesh as well as its pipeline an create the descriptor sets
        let mesh_locked = match node.value{
            next_tree::content::ContentType::Mesh(ref mesh) => mesh.clone(),
            _ => return command_buffer, //is no mesh :(
        };

        let mesh = mesh_locked.lock().expect("failed to lock mesh in cb creation");

        let mesh_transform = node.attributes.get_matrix();

        let material_locked = mesh.get_material();
        let mut material = material_locked
        .lock()
        .expect("failed to lock mesh for command buffer generation");

        let pipeline = material.get_vulkano_pipeline();

        let set_01 = {
            //aquirre the tranform matrix and generate the new set_01
            material.get_set_01(mesh_transform)
        };

        let set_02 = {
            material.get_set_02()
        };

        let set_03 = {
            material.get_set_03()
        };

        let set_04 = {
            material.get_set_04()
        };

        //extend the current command buffer by this mesh
        command_buffer
            .draw_indexed(
                pipeline,

                vulkano::command_buffer::DynamicState{
                    line_width: None,
                    viewports: Some(vec![vulkano::pipeline::viewport::Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                        depth_range: 0.0 .. 1.0,
                    }]),
                    scissors: None,
                },
                mesh
                .get_vertex_buffer(), //vertex buffer (static usually)

                mesh
                .get_index_buffer(
                    self.device.clone(), self.queue.clone()
                ).clone(), //index buffer
                (set_01, set_02, set_03, set_04), //descriptor sets (currently static)
                ()
            )
            .expect("Failed to draw in command buffer!")
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
