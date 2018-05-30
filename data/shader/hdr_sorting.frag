#version 450

//tries to get the input attachment
layout(input_attachment_index = 0, set = 0, binding = 0) uniform subpassInputMS color_input;
//The inputs for the hdr -> ldr pass
layout(set = 0, binding = 1) uniform hdr_settings{
  uint sampling_rate;
  float bloom_brightness;
}u_sorting_settings;


//Get the uvs
layout(location = 0) in vec2 inter_coord;
layout(location = 1) in vec2 v_pos;


//outputs the fragment color
layout(location = 0) out vec4 LdrColor;
layout(location = 1) out vec4 HdrColor;

vec4 resolve_msaa(){
  vec4 result = vec4(0.0);
	for (int i = 0; i < u_sorting_settings.sampling_rate || i<= 16; i++)
	{
		vec4 val = subpassLoad(color_input, i);
		result += val;
	}
	// Average resolved samples
  return result / u_sorting_settings.sampling_rate;
}

void main()
{

  vec4 resolved_color = resolve_msaa();

  //make greyscale and have a look at the brighness
  float brightness = dot(resolved_color.rgb, vec3(0.2126, 0.7152, 0.0722));

  if(brightness > 1.0 ){
    //calmp to have a nice bloom
    vec3 hdr_col = mix(vec3(0.0), resolved_color.rgb, u_sorting_settings.bloom_brightness);
    HdrColor = vec4(hdr_col, 1.0);
  }else{
    HdrColor = vec4(vec3(0.0), 0.0);
  }

  LdrColor = resolved_color;


}
