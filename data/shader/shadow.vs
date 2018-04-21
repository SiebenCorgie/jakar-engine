#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec4 tangent;
layout(location = 4) in vec4 color;

layout(location = 1) out vec4 test;


//Global uniforms
layout(set = 0, binding = 0) uniform LightData {
  mat4 model;
  mat4 viewproj;
} u_light_main;

void main(){
  //The proj has been manipulated like here: https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
  vec4 ndc_Pos = u_light_main.viewproj * u_light_main.model * vec4(position, 1.0);
  test = vec4(0.0);
  gl_Position = ndc_Pos;
}
