use std::sync::{Arc, Mutex};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano;

use core::engine_settings::EngineSettings;

#[derive(Clone)]
pub struct ObjectPass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
    pub image_hdr_msaa_format: Format,
    pub image_msaa_depth_format: Format,
    pub swapchain_format: Format,
    pub static_msaa_factor: u32,
}

impl ObjectPass{
    pub fn new(device: Arc<Device>, swapchain_format: Format, msaa_factor: u32,) -> Self{

        let hdr_msaa_format = vulkano::format::Format::R16G16B16A16Sfloat;
        let msaa_depth_format = vulkano::format::Format::D16Unorm;

        //Setup the render_pass layout for the forward pass
        let main_renderpass = Arc::new(
            ordered_passes_renderpass!(device.clone(),
                attachments: {
                    // The first framebuffer attachment is the raw_render_color image.
                    raw_render_color: {
                        load: Clear,
                        store: Store,
                        format: hdr_msaa_format, //Defined that it works by the vulkan implementation
                        samples: msaa_factor,     // This has to match the image definition.
                    },

                    //the second one is the msaa depth buffer
                    raw_render_depth: {
                        load: Clear,
                        store: DontCare,
                        format: msaa_depth_format, //works per vulkan definition
                        samples: msaa_factor,
                    },

                    //Holds only fragments with at leas one value over 1.0
                    hdr_fragments: {
                        load: Clear,
                        store: Store,
                        format: hdr_msaa_format,
                        samples: 1,
                    },

                    //the final image
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain_format, //this needs to have the format which is presentable to the window
                        samples: 1, //target image is not sampled
                    }
                },
                passes:[
                    {
                        color: [raw_render_color],
                        depth_stencil: {raw_render_depth},
                        input: []
                    },

                    //Resolves msaa and creates a HDR fragment buffer
                    {
                        color: [hdr_fragments],
                        depth_stencil: {},
                        input: [raw_render_color]
                    },

                    {
                        color: [color],
                        depth_stencil: {},
                        input: [raw_render_color]
                        //resolve: [color]
                    }

                ]

            ).expect("failed to create main render_pass")
        );

        ObjectPass{
            render_pass: main_renderpass,
            image_hdr_msaa_format: hdr_msaa_format,
            image_msaa_depth_format: msaa_depth_format,
            swapchain_format: swapchain_format,
            static_msaa_factor: msaa_factor,
        }
    }

}

///A collection of the available render pass definitions.
#[derive(Clone)]
pub struct RenderPasses {
    pub object_pass: ObjectPass,
}

impl RenderPasses{
    pub fn new(device: Arc<Device>, swapchain_format: Format, settings: Arc<Mutex<EngineSettings>>) -> Self{

        let msaa_factor = {
            let mut set_lck = settings.lock().expect("failed to lock settings");
            set_lck.get_render_settings().get_msaa_factor()
        };


        let object_pass = ObjectPass::new(device, swapchain_format, msaa_factor);

        RenderPasses{
            object_pass: object_pass,
        }
    }

    pub fn conf_to_pass(&self, conf: RenderPassConf) -> Arc<RenderPassAbstract + Send + Sync>{
        match conf{
            RenderPassConf::ObjectPass => self.object_pass.render_pass.clone()
        }
    }
}

///Enum listing all the render passes availabe. They need to be known since we build pipelines against them
/// later.
#[derive(PartialEq, Clone)]
pub enum RenderPassConf{
    ///Currently renders everything, from the objects to the post progress.
    ObjectPass,
}
