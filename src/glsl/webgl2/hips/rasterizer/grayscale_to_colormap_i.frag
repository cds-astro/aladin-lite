#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
precision mediump int;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;
in float m_start;
in float m_end;

out vec4 out_frag_color;

#include ../color_i.glsl;

uniform float opacity;

void main() {
    vec4 color_start = get_colormap_from_grayscale_texture(frag_uv_start);
    vec4 color_end = get_colormap_from_grayscale_texture(frag_uv_end);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = out_frag_color.a * opacity;
}