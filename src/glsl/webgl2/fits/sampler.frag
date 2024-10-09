#version 300 es
precision lowp float;
precision lowp sampler2D;
precision lowp isampler2D;
precision lowp usampler2D;
precision mediump int;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;

uniform float min_value;
uniform float max_value;
uniform int H;

uniform float reversed;

#include ./../colormaps/colormap.glsl;
#include ./../hips/transfer_funcs.glsl;
#include ./../hips/tonal_corrections.glsl;

vec4 apply_colormap_to_grayscale(float x, float a) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    // apply reversed
    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha) * a, vec4(0.0), float(x == blank || isnan(x)));
    return apply_tonal(new_color);
}

void main() {
    vec4 color = texture(tex, frag_uv);
    out_frag_color = apply_colormap_to_grayscale(color.r, color.a);
    out_frag_color.a = out_frag_color.a * opacity;
}