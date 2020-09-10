#version 300 es
precision highp float;
precision lowp sampler2DArray;
precision lowp isampler2DArray;
precision highp int;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;

@import ../color;

void main() {
    vec4 color_start = color_fits(frag_uv_start);
    vec4 color_end = color_fits(frag_uv_end);
    //vec4 color_end = vec4(1.0, 0.0, 0.0, 1.0);
    out_frag_color = mix(color_start, color_end, frag_blending_factor);
}