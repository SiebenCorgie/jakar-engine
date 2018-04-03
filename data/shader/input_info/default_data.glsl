#version 450


//Global uniforms describing
// current camera position
// model transform matrix (rotation, scale and location)
// view matrix
// projection matrix (corrected to be used with gl_Position and depth)
layout(set = 0, binding = 0) uniform Data {
  vec3 camera_position;
  mat4 model;
  mat4 view;
  mat4 proj;
  float near;
  float far;
} u_main;

//Global uniforms
layout(set = 0, binding = 1) uniform LightData {
  mat4 model;
  mat4 viewproj;
} u_light_main;

void main(){}
