uniform sampler2DArray tex;
uniform isampler2DArray texInt;
uniform float bscale;
uniform float bzero;

uniform float min_value;
uniform float max_value;
uniform int H;

uniform float size_tile_uv;
uniform float blank_value;

@import ../colormaps/colormap;
@import ./transfer_funcs;

vec3 reverse_uv(vec3 uv) {
    uv.y = size_tile_uv + 2.0*size_tile_uv*floor(uv.y / size_tile_uv) - uv.y;

    return uv;
}

const vec4 transparent = vec4(0.0, 0.0, 1.0, 1.0);

vec4 color_fits(vec3 UV) {
    float x = texture(tex, reverse_uv(UV)).r;

    if (x == blank_value) {
        return transparent;
    }

    float alpha = x * bscale + bzero;
    float h = transfer_func(H, alpha, min_value, max_value);
    vec4 color = vec4(colormap_f(h).rgb, 1.0);

    return color;
}

vec4 color_fits_int(vec3 UV) {
    int x = texture(texInt, reverse_uv(UV)).r;

    if (x == int(blank_value)) {
        return transparent;
    }

    float alpha = float(x) * bscale + bzero;
    float h = transfer_func(H, alpha, min_value, max_value);
    vec4 color = vec4(colormap_f(h).rgb, 1.0);

    return color;
}

vec4 color(vec3 UV) {
    vec4 color = vec4(texture(tex, UV).rgb, 1.0);
    return color;
}