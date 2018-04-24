///Defines the default data struct for an shader containing:
/// - Camera Position (vec3)
/// - Model Transform (mat4)
/// - View Matrix (mat4)
/// - Projection Matrix (mat4)
pub mod default_data;

///Defines light types
pub mod lights;

///Defines the PBR Texture info, like which texture is used and what a parameters of them.
pub mod pbr_texture_info;

///Defines the texture sets usable in a pbr material
pub mod pbr_texture_sets;


///Keeps track of all the available shader inputs grouped by Descriptorset
#[derive(Clone)]
pub enum DescriptorSetFamiliy{
    //Data for the objects
    CameraData,
    Lights,
    MaterialTextures,
    MaterialData,
    ShadowMaskInfo,
    CascadedCameraInfo,
    SingleShadowInfo,
    //PostProgress general data like exposure etc
    PostProgressData,
    BlurSettings,
    //attachments
    MultisampledColorAndDepth,
    MultisampledColor,
    DebugGrid,
}
