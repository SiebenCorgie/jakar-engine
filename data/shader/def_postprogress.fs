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

vec4 resolve_msaa(){
  vec4 result = vec4(0.0);
	for (int i = 0; i < u_hdr_settings.sampling_rate || i<= 16; i++)
	{
		vec4 val = subpassLoad(color_input, i);
		result += val;
	}
	// Average resolved samples
  return result / u_hdr_settings.sampling_rate;
}


void main()
{
  vec3 hdrColor = resolve_msaa().rgb;

  // Exposure tone mapping
  vec3 mapped = vec3(1.0) - exp(-hdrColor * u_hdr_settings.exposure);
  // Gamma correction
  mapped = pow(mapped, vec3(1.0 / u_hdr_settings.gamma));

  FragColor = vec4(mapped, 1.0);

}
