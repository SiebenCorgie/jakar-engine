#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec4 tangent;
layout(location = 4) in vec4 color;



//Global uniforms
layout(set = 0, binding = 0) uniform Data {
  vec3 camera_position;
  mat4 model;
  mat4 view;
  mat4 proj;
} u_main;


void main() {
  //The proj has been manipulated like here: https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
  gl_Position = u_main.proj * u_main.view * u_main.model * vec4(position, 1.0);
}
