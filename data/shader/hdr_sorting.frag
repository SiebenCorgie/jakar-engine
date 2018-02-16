#version 450

//tries to get the input attachment
layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInputMS color_input;
//The inputs for the hdr -> ldr pass
layout(set = 1, binding = 0) uniform hdr_settings{
  float exposure;
  float gamma;
  float near;
  float far;
  int sampling_rate;
  int show_mode;
}u_hdr_settings;


//Get the uvs
layout(location = 0) in vec2 inter_coord;
layout(location = 1) in vec2 v_pos;


//outputs the fragment color
layout(location = 0) out vec4 HDRColor;

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

  vec4 hdrColor = resolve_msaa();

  if(hdrColor.x > 1.0 ||hdrColor.y > 1.0 ||hdrColor.z > 1.0 ){
    HDRColor = hdrColor;
  }else{
    HDRColor = vec4(vec3(0.0), 1.0);
  }


}
