#version 300 es
precision lowp float;
precision lowp sampler2DArray;
precision lowp isampler2DArray;
precision lowp usampler2DArray;

uniform sampler2DArray tex;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;
uniform float opacity;

#include ../color.glsl;

void main() {
    vec4 color_start = get_color_from_texture(frag_uv_start);
    vec4 color_end = get_color_from_texture(frag_uv_end);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = opacity * out_frag_color.a;
}