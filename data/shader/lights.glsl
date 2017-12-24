#version 450


//contains all light definitions

//General definition (might be moved to specialisation constants later)
#define MAX_DIR_LIGHTS 6
#define MAX_POINT_LIGHTS 6
#define MAX_SPOT_LIGHTS 6

//==============================================================================
struct PointLight
{
  vec3 color;
  vec3 location;
  float intensity;
};

layout(set = 3, binding = 0) uniform point_lights{
  PointLight p_light[MAX_POINT_LIGHTS];
}u_point_light;
//==============================================================================
struct DirectionalLight
{
  vec3 color;
  vec3 direction;
  float intensity;
};

layout(set = 3, binding = 1) uniform directional_lights{
  DirectionalLight d_light[MAX_DIR_LIGHTS];
}u_dir_light;
//==============================================================================
struct SpotLight
{
  vec3 color;
  vec3 direction;
  vec3 location;

  float intensity;
  float outer_radius;
  float inner_radius;

};

layout(set = 3, binding = 2) uniform spot_lights{
  SpotLight s_light[MAX_SPOT_LIGHTS];
}u_spot_light;
//==============================================================================

//descibes the real count of lights used
layout(set = 3, binding = 3) uniform LightCount{
  uint points;
  uint directionals;
  uint spots;
}u_light_count;

void main(){}
