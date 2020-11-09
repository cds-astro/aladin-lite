#version 300 es
precision mediump float;
precision highp sampler2D;
precision highp isampler2D;
precision mediump int;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;

@import ../color_i;

uniform vec3 C;
uniform float K;
uniform float opacity;

void main() {
    vec3 color_start = K * C * get_grayscale_from_texture(frag_uv_start);
    vec3 color_end = K * C * get_grayscale_from_texture(frag_uv_end);
    //vec4 color_end = vec4(1.0, 0.0, 0.0, 1.0);
    out_frag_color = vec4(mix(color_start, color_end, frag_blending_factor), 1.0);
    out_frag_color.a = opacity;
}