use std::sync::Arc;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::device::Device;
use vulkano::format::Format;





///Is able to blur fragments based on settings supplied with the first descriptor set
#[derive(Clone)]
pub struct BlurPass {
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
}

impl BlurPass {
    pub fn new(
        device: Arc<Device>,
        hdr_msaa_format: Format,
    ) -> Self{
        let render_pass = Arc::new(
            ordered_passes_renderpass!(device.clone(),
                attachments: {
                    //The blured fragments
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
