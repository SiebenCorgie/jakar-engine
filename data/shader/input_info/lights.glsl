#version 450


//LIGHTS
//==============================================================================
//Represents a single cluster
struct Cluster{
  uint point_count;
  uint spot_count;
  uint point_indice[512];
  uint spot_indice[512];
};

const uvec3 cluster_size = uvec3(32,16,32);
//Represents all clusters in the 3d grid
layout(set = 3, binding = 0) readonly buffer ClusterBuffer {
  vec3 min_extend;
  vec3 max_extend;
  Cluster data[cluster_size.x][cluster_size.y][cluster_size.z];
} indice_buffer;


struct PointLight
{
  vec3 color;
  vec3 location;
  float intensity;
  float radius;
};

layout(set = 3, binding = 1) readonly buffer point_lights{
  PointLight p_light[];
}u_point_light;
//==============================================================================
struct DirectionalLight
{
  vec4 shadow_region;
  mat4 light_space;
  vec3 color;
  vec3 direction;
  float intensity;
  uint pcf_samples;
};

layout(set = 3, binding = 2) readonly buffer directional_lights{
  DirectionalLight d_light[];
}u_dir_light;
//==============================================================================
struct SpotLight
{
  vec3 color;
  vec3 direction;
  vec3 location;

  float intensity;
  float radius;
  float outer_radius;
  float inner_radius;

};

layout(set = 3, binding = 3) readonly buffer spot_lights{
  SpotLight s_light[];
}u_spot_light;
//==============================================================================

//descibes the count of lights used
layout(set = 3, binding = 4) uniform LightCount{
  uint points;
  uint directionals;
  uint spots;
}u_light_count;
//==============================================================================

void main(){}
