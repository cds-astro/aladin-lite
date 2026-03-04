uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

#include ../colormaps/colormap.glsl;
#include ../transfer_funcs.glsl;
#include ../tonal_corrections.glsl;
#include ../decode.glsl;

/////////////////////////////////////////////
/// FITS sampler

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    // apply reversed
    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    // apply reversed
    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uv2c_f32(vec2 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uv2c_i32(vec2 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return val2c(val);
}

vec4 uv2c_i16(vec2 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return val2c(val);
}

vec4 uv2c_u8(vec2 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}
