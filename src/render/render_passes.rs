use std::sync::{Arc, Mutex};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano;

use core::engine_settings::EngineSettings;

#[derive(Clone)]
pub struct ObjectPass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
}

impl ObjectPass{
    pub fn new(device: Arc<Device>, msaa_factor: u32, hdr_msaa_format: Format, msaa_depth_format: Format) -> Self{

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
                    //The non hdr framgnets
                    ldr_fragments: {
                        load: Clear,
                        store: Store,
                        format: hdr_msaa_format,
                        samples: 1,
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
                        color: [ldr_fragments, hdr_fragments],
                        depth_stencil: {},
                        input: [raw_render_color]
                    }
                ]

            ).expect("failed to create main render_pass")
        );

        ObjectPass{
            render_pass: main_renderpass,
        }
    }

}

///Is able to blur fragments based on settings supplied with the first descriptor set
#[derive(Clone)]
pub struct BlurPass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
}

impl BlurPass {
    pub fn new(device: Arc<Device>, hdr_msaa_format: Format) -> Self{
        let render_pass = Arc::new(
            ordered_passes_renderpass!(device.clone(),
                attachments: {
                    //The non hdr framgnets
                    out_hdr_fragments: {
                        load: Clear,
                        store: Store,
                        format: hdr_msaa_format,
                        samples: 1,
                    }
                },
                passes:[
                    //The actual pass
                    {
                        color: [out_hdr_fragments],
                        depth_stencil: {},
                        input: []
                    }
                ]

            ).expect("failed to create main render_pass")
        );

        BlurPass{
            render_pass: render_pass,
        }
    }
}

///Takes currently:
/// - LDR Fragments
/// - Blured hdr fragments
///
/// Thoose are assembled to the final imaged and then tone mapping etc. is applied
#[derive(Clone)]
pub struct AssemblePass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
}

impl AssemblePass{
    pub fn new(device: Arc<Device>, swapchain_format: Format) -> Self{
        let render_pass = Arc::new(
            ordered_passes_renderpass!(device.clone(),
                attachments: {
                    final_image: {
                        load: Clear,
                        store: Store,
                        format: swapchain_format,
                        samples: 1,
                    }
                },
                passes:[
                    //The actual pass
                    {
                        color: [final_image],
                        depth_stencil: {},
                        input: []
                    }
                ]

            ).expect("failed to create main render_pass")
        );

        AssemblePass{
            render_pass: render_pass,
        }
    }
}

//TODO create pass

///A collection of the available render pass definitions.
#[derive(Clone)]
pub struct RenderPasses {
    ///Renderst the objects in a forward manor, in a second pass the msaa is resolved and the image
    /// is split in a hdr and ldr part.
    pub object_pass: ObjectPass,
    ///Blurs the first texture and writes it blured based on settings to the output
    pub blur_pass: BlurPass,
    ///Takes all the generated images and combines them to the final image
    pub assemble: AssemblePass,


    //Holds all the used formats
    pub image_hdr_msaa_format: Format,
    pub image_msaa_depth_format: Format,
    pub swapchain_format: Format,
    pub static_msaa_factor: u32,
}

impl RenderPasses{
    pub fn new(device: Arc<Device>, swapchain_format: Format, settings: Arc<Mutex<EngineSettings>>) -> Self{

        let hdr_msaa_format = vulkano::format::Format::R16G16B16A16Sfloat;
        let msaa_depth_format = vulkano::format::Format::D16Unorm;

        let msaa_factor = {
            let mut set_lck = settings.lock().expect("failed to lock settings");
            set_lck.get_render_settings().get_msaa_factor()
        };


        let object_pass = ObjectPass::new(device.clone(),  msaa_factor, hdr_msaa_format, msaa_depth_format);
        let blur_pass = BlurPass::new(device.clone(), hdr_msaa_format);
        let assemble = AssemblePass::new(device.clone(), swapchain_format);

        RenderPasses{
            object_pass: object_pass,
            blur_pass: blur_pass,
            assemble: assemble,

            image_hdr_msaa_format: hdr_msaa_format,
            image_msaa_depth_format: msaa_depth_format,
            swapchain_format: swapchain_format,
            static_msaa_factor: msaa_factor,
        }
    }

    pub fn conf_to_pass(&self, conf: RenderPassConf) -> Arc<RenderPassAbstract + Send + Sync>{
        match conf{
            RenderPassConf::ObjectPass => self.object_pass.render_pass.clone(),
            RenderPassConf::BlurPass => self.blur_pass.render_pass.clone(),
            RenderPassConf::AssemblePass => self.assemble.render_pass.clone(),
        }
    }
}

///Enum listing all the render passes availabe. They need to be known since we build pipelines against them
/// later.
#[derive(PartialEq, Clone)]
pub enum RenderPassConf{
    ///Currently renders everything, from the objects to the post progress.
    ObjectPass,
    BlurPass,
    AssemblePass,
}
