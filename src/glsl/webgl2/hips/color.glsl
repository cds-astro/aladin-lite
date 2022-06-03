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

@import ../colormaps/colormap;
@import ./transfer_funcs;

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

vec4 get_color_from_texture(vec3 UV) {
    return get_pixels(UV);
}

vec4 get_colormap_from_grayscale_texture(vec3 UV) {
    // FITS data pixels are reversed along the y axis
    vec3 uv = mix(UV, reverse_uv(UV), float(tex_storing_fits == 1));

    float x = get_pixels(uv).r;
    //if (x == blank) {
    //    return blank_color;
    //} else {
        float alpha = x * scale + offset;
        alpha = transfer_func(H, alpha, min_value, max_value);

        return mix(colormap_f(alpha), vec4(0.0), float(x == blank || isnan(x)));
    //}
}

uniform vec4 C;
uniform float K;
vec4 get_color_from_grayscale_texture(vec3 UV) {
    // FITS data pixels are reversed along the y axis
    vec3 uv = mix(UV, reverse_uv(UV), float(tex_storing_fits == 1));

    float x = get_pixels(uv).r;
    //if (x == blank) {
    //    return blank_color;
    //} else {
        float alpha = x * scale + offset;
        alpha = transfer_func(H, alpha, min_value, max_value);

        return mix(vec4(C.rgb * K * alpha, C.a), vec4(0.0), float(x == blank || isnan(x)));
    //}
}