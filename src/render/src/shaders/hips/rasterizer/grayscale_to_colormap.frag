#version 300 es
precision mediump float;
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
    vec4 color_start = vec4(colormap_f(get_grayscale_from_texture(frag_uv_start, m_start)).rgb, 1.0);
    vec4 color_end = vec4(colormap_f(get_grayscale_from_texture(frag_uv_end, m_end)).rgb, 1.0);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = opacity;
}