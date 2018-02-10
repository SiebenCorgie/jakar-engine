#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 tex_coord;

layout(location = 0) out vec2 inter_coord;
layout(location = 1) out vec2 v_pos;

void main()
{
    inter_coord = tex_coord;
    v_pos = position;
    gl_Position = vec4(position.x, position.y, 0.0, 1.0);
}
