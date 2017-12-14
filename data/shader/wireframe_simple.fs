#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec4 color;

///outgoing final color
layout(location = 0) out vec4 f_color;

void main() {
  f_color = color;
}
