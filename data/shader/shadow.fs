#version 450

layout(set = 0, binding = 1) uniform sampler2D t_Albedo;
layout(set = 0, binding = 2) uniform MaskedInfo {
  uint b_is_masked;
  float alpha_cut_off;
} u_mask_info;


layout(location = 1) in vec2 uv;


void main(){
  //Currently this only checks if it should be drawn or not depending on the
  //alpha value and the alpha_cutoff of this mateial if supplied.
  if (u_mask_info.b_is_masked == 1){
    if (texture(t_Albedo, uv).a < u_mask_info.alpha_cut_off){
      //we can discard this fragment
      discard;
    }
  }

}
