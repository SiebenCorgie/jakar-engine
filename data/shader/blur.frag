#version 450

//The sampleable input_tex used for bluring
layout(set = 0, binding = 0) uniform sampler2D input_tex;

//The inputs for the hdr -> ldr pass
layout(set = 0, binding = 1) uniform blur_settings{
  int horizontal;
  //TODO verfiy
  //float weight[5];
  float scale;
  float strength;

}u_blur_settings;


//Get the uvs
layout(location = 0) in vec2 inter_coord;
layout(location = 1) in vec2 v_pos;


//outputs the fragment color
layout(location = 0) out vec4 BluredImage;

void main()
{

  float weight[5];
  weight[0] = 0.227027;
  weight[1] = 0.1945946;
  weight[2] = 0.1216216;
  weight[3] = 0.054054;
  weight[4] = 0.016216;

  vec2 tex_offset = 1.0 / textureSize(input_tex, 0) * u_blur_settings.scale; // gets size of single texel
  vec3 result = texture(input_tex, inter_coord).rgb * weight[0]; // current fragment's contribution
  if(u_blur_settings.horizontal == 1)
  {
      for(int i = 1; i < 5; ++i)
      {
          result += texture(input_tex, inter_coord + vec2(tex_offset.x * i, 0.0)).rgb * weight[i] * u_blur_settings.strength;
          result += texture(input_tex, inter_coord - vec2(tex_offset.x * i, 0.0)).rgb * weight[i] * u_blur_settings.strength;
      }
  }
  else
  {
      for(int i = 1; i < 5; ++i)
      {
          result += texture(input_tex, inter_coord + vec2(0.0, tex_offset.y * i)).rgb * weight[i] * u_blur_settings.strength;
          result += texture(input_tex, inter_coord - vec2(0.0, tex_offset.y * i)).rgb * weight[i] * u_blur_settings.strength;
      }
  }
  BluredImage = vec4(result, 1.0);


}
