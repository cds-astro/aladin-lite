#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray tex;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;
uniform float opacity;

#include ../color.glsl;

void main() {
    vec4 color_start = uvw2cmap_rgba(frag_uv_start);
    vec4 color_end = uvw2cmap_rgba(frag_uv_end);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = opacity * out_frag_color.a;
}