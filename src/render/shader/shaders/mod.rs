///The normal PBR fragment shader
pub mod pbr_fragment;

///The normal PBR vertex shader
pub mod pbr_vertex;

///A wireframe shader for debuging
pub mod wireframe_vertex;

///A wireframe shader for debuging
pub mod wireframe_fragment;

///The default post progress vertex shader
pub mod default_pstprg_vertex;

///The default post progress fragment shader (does all the work)
pub mod default_pstprg_fragment;

///Sort out HDR fragments for later processing
pub mod hdr_resolve;
