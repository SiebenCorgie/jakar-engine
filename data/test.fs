#version 450

//General definition
#define MAX_DIR_LIGHTS 6
#define MAX_POINT_LIGHTS 6
#define MAX_SPOT_LIGHTS 6

const float kPi = 3.14159265;


///INS FROM VERTEX
//Vertex Shader Input
layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 FragmentPosition;
layout(location = 2) in vec2 v_TexCoord;
layout(location = 3) in vec3 v_position;
layout(location = 4) in mat3 v_TBN;


//Global uniforms
layout(set = 0, binding = 0) uniform Data {
  vec3 camera_position;
  mat4 model;
  mat4 view;
  mat4 proj;
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
  float normal_factor;
  float metal_factor;
  float roughness_factor;
  float occlusion_factor;
} u_tex_fac;

//LIGHTS
//definitions of the lights for the unsized arrays
struct PointLight
{
  vec3 color;
  vec3 location;
  float intensity;
};

struct DirectionalLight
{
  vec3 color;
  vec3 direction;
  vec3 location;
  float intensity;
};

struct SpotLight
{
  vec3 color;
  vec3 direction;
  vec3 location;

  float intensity;
  float outer_radius;
  float inner_radius;

};


//And the send bindings from rust/vulkano
layout(set = 3, binding = 0) uniform point_lights{
  PointLight p_light[MAX_POINT_LIGHTS];
}u_point_light;

layout(set = 3, binding = 1) uniform directional_lights{
  DirectionalLight d_light[MAX_DIR_LIGHTS];
}u_dir_light;

layout(set = 3, binding = 2) uniform spot_lights{
  SpotLight s_light[MAX_SPOT_LIGHTS];
}u_spot_light;

layout(set = 3, binding = 3) uniform LightCount{
  uint points;
  uint directionals;
  uint spots;
}u_light_count;

///outgoing final color
layout(location = 0) out vec4 f_color;


const float PI = 3.14159265359;
// ----------------------------------------------------------------------------
// Easy trick to get tangent-normals to world-space to keep PBR code simplified.
// Don't worry if you don't get what's going on; you generally want to do normal
// mapping the usual way for performance anways; I do plan make a note of this
// technique somewhere later in the normal mapping tutorial.
vec3 getNormalFromMap()
{
    vec3 tangentNormal = texture(t_Normal, v_TexCoord).xyz * 2.0 - 1.0;

    vec3 Q1  = dFdx(FragmentPosition);
    vec3 Q2  = dFdy(FragmentPosition);
    vec2 st1 = dFdx(v_TexCoord);
    vec2 st2 = dFdy(v_TexCoord);

    vec3 N   = normalize(v_normal);
    vec3 T  = normalize(Q1*st2.t - Q2*st1.t);
    vec3 B  = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);

    return normalize(TBN * tangentNormal);
}
// ----------------------------------------------------------------------------
float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a = roughness*roughness;
    float a2 = a*a;
    float NdotH = max(dot(N, H), 0.0);
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
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}
// ----------------------------------------------------------------------------
vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

//Calculates a point ligh -----------------------------------------------------
vec3 calcPointLight(PointLight light, vec3 FragmentPosition, vec3 albedo, float metallic, float roughness, vec3 V, vec3 N, vec3 F0)
{
  // calculate per-light radiance
  vec3 L = normalize(light.location - FragmentPosition);
  vec3 H = normalize(V + L);
  float distance = length(light.location - FragmentPosition);
  float attenuation = 1.0 / (distance * distance);
  vec3 radiance = light.color * light.intensity * attenuation;

  // Cook-Torrance BRDF
  float NDF = DistributionGGX(N, H, roughness);
  float G   = GeometrySmith(N, V, L, roughness);
  vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

  vec3 nominator    = NDF * G * F;
  float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001; // 0.001 to prevent divide by zero.
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
  float NdotL = max(dot(N, L), 0.0);

  // add to outgoing radiance Lo
  return (kD * albedo / PI + specular) * radiance * NdotL;  // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again
}

//Calculates a directional light and outputs the pixel contribution------------
vec3 calcDirectionalLight(DirectionalLight light, vec3 FragmentPosition, vec3 albedo, float metallic, float roughness, vec3 V, vec3 N, vec3 F0)
{
  // calculate per-light radiance
  //L is always the same vector (directional light)
  vec3 L = normalize(-light.direction);
  vec3 H = normalize(V + L);

  vec3 radiance = light.color * light.intensity;

  // Cook-Torrance BRDF
  float NDF = DistributionGGX(N, H, roughness);
  float G   = GeometrySmith(N, V, L, roughness);
  vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

  vec3 nominator    = NDF * G * F;
  float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001; // 0.001 to prevent divide by zero.
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
  float NdotL = max(dot(N, L), 0.0);

  // add to outgoing radiance Lo
  return (kD * albedo / PI + specular) * radiance * NdotL;  // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again
}

