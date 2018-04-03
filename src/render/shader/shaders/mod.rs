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

///Blurs the attached texture based on some settings
pub mod blur;

///A easy framgent shader which only transfors default vertices to a light space
pub mod shadow_vertex;

///A fragment shader that outputs nothing but the depth calculated in the vertex shader before
pub mod shadow_fragment;
