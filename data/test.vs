#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec4 tangent;
layout(location = 4) in vec4 color;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 FragmentPosition;
layout(location = 2) out vec2 tex_coordinates;
layout(location = 3) out mat3 TBN;



//Global uniforms
layout(set = 0, binding = 0) uniform Data {
  vec3 camera_position;
  mat4 model;
  mat4 view;
  mat4 proj;
} u_main;


void main() {

  //new
  v_normal = mat3(transpose(inverse(u_main.model))) * normal;

  //v_normal = mat3(u_main.model) * normal;
  tex_coordinates = tex_coord;

  //Create TBN
  vec3 T = normalize(vec3(u_main.model * tangent));
  vec3 N = normalize(vec3(u_main.model * vec4(normal, 0.0)));
  // re-orthogonalize T with respect to N
  //T = normalize(T - dot(T, N) * N);
  // then retrieve perpendicular vector B with the cross product of T and N
  vec3 B = cross(N, T);

  TBN = mat3(T, B, N);

  //todo test if thats right
  FragmentPosition = vec3(u_main.model * vec4(position, 1.0));

  gl_Position = u_main.proj * u_main.view * u_main.model * vec4(position, 1.0);
}