//Calculates a point ligh -----------------------------------------------------
vec3 calcSpotLight(SpotLight light, vec3 FragmentPosition, vec3 albedo, float metallic, float roughness, vec3 V, vec3 N, vec3 F0)
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
  float attenuation = 1.0 / (distance * distance);
  vec3 radiance = light.color * light.intensity * attenuation;

  // Cook-Torrance BRDF
  float NDF = DistributionGGX(N, H, roughness);
  float G   = GeometrySmith(N, V, L, roughness);
  vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

  vec3 nominator    = NDF * G * F;
  float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001; // 0.001 to prevent divide by zero.
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
  float NdotL = max(dot(N, L), 0.0);

  // add to outgoing radiance Lo
  return ((kD * albedo / PI + specular) * radiance * NdotL) * spot_intensity;  // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again
}

vec3 srgb_to_linear(vec3 c) {
    return mix(c / 12.92, pow((c + 0.055) / 1.055, vec3(2.4)), step(0.04045, c));
}

// ----------------------------------------------------------------------------
void main()
{
  //Set albedo color
  vec3 albedo = vec3(0.0);
  if (u_tex_usage_info.b_albedo != 1) {
    albedo = u_tex_fac.albedo_factor.xyz;
  }else{
    albedo = pow(texture(t_Albedo, v_TexCoord).rgb, vec3(2.2)) * u_tex_fac.albedo_factor.xyz;
  }

  //Set metallic color
  float metallic = 0.0;
  if (u_tex_usage_info.b_metal != 1) {
    metallic = u_tex_fac.metal_factor;
  }else{
    metallic = texture(t_Metall_Rough, v_TexCoord).g * u_tex_fac.metal_factor;
  }

  //Set roughness color
  float roughness = 0.0;
  if (u_tex_usage_info.b_roughness != 1) {
    roughness = u_tex_fac.roughness_factor;
  }else{
    roughness = texture(t_Metall_Rough, v_TexCoord).b * u_tex_fac.roughness_factor;
  }

  //Set ao color
  float ao = 0.0;
  if (u_tex_usage_info.b_occlusion != 1) {
    ao = u_tex_fac.occlusion_factor;
  }else{
    ao = texture(t_Metall_Rough, v_TexCoord).r * u_tex_fac.occlusion_factor;
  }

  //TODO implemetn emmessive
  vec3 N;
  if (u_tex_usage_info.b_normal != 1){
    //N = vec3(u_tex_fac.normal_factor);
    //from three-rs
    N = v_normal; //use the vertex normal
  }else {
    N = texture(t_Normal, v_TexCoord).rgb;
    //N = srgb_to_linear(N);
    N = normalize(v_TBN * ((2.0 * N - 1.0) * vec3(u_tex_fac.normal_factor, u_tex_fac.normal_factor, 1.0)));
  }

  //N = normalize(TBN * ((2.0 * N - 1.0) * u_tex_fac.normal_factor));

  //N = normalize(N * 2.0 - 1.0);
  //N = normalize(TBN * N);

  //vec3 N = normalize(v_normal);
  //vec3 N = getNormalFromMap();
  vec3 V = normalize(u_main.camera_position - v_position);

  // calculate reflectance at normal incidence; if dia-electric (like plastic) use F0
  // of 0.04 and if sit's a metal, use the albedo color as F0 (metallic workflow)
  vec3 F0 = vec3(0.04);
  F0 = mix(F0, albedo, metallic);

  // reflectance equation
  vec3 Lo = vec3(0.0);
  //Point Lights
  for(int i = 0; i < min(MAX_POINT_LIGHTS, u_light_count.points); ++i)
  {
    if (u_point_light.p_light[i].intensity == 0.0){
      continue;
    }
    Lo += calcPointLight(u_point_light.p_light[i], FragmentPosition, albedo, metallic, roughness, V, N, F0);
  }

  //Directional Lights
  for(int i = 0; i < min(MAX_DIR_LIGHTS, u_light_count.directionals); ++i){
    if (u_dir_light.d_light[i].intensity == 0.0){
      continue;
    }
    Lo += calcDirectionalLight(u_dir_light.d_light[i], FragmentPosition, albedo, metallic, roughness, V, N, F0);
  }
  //Spot Lights
  for(int i = 0; i < min(MAX_SPOT_LIGHTS, u_light_count.spots); ++i){
    if (u_spot_light.s_light[i].intensity == 0.0){
      continue;
    }
    Lo += calcSpotLight(u_spot_light.s_light[i], FragmentPosition, albedo, metallic, roughness, V, N, F0);
  }


  // ambient lighting (note that the next IBL tutorial will replace
  // this ambient lighting with environment lighting).
  vec3 ambient = vec3(0.03) * albedo * ao;

  vec3 color = ambient + Lo;

  // HDR tonemapping
  color = color / (color + vec3(1.0));
  // gamma correct
  color = pow(color, vec3(1.0/2.2));

  f_color = vec4(color, 1.0);

  //f_color = vec4(N, 1.0);
}
