precision mediump float;
precision mediump sampler2D;
precision mediump int;

varying vec3 frag_uv_start;
varying vec3 frag_uv_end;
varying float frag_blending_factor;
varying float m_start;
varying float m_end;

uniform float opacity;

@import ../color;

void main() {
    vec4 color_start = get_color_from_texture(frag_uv_start);
    vec4 color_end = get_color_from_texture(frag_uv_end);

    vec4 out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = opacity * out_frag_color.a;

    gl_FragColor = out_frag_color;
}