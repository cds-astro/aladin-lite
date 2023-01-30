#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
precision mediump int;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;

// fits values
uniform float scale;
uniform float offset;
uniform float blank;

// stretch values
uniform float min_value;
uniform float max_value;
uniform int H;

uniform float opacity;

@include "../colormaps/colormap.glsl"
@include "../hips/transfer_funcs.glsl"

void main() {
    if (frag_uv.x >= 0.0 && frag_uv.x <= 1.0 && frag_uv.y >= 0.0 && frag_uv.y <= 1.0) {
        float x = texture(tex, frag_uv).r;
        float alpha = x * scale + offset;
        alpha = transfer_func(H, alpha, min_value, max_value);

        out_frag_color = mix(
            colormap_f(alpha),
            vec4(0.0),
            float(x == blank || isnan(x))
        );

        out_frag_color.a = out_frag_color.a * opacity;
    } else {
        discard;
    }
}