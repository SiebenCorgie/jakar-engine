#version 450


//PBR_TEXTURE_USAGE
//Texture usage infos (!= 1 is "not used" for now)
layout(set = 2, binding = 0) uniform TextureUsageInfo {
  uint b_albedo;
  uint b_normal;
  uint b_metal;
  uint b_roughness;
  uint b_occlusion;
  uint b_emissive;
} u_tex_usage_info;

layout(set = 2, binding = 1) uniform TextureFactors {
  vec4 albedo_factor;
  vec3 emissive_factor;
  float max_emission;
  float normal_factor;
  float metal_factor;
  float roughness_factor;
  float occlusion_factor;
} u_tex_fac;

void main(){}
