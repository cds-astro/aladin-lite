uniform sampler2DArray tex;
uniform isampler2DArray texInt;
uniform float bscale;
uniform float bzero;

uniform float min_value;
uniform float max_value;
uniform int H;

uniform float size_tile_uv;
uniform float blank_value;

uniform vec3 color_fits;
uniform int fits_storing_integers;

@import ../colormaps/colormap;
@import ./transfer_funcs;

vec3 reverse_uv(vec3 uv) {
    uv.y = size_tile_uv + 2.0*size_tile_uv*floor(uv.y / size_tile_uv) - uv.y;

    return uv;
}

const vec4 transparent = vec4(0.0, 0.0, 1.0, 1.0);

vec4 color_hips_fits_from_colormap(vec3 UV) {
    float x = 0.0;
    if (fits_storing_integers == 0) {
        x = texture(tex, reverse_uv(UV)).r;
    } else {
        x = texture(texInt, reverse_uv(UV)).r;
    }

    if (x == blank_value) {
        return transparent;
    }

    float alpha = x * bscale + bzero;
    float h = transfer_func(H, alpha, min_value, max_value);
    vec4 color = vec4(colormap_f(h).rgb, 1.0);

    return color;
}

vec4 color_hips_fits_from_color(vec3 UV) {
    float x = 0.0;
    if (fits_storing_integers == 0) {
        x = texture(tex, reverse_uv(UV)).r;
    } else {
        x = texture(texInt, reverse_uv(UV)).r;
    }

    if (x == blank_value) {
        return transparent;
    }

    float alpha = x * bscale + bzero;
    float h = transfer_func(H, alpha, min_value, max_value);
    vec4 color = vec4(color_fits, h);

    return color;
}

vec4 color_hips(vec3 UV) {
    vec4 color = vec4(texture(tex, UV).rgb, 1.0);
    return color;
}