#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray tex;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;

#include ../color.glsl;

uniform float opacity;

void main() {
    // FITS data pixels are reversed along the y axis
    vec3 uv0 = frag_uv_start;
    vec3 uv1 = frag_uv_end;
    uv0.y = 1.0 - uv0.y;
    uv1.y = 1.0 - uv1.y;

    vec4 color_start = uvw2c_u8(uv0);
    vec4 color_end = uvw2c_u8(uv1);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = out_frag_color.a * opacity;
}