use std::sync::Arc;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
use vulkano::format::Format;



#[derive(Clone)]
pub struct ShadowPass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
}

impl ShadowPass{
    pub fn new(
        device: Arc<Device>,
        msaa_depth_format: Format
    ) -> Self{
        let render_pass = Arc::new(
            ordered_passes_renderpass!(device.clone(),
                attachments: {
                    //The depth image
                    out_depth: {
                        load: Clear,
                        store: Store,
                        format: msaa_depth_format,
                        samples: 1,
                    }
                },
                passes:[
                    //The actual pass
                    {
                        color: [],
                        depth_stencil: {out_depth},
                        input: []
                    }
                ]

            ).expect("failed to create main render_pass")
        );

        ShadowPass{
            render_pass: render_pass,
        }
    }

}
