precision mediump float;
precision mediump sampler2D;
precision mediump int;

varying vec3 frag_uv_start;
varying vec3 frag_uv_end;
varying float frag_blending_factor;
varying float m_start;
varying float m_end;

@import ../color;

uniform float opacity;

uniform float first_survey;

vec4 get_color(vec3 uv, float empty) {
    vec4 c = get_colormap_from_grayscale_texture(uv);
    vec4 c1 = mix(c, blank_color, empty);
    vec4 c2 = mix(c, colormap_f(0.0), empty);

    vec4 color = mix(c1, c2, first_survey);
    return color;
}

void main() {
    vec4 color_start = get_color(frag_uv_start, m_start);
    vec4 color_end = get_color(frag_uv_end, m_end);

    vec4 out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = out_frag_color.a * opacity;

    gl_FragColor = out_frag_color;
}