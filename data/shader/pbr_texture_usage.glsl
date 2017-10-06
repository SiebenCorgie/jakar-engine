//TEXTURE_USAGE
//Texture usage infos (!= 1 is "not used" for now)
layout(set = 2, binding = 0) uniform TextureUsageInfo {
  uint b_albedo;
  uint b_normal;
  uint b_metal;
  uint b_roughness;
  uint b_occlusion;
  uint b_emissive;
} u_tex_usage_info;
