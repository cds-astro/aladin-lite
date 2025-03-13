#version 300 es
precision highp float;
precision highp sampler2D;
precision lowp isampler2D;
precision lowp usampler2D;
precision highp int;

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

vec4 apply_colormap_to_grayscale(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    // apply reversed
    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x)));
    return apply_tonal(new_color);
}

highp float decode32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

void main() {
    highp float value = decode32(texture(tex, frag_uv).abgr*255.0);
    // reconstruct the float value
    out_frag_color = apply_colormap_to_grayscale(value);
    out_frag_color.a = out_frag_color.a * opacity;
}