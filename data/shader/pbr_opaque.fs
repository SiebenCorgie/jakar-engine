#version 450

#extension GL_ARB_shading_language_420pack : enable

const float kPi = 3.14159265;


///INS FROM VERTEX
//Vertex Shader Input
layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 FragmentPosition;
layout(location = 2) in vec2 v_TexCoord;
layout(location = 3) in vec3 v_position;
layout(location = 4) in vec4 ndc_Pos;
layout(location = 5) in mat3 v_TBN;



//Global uniforms
layout(set = 0, binding = 0) uniform Data {
  vec3 camera_position;
  mat4 model;
  mat4 view;
  mat4 proj;
  float near;
  float far;
} u_main;

//TEXTURES
layout(set = 1, binding = 0) uniform sampler2D t_Albedo;
layout(set = 1, binding = 1) uniform sampler2D t_Normal;
layout(set = 1, binding = 2) uniform sampler2D t_Metall_Rough;
layout(set = 1, binding = 3) uniform sampler2D t_Occlusion;
layout(set = 1, binding = 4) uniform sampler2D t_Emissive;
//TEXTURE_USAGE
//Texture usage infos (!= 1 is "not used" for now)
layout(set = 2, binding = 0) uniform TextureUsageInfo {
  uint b_albedo;
  uint b_normal;
  uint b_metal;
  uint b_roughness;
  uint b_occlusion;
  uint b_emissive;
} u_tex_usage_info;

//TEXTURE_FACTORS
//Linear Texture factors from the material
layout(set = 2, binding = 1) uniform TextureFactors {
  vec4 albedo_factor;
  vec3 emissive_factor;
  float max_emission;
  float normal_factor;
  float metal_factor;
  float roughness_factor;
  float occlusion_factor;
} u_tex_fac;



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
//The shadow maps.
layout(set = 3, binding = 5) uniform sampler2D t_DirectionalShadows;

