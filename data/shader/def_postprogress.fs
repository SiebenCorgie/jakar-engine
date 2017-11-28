#version 450




//tries to get the input attachment
layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInputMS color_input;
layout(input_attachment_index = 0, set = 0, binding = 1) uniform subpassInputMS depths_input;

//outputs the fragment color
layout(location = 0) out vec4 FragColor;

//The inputs for the hdr -> ldr pass
layout(set = 1, binding = 0) uniform hdr_settings{
  float exposure;
  float gamma;
  int sampling_rate;

}u_hdr_settings;


void main()
{
  vec3 hdrColor = subpassLoad(color_input, u_hdr_settings.sampling_rate).rgb;

  // Exposure tone mapping
  vec3 mapped = vec3(1.0) - exp(-hdrColor * u_hdr_settings.exposure);
  // Gamma correction
  mapped = pow(mapped, vec3(1.0 / u_hdr_settings.gamma));

  FragColor = vec4(mapped, 1.0);

}
