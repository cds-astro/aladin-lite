//const int MAX_NUM_TEX = 3;
uniform sampler2D tex1;
uniform sampler2D tex2;
uniform sampler2D tex3;

uniform int num_tex;

uniform float scale;
uniform float offset;
uniform float blank;

uniform float min_value;
uniform float max_value;
uniform int H;

uniform float size_tile_uv;

uniform int tex_storing_fits;

@include "../colormaps/colormap.glsl"
@include "./transfer_funcs.glsl"
@include "./tonal_corrections.glsl"
@include "./hsv.glsl"

vec4 get_pixels(vec3 uv) {
    int idx_texture = int(uv.z);
    if (idx_texture == 0) {
        return texture(tex1, uv.xy);
    } else if (idx_texture == 1) {
        return texture(tex2, uv.xy);
    } else if (idx_texture == 2) {
        return texture(tex3, uv.xy);
    } else {
        return vec4(0.0, 1.0, 0.0, 1.0);
    }
}

vec3 reverse_uv(vec3 uv) {
    uv.y = size_tile_uv + 2.0*size_tile_uv*floor(uv.y / size_tile_uv) - uv.y;
    return uv;
}

uniform float reversed;

vec4 get_color_from_texture(vec3 UV) {
    vec4 color = get_pixels(UV);
    
    color.r = transfer_func(H, color.r, min_value, max_value);
    color.g = transfer_func(H, color.g, min_value, max_value);
    color.b = transfer_func(H, color.b, min_value, max_value);

    // apply reversed
    color.rgb = mix(color.rgb, 1.0 - color.rgb, reversed);

    return apply_tonal(color);
}

vec4 apply_colormap_to_grayscale(float x, float a) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    // apply reversed
    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha) * a, vec4(0.0), float(x == blank || isnan(x)));
    return apply_tonal(new_color);
}

vec4 get_colormap_from_grayscale_texture(vec3 UV) {
    // FITS data pixels are reversed along the y axis
    vec3 uv = mix(UV, reverse_uv(UV), float(tex_storing_fits == 1));

    vec4 color = get_pixels(uv);
    return apply_colormap_to_grayscale(color.r, color.a);
}