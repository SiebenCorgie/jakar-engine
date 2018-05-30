use core::engine_settings;
use render;
use render::render_helper;
use render::frame_system::FrameSystem;
use render::light_system::LightSystem;
use render::post_progress::PostProgress;
use render::pipeline;
use core::resource_management::asset_manager::AssetManager;
use core::next_tree::{SceneTree, ValueTypeBool, SceneComparer,JakarNode};
use core::next_tree::content::ContentType;
use core::resources::camera::Camera;
use render::renderer::RenderDebug;
use render::shader::shaders::hdr_resolve;

use tools::callbacks::*;


use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::*;

use jakar_threadpool::*;


///Collects all thingy which are needed to render all objects in the forward pass
pub struct ForwardSystem {
    engine_settings:  Arc<Mutex<engine_settings::EngineSettings>>,

    //a copy of the device
    device: Arc<vulkano::device::Device>,
    //a copy of the queue
    queue: Arc<vulkano::device::Queue>,

    sort_buffer_pool: CpuBufferPool<hdr_resolve::ty::hdr_settings>,
    sort_desc_pool: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract + Send + Sync>>,

    ///A pipeline used to sort hdr fragments
    resolve_pipe: Arc<pipeline::Pipeline>,


}


impl ForwardSystem{
    pub fn new(
        engine_settings:  Arc<Mutex<engine_settings::EngineSettings>>,
        device: Arc<vulkano::device::Device>,
        queue: Arc<vulkano::device::Queue>,
        resolve_pipe: Arc<pipeline::Pipeline>,
    ) -> Self{


        let sort_buffer_pool = CpuBufferPool::uniform_buffer(device.clone());
        let sort_desc_pool = FixedSizeDescriptorSetsPool::new(resolve_pipe.get_pipeline_ref(), 0);


        ForwardSystem{
            engine_settings,
            device,
            queue,
            resolve_pipe,
            sort_buffer_pool,
            sort_desc_pool
        }
    }

