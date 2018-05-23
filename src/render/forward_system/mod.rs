use core::engine_settings;
use render;
use render::render_helper;
use render::frame_system::FrameSystem;
use render::light_system::LightSystem;
use render::post_progress::PostProgress;
use render::pipeline;
use core::resource_management::asset_manager::AssetManager;
use core::next_tree::{SceneTree, ValueTypeBool, SceneComparer};
use core::next_tree::content::ContentType;
use core::resources::camera::Camera;

use vulkano::image::traits::ImageViewAccess;
use vulkano::image::traits::ImageAccess;
use vulkano::image::attachment::AttachmentImage;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::format::Format;
use vulkano::image::ImageUsage;
use vulkano::image::StorageImage;
use vulkano::image::Dimensions;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano;

use std::sync::{Arc, Mutex};



///Collects all thingy which are needed to render all objects in the forward pass
pub struct ForwardSystem {
    engine_settings:  Arc<Mutex<engine_settings::EngineSettings>>,

    //a copy of the device
    device: Arc<vulkano::device::Device>,
    //a copy of the queue
    queue: Arc<vulkano::device::Queue>,

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



        ForwardSystem{
            engine_settings,
            device,
            queue,
            resolve_pipe,
        }
    }

    ///renders several forward shadeable nodes in this asset managers active scene.
    ///Returns the CommandBuffer passless
    pub fn do_forward_shading(
        &self,
        frame_system: &FrameSystem,
        light_system: &LightSystem,
        post_progress: &PostProgress,
        asset_manager: &mut AssetManager,
        command_buffer: AutoCommandBufferBuilder,
    ) -> AutoCommandBufferBuilder{

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



        //Draw opaque meshes
        //now we are in the main render pass in the forward pass, using this to draw all meshes
        //add all opaque meshes to the command buffer
        for opaque_mesh in opaque_meshes.iter(){
            let transform = opaque_mesh.get_attrib().get_matrix();

            if let ContentType::Mesh(ref mesh) = opaque_mesh.get_value(){


                let mesh_lck = mesh.lock().expect("failed to lock mesh for drawing!");
                new_cb = mesh_lck.draw(
                    new_cb,
                    frame_system,
                    light_system,
                    transform,
                );


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
                );
            }else{
                println!("Mesh was no actual mesh...", );
                continue;
            }
        }


        //TODO draw debug stuff

        //Now change to the hdr sorting pass
        let next_stage = new_cb.next_subpass(false).expect("failed to change to Hdr Sorting render pass");

        //now draw to the sorted image
        let mut final_cb = self.sort_hdr(next_stage, frame_system, post_progress);
        //finally end this pass end return
        final_cb = final_cb.end_render_pass().expect("failed to end object pass");

        final_cb
    }

    ///Sorts the current rendered image to an hdr fragments only image
    fn sort_hdr(&self,
        command_buffer: AutoCommandBufferBuilder,
        frame_system: &FrameSystem,
        post_progress: &PostProgress,
    ) -> AutoCommandBufferBuilder{
        //create the descriptor set for the current image
        let attachments_ds = PersistentDescriptorSet::start(self.resolve_pipe.get_pipeline_ref(), 0) //at binding 0
            .add_image(frame_system.get_passes().object_pass.get_images().forward_hdr_image.clone())
            .expect("failed to add hdr_image to sorting pass descriptor set")
            .build()
            .expect("failed to build postprogress cb");

        //the settings for this pass
        let settings = post_progress.get_hdr_settings();

        let settings_buffer = PersistentDescriptorSet::start(self.resolve_pipe.get_pipeline_ref(), 1) //At binding 1
            .add_buffer(settings)
            .expect("failed to add hdr image settings buffer to post progress attachment")
            .build()
            .expect("failed to build settings attachment for postprogress pass");

        //perform the post progress
        let new_command_buffer = command_buffer.draw(
            self.resolve_pipe.get_pipeline_ref(),
            frame_system.get_dynamic_state().clone(),
            vec![post_progress.get_screen_vb()],
            (attachments_ds, settings_buffer),
            ()
        ).expect("failed to add draw call for the sorting plane");

        new_command_buffer
    }
}
