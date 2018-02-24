#version 450


//contains all light definitions

//layout (constant_id = 0) const int TEST_CONST = 512;
/*layout (constant_id = 1) const int MAX_DIR_LIGHTS = 6; */
/*layout (constant_id = 2) const int MAX_SPOT_LIGHTS = 512; */

//==============================================================================
struct PointLight
{
  vec3 color;
  vec3 location;
  float intensity;
  float radius;
};

layout(set = 3, binding = 0) buffer point_lights{
  PointLight p_light[];
}u_point_light;
//==============================================================================
struct DirectionalLight
{
  vec3 color;
  vec3 direction;
  float intensity;
};

layout(set = 3, binding = 1) buffer directional_lights{
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

layout(set = 3, binding = 2) buffer spot_lights{
  SpotLight s_light[];
}u_spot_light;
//==============================================================================

//descibes the count of lights used
layout(set = 3, binding = 3) uniform LightCount{
  uint points;
  uint directionals;
  uint spots;
}u_light_count;

void main(){}
