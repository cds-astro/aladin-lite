uniform isampler2DArray tex;
uniform float scale;
uniform float offset;
uniform float blank;

uniform float min_value;
uniform float max_value;
uniform int H;

uniform float size_tile_uv;

uniform float tex_storing_integers;
uniform int tex_storing_fits;

@import ../colormaps/colormap;
@import ./transfer_funcs;

vec3 reverse_uv(vec3 uv) {
    uv.y = size_tile_uv + 2.0*size_tile_uv*floor(uv.y / size_tile_uv) - uv.y;

    return uv;
}

float get_grayscale_from_texture(vec3 UV) {
    vec3 uv = UV;
    // FITS data pixels are reversed along the y axis
    if (tex_storing_fits == 1) {
        uv = reverse_uv(UV);
    }

    float x = float(texture(tex, uv).r);

    /*if (x == blank) {
        return transparent;
    }*/

    float alpha = x * scale + offset;
    float h = transfer_func(H, alpha, min_value, max_value);

    return h;
}