//==============================================================================
///outgoing final color
layout(location = 0) out vec4 f_color;
//==============================================================================
//GLOBAL VARS
vec4 albedo;
float metallic;
float roughness;
vec3 V;
vec3 surf_normal;
vec3 F0;
//==============================================================================
//Consts
const float PI = 3.14159265359;
// ----------------------------------------------------------------------------
float DistributionGGX(vec3 H)
{
    float a = roughness*roughness;
    float a2 = a*a;
    float NdotH = max(dot(surf_normal, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / denom;
}
// ----------------------------------------------------------------------------
float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}
// ----------------------------------------------------------------------------
float GeometrySmith(vec3 surf_normal, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(surf_normal, V), 0.0);
    float NdotL = max(dot(surf_normal, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}
// ----------------------------------------------------------------------------
vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

vec3 srgb_to_linear(vec3 c) {
    return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

// ----------------------------------------------------------------------------
//calculates the light falloff based on a distance and a radius
//shamlessly stolen from epics paper: Real Shading in Unreal Engine 4
//but currently using one from frostbite... have to decice...
// https://seblagarde.files.wordpress.com/2015/07/course_notes_moving_frostbite_to_pbr_v32.pdf
//Source: https://cdn2.unrealengine.com/Resources/files/2013SiggraphPresentationsNotes-26915738.pdf figure (9)
float calcFalloff(float dist, float radius){
  float invSqrAttRadius = 1/(radius * radius);
  float square_dis = dist * dist;
  float  factor = square_dis * invSqrAttRadius;
  float  smoothFactor = clamp(1.0f - factor * factor, 0.0, 1.0);

  return  smoothFactor * smoothFactor;
}

//Claculates how much shadow should be drawn.
float ShadowCalculation(vec4 fragPosLightSpace, sampler2D shadowmap, vec3 l_dir, uint pcf_fac)
{
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    projCoords = projCoords * 0.5 + 0.5;
    //Early return if we would have to sample outisde the map TODO: check the region we are in
    if(projCoords.z > 1.0 || projCoords.z < 0.0 ||
      projCoords.x > 1.0 || projCoords.x < 0.0 ||
      projCoords.y > 1.0 || projCoords.y < 0.0){
      return 0.0;
    }

    vec2 texelSize = 1.0 / textureSize(shadowmap, 0);


    float closestDepth = texture(shadowmap, projCoords.xy).r;
    float currentDepth = projCoords.z;
    float bias = max(0.05 * (1.0 - dot(v_normal, l_dir)), 0.005);
    float shadow = 0.0;

    int range_start = int(pcf_fac);
    if (pcf_fac > 0){
      //Sample in each direction and add the value to the shadow value
      for (int x = -1 * range_start; x <= range_start; ++x){
        for (int y = -1 * range_start; y <= range_start; ++y){
          //Sample the new shadow depth and compare then add
          float pcfDepth = texture(shadowmap, projCoords.xy + vec2(x, y) * texelSize).r;
          shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
        }
      }
      //Now divide the shadow value
      float samples_per_dir = ((pcf_fac * 2) + 1);
      shadow = shadow / (samples_per_dir * samples_per_dir);
    }else{
      shadow = currentDepth - bias > closestDepth  ? 1.0 : 0.0;
    }


    return shadow;
}

//Calculates a point ligh -----------------------------------------------------
vec3 calcPointLight(PointLight light, vec3 F0)
{
  // calculate per-light radiance
  vec3 L = normalize(light.location - FragmentPosition);
  vec3 H = normalize(V + L);
  float distance = length(light.location - FragmentPosition);

  float falloff = calcFalloff(distance, light.radius);
  //float attenuation = 1.0 / (distance * distance);
  vec3 radiance = light.color * light.intensity * falloff;

  // Cook-Torrance BRDF
  float NDF = DistributionGGX(H);
  float G   = GeometrySmith(surf_normal, V, L, roughness);
  vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

  vec3 nominator    = NDF * G * F;
  float denominator = 4 * max(dot(surf_normal, V), 0.0) * max(dot(surf_normal, L), 0.0) + 0.001; // 0.001 to prevent divide by zero.
  vec3 specular = nominator / denominator;

  // kS is equal to Fresnel
  vec3 kS = F;
  // for energy conservation, the diffuse and specular light can't
  // be above 1.0 (unless the surface emits light); to preserve this
  // relationship the diffuse component (kD) should equal 1.0 - kS.
  vec3 kD = vec3(1.0) - kS;
  // multiply kD by the inverse metalness such that only non-metals
  // have diffuse lighting, or a linear blend if partly metal (pure metals
  // have no diffuse light).
  kD *= 1.0 - metallic;

  // scale light by NdotL
  float NdotL = max(dot(surf_normal, L), 0.0);

  // add to outgoing radiance Lo
  return (kD * albedo.xyz / PI + specular) * radiance * NdotL;  // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again
}

//Calculates a directional light and outputs the pixel contribution------------
vec3 calcDirectionalLight(DirectionalLight light, vec3 F0)
{
  // calculate per-light radiance
  //L is always the same vector (directional light)
  vec3 L = normalize(-light.direction);
  vec3 H = normalize(V + L);

  vec3 radiance = light.color * light.intensity;

  //now darken the light contribution by the shadow value
  //but only do if we have a actual region to use
  //The region goes from x/y to z/w
  if (light.shadow_region.x != light.shadow_region.z && light.shadow_region.y != light.shadow_region.w){
    vec4 FragPosLightSpace = light.light_space * vec4(FragmentPosition, 1.0);
    float shadow = ShadowCalculation(
      FragPosLightSpace, t_DirectionalShadows, light.direction, light.pcf_samples
    );
    radiance = (1.0 - shadow) * radiance;
  }

  // Cook-Torrance BRDF
  float NDF = DistributionGGX(H);
  float G   = GeometrySmith(surf_normal, V, L, roughness);
  vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

  vec3 nominator    = NDF * G * F;
  float denominator = 4 * max(dot(surf_normal, V), 0.0) * max(dot(surf_normal, L), 0.0) + 0.001; // 0.001 to prevent divide by zero.
  vec3 specular = nominator / denominator;

  // kS is equal to Fresnel
  vec3 kS = F;
  // for energy conservation, the diffuse and specular light can't
  // be above 1.0 (unless the surface emits light); to preserve this
  // relationship the diffuse component (kD) should equal 1.0 - kS.
  vec3 kD = vec3(1.0) - kS;
  // multiply kD by the inverse metalness such that only non-metals
  // have diffuse lighting, or a linear blend if partly metal (pure metals
  // have no diffuse light).
  kD *= 1.0 - metallic;

  // scale light by NdotL
  float NdotL = max(dot(surf_normal, L), 0.0);

  // add to outgoing radiance Lo
  return (kD * albedo.xyz / PI + specular) * radiance * NdotL;  // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again
}

//Calculates a point ligh -----------------------------------------------------
vec3 calcSpotLight(SpotLight light, vec3 F0)
{
  //because of spot character we first have a look if the light is in the
  //spot and create a custom interpolation value based on it

  //if the fragment is fully in the inner circle, calculate like a spot light
  vec3 lightDir = normalize(light.location - FragmentPosition);
  float theta     = dot(lightDir, normalize(-light.direction));
  float epsilon   = light.inner_radius - light.outer_radius;
  float spot_intensity = clamp((theta - light.outer_radius) / epsilon, 0.0, 1.0);


  // calculate per-light radiance
  vec3 L = normalize(light.location - FragmentPosition);
  vec3 H = normalize(V + L);
  float distance = length(light.location - FragmentPosition);
  float falloff = calcFalloff(distance, light.radius);
  vec3 radiance = light.color * light.intensity * falloff;

  // Cook-Torrance BRDF
  float NDF = DistributionGGX(H);
  float G   = GeometrySmith(surf_normal, V, L, roughness);
  vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

  vec3 nominator    = NDF * G * F;
  float denominator = 4 * max(dot(surf_normal, V), 0.0) * max(dot(surf_normal, L), 0.0) + 0.001; // 0.001 to prevent divide by zero.
  vec3 specular = nominator / denominator;

  vec3 kS = F;
  vec3 kD = vec3(1.0) - kS;
  kD *= 1.0 - metallic;
  float NdotL = max(dot(surf_normal, L), 0.0);
  return ((kD * albedo.xyz / PI + specular) * radiance * NdotL) * spot_intensity;  // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again
}

bool isInClusters(){
  if (
    FragmentPosition.x < indice_buffer.min_extend.x ||
    FragmentPosition.y < indice_buffer.min_extend.y ||
    FragmentPosition.z < indice_buffer.min_extend.z
    ){return false;}

  if (
    FragmentPosition.x > indice_buffer.max_extend.x ||
    FragmentPosition.y > indice_buffer.max_extend.y ||
    FragmentPosition.z > indice_buffer.max_extend.z
    ){return false;}

  return true;
}

// ----------------------------------------------------------------------------
void main()
{
  if (u_tex_usage_info.b_albedo != 1) {
    albedo = u_tex_fac.albedo_factor;
  }else{
    //convert from srgb (lazy)
    albedo = texture(t_Albedo, v_TexCoord);// * u_tex_fac.albedo_factor;
    albedo.xyz = srgb_to_linear(albedo.xyz);
  }

  //Set metallic color
  if (u_tex_usage_info.b_metal != 1) {
    metallic = u_tex_fac.metal_factor;
  }else{
    metallic = texture(t_Metall_Rough, v_TexCoord).b * u_tex_fac.metal_factor;
  }

  //Set roughness color
  if (u_tex_usage_info.b_roughness != 1) {
    roughness = u_tex_fac.roughness_factor;
  }else{
    roughness = texture(t_Metall_Rough, v_TexCoord).g * u_tex_fac.roughness_factor;
  }

  //Set ao color
  float ao = 0.0;
  if (u_tex_usage_info.b_occlusion != 1) {
    ao = u_tex_fac.occlusion_factor;
  }else{
    ao = texture(t_Occlusion, v_TexCoord).r * u_tex_fac.occlusion_factor;
  }

  //Set emessive color
  vec3 emissive = vec3(0.0);
  if (u_tex_usage_info.b_emissive != 1) {
    emissive = vec3(u_tex_fac.emissive_factor * u_tex_fac.max_emission);
  }else{
    emissive = texture(t_Emissive, v_TexCoord).rgb * u_tex_fac.emissive_factor * u_tex_fac.max_emission;
  }

  //TODO implemetn emmessive
  if (u_tex_usage_info.b_normal != 1){
    //surf_normal = vec3(u_tex_fac.normal_factor);
    //from three-rs
    surf_normal = v_normal; //use the vertex normal
  }else {
    surf_normal = texture(t_Normal, v_TexCoord).rgb ;
    surf_normal = normalize(v_TBN * ((surf_normal * 2 - 1) * vec3(u_tex_fac.normal_factor, u_tex_fac.normal_factor, 1.0)));
  }
  V = normalize(u_main.camera_position - FragmentPosition);

  // calculate reflectance at normal incidence; if dia-electric (like plastic) use F0
  // of 0.04 and if sit's a metal, use the albedo color as F0 (metallic workflow)
  F0 = vec3(0.04);
  F0 = mix(F0, albedo.xyz, metallic);

  // reflectance equation
  vec3 Lo = vec3(0.0);

  //We can early check if we are inside the clusters which where calculated. If not we can skip point
  // and spotlight calculation
  if (isInClusters()){

    float x_length = indice_buffer.max_extend.x - indice_buffer.min_extend.x;
    float fragment_x_length = indice_buffer.max_extend.x - FragmentPosition.x;
    //No find out at which 1/16th of the x_length we are
    uint in_x = clamp( uint(fragment_x_length / (x_length * (1.0/float(cluster_size.x)))), 0, cluster_size.x-1);

    float y_length = indice_buffer.max_extend.y - indice_buffer.min_extend.y;
    float fragment_y_length = indice_buffer.max_extend.y - FragmentPosition.y;
    //No find out at which 1/16th of the x_length we are
    uint in_y = clamp( uint(fragment_y_length / (y_length * (1.0/float(cluster_size.y)))), 0, cluster_size.y-1);

    float z_length = indice_buffer.max_extend.z - indice_buffer.min_extend.z;
    float fragment_z_length = indice_buffer.max_extend.z - FragmentPosition.z;
    //No find out at which 1/16th of the x_length we are
    uint in_z = clamp( uint(fragment_z_length / (z_length * (1.0/float(cluster_size.z)))), 0, cluster_size.z-1);

    uint p_light_count = indice_buffer.data[cluster_size.x-1 - in_x][cluster_size.y-1 - in_y][cluster_size.z-1 - in_z].point_count;
    //Point Lights
    for(uint l_i = 0; l_i < p_light_count && l_i < 512; l_i++)
    {
      uint index = indice_buffer.data[cluster_size.x-1 - in_x][cluster_size.y-1 - in_y][cluster_size.z-1 - in_z].point_indice[l_i];
      PointLight light = u_point_light.p_light[index];
      Lo += calcPointLight(light, F0);
    }

    uint s_light_count = indice_buffer.data[cluster_size.x-1 - in_x][cluster_size.y-1 - in_y][cluster_size.z-1 - in_z].spot_count;
    //Point Lights
    for(uint l_i_s = 0; l_i_s < s_light_count && l_i_s < 512; l_i_s++)
    {
      uint index = indice_buffer.data[cluster_size.x-1 - in_x][cluster_size.y-1 - in_y][cluster_size.z-1 - in_z].spot_indice[l_i_s];
      SpotLight light = u_spot_light.s_light[index];
      Lo += calcSpotLight(light, F0);
    }

  }
/*
  //Spot Lights
  for(int i = 0; i < u_light_count.spots; i++){
    Lo += calcSpotLight(u_spot_light.s_light[i], FragmentPosition, albedo.xyz, metallic, roughness, V, surf_normal, F0);
  }
*/

  //Directional Lights
  for(int i = 0; i < u_light_count.directionals; i++){
    vec4 frag_in_light = u_dir_light.d_light[i].light_space * vec4(FragmentPosition, 1.0);
    Lo += calcDirectionalLight(u_dir_light.d_light[i], F0);
  }
  //TODO
  // ambient lighting (note that the next IBL tutorial will replace
  // this ambient lighting with environment lighting).
  vec3 ambient = vec3(0.03) * albedo.xyz * ao;

  vec3 color = ambient + Lo + emissive;

  f_color = vec4(color, albedo.a);
}
