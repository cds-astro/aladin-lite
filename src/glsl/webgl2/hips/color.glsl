uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;
uniform float size_tile_uv;
uniform int tex_storing_fits;

#include ../colormaps/colormap.glsl;
#include ./transfer_funcs.glsl;
#include ./tonal_corrections.glsl;
#include ./hsv.glsl;

vec4 get_pixels(vec3 uv) {
    return texture(tex, uv);
}

vec3 reverse_uv(vec3 uv) {
    uv.y = size_tile_uv + 2.0*size_tile_uv*floor(uv.y / size_tile_uv) - uv.y;
    return uv;
}

vec4 get_color_from_texture(vec3 UV) {
    vec4 color = get_pixels(UV);
    
    color.r = transfer_func(H, color.r, min_value, max_value);
    color.g = transfer_func(H, color.g, min_value, max_value);
    color.b = transfer_func(H, color.b, min_value, max_value);

    // apply reversed
    color.rgb = mix(color.rgb, 1.0 - color.rgb, reversed);

    return apply_tonal(color);
}

vec4 apply_colormap_to_grayscale(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    // apply reversed
    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

highp float decode32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

vec4 get_colormap_from_grayscale_texture(vec3 UV) {
    // FITS data pixels are reversed along the y axis
    vec3 uv = mix(UV, reverse_uv(UV), float(tex_storing_fits == 1));

    float value = decode32(get_pixels(uv).abgr*255.0);
    return apply_colormap_to_grayscale(value);
}

vec4 get_colormap_from_color_texture(vec3 uv) {
    float value = get_pixels(uv).r;
    return apply_colormap_to_grayscale(value);
}