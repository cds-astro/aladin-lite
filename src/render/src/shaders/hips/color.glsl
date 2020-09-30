uniform sampler2DArray tex;
uniform isampler2DArray texInt;
uniform float scale;
uniform float offset;
uniform float blank;

uniform float min_value;
uniform float max_value;
uniform int H;

uniform float size_tile_uv;

uniform int tex_storing_integers;

@import ../colormaps/colormap;
@import ./transfer_funcs;

vec3 reverse_uv(vec3 uv) {
    uv.y = size_tile_uv + 2.0*size_tile_uv*floor(uv.y / size_tile_uv) - uv.y;

    return uv;
}

const vec4 transparent = vec4(0.0, 1.0, 0.0, 1.0);

float get_grayscale_from_texture(vec3 UV) {
    float x = 0.0;
    if (tex_storing_integers == 0) {
        x = texture(tex, reverse_uv(UV)).r;
    } else {
        x = texture(texInt, reverse_uv(UV)).r;
    }

    if (x == blank) {
        return transparent;
    }

    float alpha = x * scale + offset;
    float h = transfer_func(H, alpha, min_value, max_value);

    return h;
}

vec4 get_color_from_texture(vec3 UV) {
    if (tex_storing_integers == 0) {
        return vec4(texture(tex, UV).rgb, 1.0);
    } else {
        return vec4(texture(texInt, UV).rgb, 1.0);
    }
}