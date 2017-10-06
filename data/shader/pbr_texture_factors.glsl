#version 450


//PBR_TEXTURE_FACTORS
//Linear Texture factors from the material
layout(set = 2, binding = 1) uniform TextureFactors {
  vec4 albedo_factor;
  vec3 emissive_factor;
  float normal_factor;
  float metal_factor;
  float roughness_factor;
  float occlusion_factor;
} u_tex_fac;

void main(){}
