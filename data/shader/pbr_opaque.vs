#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec4 tangent;
layout(location = 4) in vec4 color;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 FragmentPosition;
layout(location = 2) out vec2 v_TexCoord;
layout(location = 3) out vec3 v_position;
layout(location = 4) out vec4 ndc_Pos;
layout(location = 5) out mat3 v_TBN;



//Global uniforms
layout(set = 0, binding = 0) uniform Data {
  vec3 camera_position;
  mat4 model;
  mat4 view;
  mat4 proj;
  float near;
  float far;
} u_main;


void main() {

  vec4 pos = u_main.model * vec4(position, 1.0);

  mat3 normal_matrix = transpose(inverse(mat3(u_main.model)));

  vec3 T = normalize(normal_matrix * tangent.xyz);
  vec3 N = normalize(normal_matrix * normal);

  //vec3 T = normalize(vec3(u_main.model * vec4(tangent.xyz, 0.0)));
  //vec3 N = normalize(vec3(u_main.model * vec4(normal, 0.0)));
  // re-orthogonalize T with respect to N
  //T = normalize(T - dot(T, N) * N);
  // then retrieve perpendicular vector B with the cross product of T and N
  vec3 B = normalize(cross(N, T) * tangent.w);

  v_TBN = mat3(T, B, N);



  FragmentPosition = vec3(pos);
  v_position = position;
  v_TexCoord = tex_coord;
  v_normal = normalize(normal_matrix * normal);

  //The proj has been manipulated like here: https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
  ndc_Pos = u_main.proj * u_main.view * u_main.model * vec4(position, 1.0);
  gl_Position = ndc_Pos;
}