    ///renders several forward shadeable nodes in this asset managers active scene.
    ///Returns the CommandBuffer passless
    pub fn do_forward_shading(
        &mut self,
        frame_system: &FrameSystem,
        light_system: &LightSystem,
        post_progress: &PostProgress,
        asset_manager: &mut AssetManager,
        command_buffer: AutoCommandBufferBuilder,
        thread_pool: &mut ThreadPool,
        debug: &mut RenderDebug,
    ) -> AutoCommandBufferBuilder{

        debug.start_node_getting();

        let mesh_comparer = SceneComparer::new()
        .with_value_type(ValueTypeBool::none().with_mesh())
        .with_frustum(asset_manager.get_camera().get_frustum_bound())
        .with_cull_distance(0.1, asset_manager.get_camera().get_view_projection_matrix())
        .without_transparency();

        let mesh_comp_trans = mesh_comparer.clone()
        .with_transparency();
        //now we can actually start the frame
        //get all opaque meshes
        let opaque_meshes = asset_manager
        .get_active_scene()
        .copy_all_nodes(&Some(mesh_comparer));
        //get all translucent meshes
        let translucent_meshes = asset_manager
        .get_active_scene()
        .copy_all_nodes(&Some(mesh_comp_trans));
        //now send the translucent meshes to another thread for ordering
        let trans_recv = render_helper::order_by_distance(
            translucent_meshes, asset_manager.get_camera()
        );

        debug.end_node_getting();

        //Go into the forward shading stage
        //first get the framebuffer for the forward pass
        let forward_frame_buffer = frame_system.get_passes().object_pass.get_framebuffer();

        //For successfull clearing we generate a vector for all images.
        let clearing_values = vec![
            [0.0, 0.0, 0.0, 1.0].into(), //forward color hdr
            1f32.into(), //forward depth
            [0.0, 0.0, 0.0, 1.0].into(),
            [0.0, 0.0, 0.0, 1.0].into(), //post progress / frame buffer image
            //1f32.into(), //
        ];

        //Start forward pass
        let mut new_cb = command_buffer.begin_render_pass(forward_frame_buffer, false, clearing_values)
            .expect("failed to start main renderpass");

        let mut draw_count = 0;

        //Warp into an option to make ownership transfering easier

        //Draw opaque meshes
        //now we are in the main render pass in the forward pass, using this to draw all meshes
        //add all opaque meshes to the command buffer
        for opaque_mesh in opaque_meshes.iter(){
            let transform = opaque_mesh.get_attrib().get_matrix();

            if let ContentType::Mesh(ref mesh) = opaque_mesh.get_value(){

                debug.start_mesh_capture();


                let mesh_lck = mesh.lock().expect("failed to lock mesh for drawing!");
                /*
                let mesh_draw_call = mesh_lck.get_draw_call(
                    frame_system,
                    light_system,
                    transform,
                    debug
                );


                new_cb = mesh_draw_call.call_box(new_cb);


                */
                new_cb = mesh_lck.draw(
                    new_cb,
                    frame_system,
                    light_system,
                    transform,
                    debug
                );


                debug.end_mesh_capture();


                draw_count += 1;

            }else{
                println!("Mesh was no actual mesh...", );
                continue;
            }
        }

        //Now recive the translucent ones and draw them
        let trans_meshses = trans_recv.recv().expect("failed to recive translucent meshes");
        //now we are in the main render pass in the forward pass, using this to draw all meshes
        //add all opaque meshes to the command buffer
        for trans_mesh in trans_meshses.iter(){
            let transform = trans_mesh.get_attrib().get_matrix();

            if let ContentType::Mesh(ref mesh) = trans_mesh.get_value(){


                let mesh_lck = mesh.lock().expect("failed to lock mesh for drawing!");

                new_cb = mesh_lck.draw(
                    new_cb,
                    frame_system,
                    light_system,
                    transform,
                    debug,
                );

                draw_count += 1;


            }else{
                println!("Mesh was no actual mesh...", );
                continue;
            }
        }

        debug.set_draw_calls(draw_count);


        //TODO draw debug stuff

        //Now change to the hdr sorting pass
        let next_stage = new_cb.next_subpass(false).expect("failed to change to Hdr Sorting render pass");

        //now draw to the sorted image
        let mut final_cb = self.sort_hdr(next_stage, frame_system, post_progress);
        //finally end this pass end return
        final_cb = final_cb.end_render_pass().expect("failed to end object pass");

        final_cb
    }
/* An option to generate the drawcalls however not implemented yet
    ///Takes a collection of nodes and creates a collection of drawcalls from them
    fn gen_draw_calls(&self
        frame_system: &FrameSystem,
        light_system: &LightSystem,
        thread_pool: &mut ThreadPool,
        all_nodes: Vec<JakarNode>,
        debug: &mut RenderDebug,
    ) -> Vec<Box<FnCbBox + Send + 'static>>{
        //Gonna prepare a collection of up to n draw calls each and supply them to the thread pool,
        //gonna wait for the returned draw calls and compine those

        let batch_size = 100;
        let mut current_batch_num = 0;
        let mut draw_call_collection: Vec<Vec<JakarNode>> = Vec::new();
        let mut current_batch = Vec::new();
        while(!all_nodes.is_empty()){
            current_batch.push(all_nodes.pop().expect("poped an empty mesh while creating draw call batch"));
            current_batch_num += 1;

            //If we reached the final size, push batch to batch collection and start again
            if current_batch_num >= batch_size{
                //Transfere current batch into a new vec and append the new vec
                let final_batch = Vec::new().append(&mut current_batch);
                draw_call_collection.push(final_batch);
                current_batch_num = 0;
            }
        }

        //Now spawn a process for each batch which will generate the draw call. When finished, send
        //them to this thread again and push all of them into one vec

        let reciver_collection = Vec::new();


        for batch in draw_call_collection{

            let (send, recv) = channel::<Vec<Box<FnCbBox + Send + 'static>>>()

            thread_pool.execute(||{

            })
        }

    }
    */

    ///Sorts the current rendered image to an hdr fragments only image
    fn sort_hdr(&mut self,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
        post_progress: &PostProgress,
    ) -> AutoCommandBufferBuilder{

        let (sampling_rate, bloom_brightness) = {
            let samples = frame_system.get_passes().static_msaa_factor;
            let brightness = self.engine_settings
            .lock().expect("failed to get settings").get_render_settings().get_bloom().brightness;

            (samples, brightness)
        };


        let hdr_resolve_settings = hdr_resolve::ty::hdr_settings{
            sampling_rate: sampling_rate,
            bloom_brightness: bloom_brightness,
        };

        let settings_buffer = self.sort_buffer_pool.next(hdr_resolve_settings)
        .expect("Failed to get sorting settings");

        let sorting_attachment = self.sort_desc_pool.next()
        .add_image(frame_system.get_passes().object_pass.get_images().forward_hdr_image.clone())
        .expect("failed to add hdr_image to sorting pass descriptor set")
        .add_buffer(settings_buffer)
        .expect("failed to add hdr image settings buffer to post progress attachment")
        .build()
        .expect("failed to build hdr sorting descriptor");


        //perform the post progress
        let new_command_buffer = command_buffer.draw(
            self.resolve_pipe.get_pipeline_ref(),
            frame_system.get_dynamic_state().clone(),
            vec![post_progress.get_screen_vb()],
            sorting_attachment,
            ()
        ).expect("failed to add draw call for the sorting plane");

        new_command_buffer
    }
}
