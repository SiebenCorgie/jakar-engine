#version 450




//tries to get the input attachment
layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInputMS color_input;
layout(input_attachment_index = 0, set = 0, binding = 1) uniform subpassInputMS depths_input;

//Get the uvs
layout(location = 0) in vec2 inter_coord;
layout(location = 1) in vec2 v_pos;


//outputs the fragment color
layout(location = 0) out vec4 FragColor;

//The inputs for the hdr -> ldr pass
layout(set = 1, binding = 0) uniform hdr_settings{
  float exposure;
  float gamma;
  float near;
  float far;
  int sampling_rate;
  int show_mode;

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

float linear_depth(float depth){
  float f= u_hdr_settings.far;
  float n = u_hdr_settings.near;
  float z = (2 * n) / (f + n - depth * (f - n));
  return z;
}

void main()
{


  //Cluster Id
  if (u_hdr_settings.show_mode == 0) {

    float depth_out = subpassLoad(depths_input, 1).x;

    float z = linear_depth(depth_out);

    FragColor = vec4(vec3(z), 1.0);
    return;
  }

  //Heat map
  if (u_hdr_settings.show_mode == 1) {

    FragColor = vec4(vec3(1.0, 0.0, 0.0), 1.0);
    return;
  }

  if (u_hdr_settings.show_mode == 2) {
    //only use the first sample, debugging should not be too heavy
    float depth_out = subpassLoad(depths_input, 1).x;

    float z = linear_depth(depth_out);
    FragColor = vec4(z, z, z, 1.0);
    return;
  }


  vec3 hdrColor = resolve_msaa().rgb;

  // Exposure tone mapping
  vec3 mapped = vec3(1.0) - exp(-hdrColor * u_hdr_settings.exposure);
  // Gamma correction
  mapped = pow(mapped, vec3(1.0 / u_hdr_settings.gamma));



  FragColor = vec4(mapped, 1.0);

}
