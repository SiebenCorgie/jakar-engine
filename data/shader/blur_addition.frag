#version 450

///Adds up to eight images on top of each other.

//Image to be blured
layout(set = 0, binding = 0) uniform sampler2D img_one;
//Possible image which can be added on top of the current one.
layout(set = 0, binding = 1) uniform sampler2D img_two;

//The inputs for the hdr -> ldr pass
layout(set = 0, binding = 2) uniform blur_settings{
  int is_horizontal;
  int add_second;
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

  vec2 tex_offset = 1.0 / textureSize(img_one, 0); // gets size of single texel
  vec4 result = texture(img_one, inter_coord) * weight[0]; // current fragment's contribution

  vec2 uvs = (inter_coord / 3.0) * 4.0;

  //TODO remove branch by lerping the offset vector
  if(u_blur_settings.is_horizontal == 1)
  {
      for(int i = 1; i < 5; ++i)
      {
          result += texture(img_one, uvs + vec2(tex_offset.x * i, 0.0)) * weight[i];
          result += texture(img_one, uvs - vec2(tex_offset.x * i, 0.0)) * weight[i];
      }
  }
  else
  {
      for(int i = 1; i < 5; ++i)
      {
          result += texture(img_one, uvs + vec2(0.0, tex_offset.y * i)) * weight[i];
          result += texture(img_one, uvs - vec2(0.0, tex_offset.y * i)) * weight[i];
      }
  }

  //Add he second image to the output
  if (u_blur_settings.add_second == 1){
    result += texture(img_two, uvs);
  }


  BluredImage = result;
}
