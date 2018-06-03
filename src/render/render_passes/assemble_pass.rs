use std::sync::Arc;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
use vulkano::image::traits::ImageViewAccess;
use vulkano::image::traits::ImageAccess;
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::format::Format;

use vulkano;


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
    ///Returns the framebuffer which will draw to a `sw_images` in your swapchain
    pub fn get_fb_assemble<I>(&self, sw_images: I) -> Arc<FramebufferAbstract + Send + Sync>
        where I: ImageAccess + ImageViewAccess + Clone + Send + Sync + 'static
    {
        Arc::new(
            vulkano::framebuffer::Framebuffer::start(self.render_pass.clone())
            //Only writes to after v
            .add(sw_images).expect("failed to add assemble image")
            .build()
            .expect("failed to build assemble framebuffer!")
        )
    }
}
