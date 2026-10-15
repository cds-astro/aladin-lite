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
#include ../hsv.glsl;
#include ../decode.glsl;

/////////////////////////////////////////////
/// RED sampler
vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    // apply reversed
    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

/// RGBA sampler
vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    // apply reversed
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    // apply reversed
    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    // apply transfer f
    v = transfer_func(H, v, min_value, max_value);
    // apply cmap
    vec4 c = colormap_f(v);
    // apply reversed
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

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

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}
