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
layout(location = 4) out mat3 v_TBN;



//Global uniforms
layout(set = 0, binding = 0) uniform Data {
  vec3 camera_position;
  mat4 model;
  mat4 view;
  mat4 proj;
} u_main;

//Generate TBN from normal only
mat3 get_tbn() {

  vec3 new_normal = normal;
  new_normal.y = -new_normal.y;

  vec3 t;
  vec3 b;
  vec3 c1 = cross(new_normal, vec3(0.0, 0.0, 1.0));
  vec3 c2 = cross(new_normal, vec3(0.0, 1.0, 0.0));
  if (length(c1) > length(c2))
    t = c1;
  else
    t = c2;
  t = normalize(t);
  b = normalize(cross(new_normal, t));

  return mat3(t,b,new_normal);
}


void main() {

  vec4 pos = u_main.model * vec4(position, 1.0);

  mat3 normal_matrix = mat3(transpose(inverse(u_main.model)));

  vec3 T = normalize(normal_matrix * tangent.xyz);
  vec3 N = normalize(normal_matrix * normal);

  //vec3 T = normalize(vec3(u_main.model * vec4(tangent.xyz, 0.0)));
  //vec3 N = normalize(vec3(u_main.model * vec4(normal, 0.0)));
  // re-orthogonalize T with respect to N
  //T = normalize(T - dot(T, N) * N);
  // then retrieve perpendicular vector B with the cross product of T and N
  vec3 B = cross(N, T) * tangent.w;

  v_TBN = mat3(T, B, N);


  FragmentPosition = vec3(pos);
  v_position = pos.xyz / pos.w;
  v_TexCoord = tex_coord;
  v_normal = mat3(transpose(inverse(u_main.model))) * normal;

  //The proj has been manipulated like here: https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
  gl_Position = u_main.proj * u_main.view * u_main.model * vec4(position, 1.0);
}
