#version 450




//tries to get the input attachment
layout(set = 0, binding = 0) uniform sampler2D color_input;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInputMS depths_input;
layout(set = 0, binding = 2) uniform sampler2D hdr_fragments;
layout(set = 0, binding = 3) uniform sampler2D average_lumiosity;
layout(set = 0, binding = 4) uniform sampler2D dir_depth;

//Get the uvs
layout(location = 0) in vec2 inter_coord;
layout(location = 1) in vec2 v_pos;


//outputs the fragment color
layout(location = 0) out vec4 FragColor;

//The inputs for the hdr -> ldr pass
layout(set = 1, binding = 0) uniform hdr_settings{
  float gamma;
  float near;
  float far;
  float use_auto_exposure;
  int sampling_rate;
  int show_mode;
}u_hdr_settings;

///Will hold the average lumiosity of this frame
layout(set = 1, binding = 1) buffer LumiosityBuffer{
  float this_average_lumiosity;
  float last_average_lumiosity;
  float exposure;
} u_lum_buf;

float linear_depth(float depth){
  float f= u_hdr_settings.far;
  float n = u_hdr_settings.near;
  float z = (2 * n) / (f + n - depth * (f - n));
  return z;
}

void main()
{
  //MainDepth
  if (u_hdr_settings.show_mode == 0) {

    float depth_out = subpassLoad(depths_input, 1).x;

    float z = linear_depth(depth_out);

    FragColor = vec4(vec3(z), 1.0);
    return;
  }

  //DebugGrid
  if (u_hdr_settings.show_mode == 1) {
    //shows a grid of all the attachments for debuging
    //color_input (unmapped)

    if ((inter_coord.x < 0.5 && inter_coord.x > 0.0) && (inter_coord.y < 0.5 && inter_coord.y > 0.0)){
      //sample the image
      vec2 coords = (inter_coord * 2.0);
      FragColor = texture(color_input, coords);
    }

    //hdr_fragments only
    else if ((inter_coord.x < 1.0 && inter_coord.x > 0.5) && (inter_coord.y < 0.5 && inter_coord.y > 0.0)){
      //sample the image
      //shrink
      vec2 coords = inter_coord * 2.0;
      //offset to the right
      coords.x = coords.x - 1.0;
      FragColor = texture(hdr_fragments, coords);
    }
    //average lumiosity
    else if ((inter_coord.x < 0.5 && inter_coord.x > 0.0) && (inter_coord.y < 1.0 && inter_coord.y > 0.5)){
      //sample the image
      //shrink
      vec2 coords = inter_coord * 2.0;
      //offset down
      coords.y = coords.y - 1.0;
      FragColor = texture(average_lumiosity, coords);
    }else{
      //If we came here, return nothing
      FragColor = vec4(vec3(0.0), 1.0);
    }
    return;
  }

  //ShadowMaps
  if (u_hdr_settings.show_mode == 2) {
    float depth = texture(dir_depth, inter_coord).r;
    FragColor = vec4(vec3(depth), 1.0);
    return;
  }
  //DirectionalDepth
  if (u_hdr_settings.show_mode == 3) {
    float depth = texture(dir_depth, inter_coord).r;
    FragColor = vec4(vec3(depth), 1.0);
    return;
  }


  //Add the blur to the image
  vec3 hdrColor = texture(color_input, inter_coord).rgb;
  vec3 bloomColor = texture(hdr_fragments, inter_coord).rgb;


  hdrColor += bloomColor; // additive blending

  float exposure;
  //This value is something else then 0.0 if we don't wnat to use the auto value;
  if (u_hdr_settings.use_auto_exposure == 0.0){
    exposure = u_lum_buf.exposure;
  }else{
    exposure = u_hdr_settings.use_auto_exposure;
  }

  // Exposure tone mapping
  vec3 mapped = vec3(1.0) - exp(-hdrColor * u_lum_buf.exposure);
  // Gamma correction
  mapped = pow(mapped, vec3(1.0 / u_hdr_settings.gamma));



  FragColor = vec4(mapped, 1.0);

}
