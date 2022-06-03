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

@import ../color;

uniform float opacity;

void main() {
    //vec4 color_start = get_color(frag_uv_start, m_start);
    //vec4 color_end = get_color(frag_uv_end, m_end);
    vec4 color_start = get_colormap_from_grayscale_texture(frag_uv_start);
    //color_start.a *= (1.0 - m_start);
    vec4 color_end = get_colormap_from_grayscale_texture(frag_uv_end);
    //color_end.a *= (1.0 - m_end);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = out_frag_color.a * opacity;
}