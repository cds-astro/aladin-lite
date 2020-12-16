const int MAX_NUM_TEX = 10;
uniform isampler2D tex[MAX_NUM_TEX];
uniform int num_tex;

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

float get_pixels(vec3 uv) {
    int idx_texture = int(uv.z);
    if (idx_texture == 0) {
        return float(texture(tex[0], uv.xy).r);
    } else if (idx_texture == 1) {
        return float(texture(tex[1], uv.xy).r);
    } else if (idx_texture == 2) {
        return float(texture(tex[2], uv.xy).r);
    } else {
        return 0.1;
    }
}

vec3 reverse_uv(vec3 uv) {
    uv.y = size_tile_uv + 2.0*size_tile_uv*floor(uv.y / size_tile_uv) - uv.y;

    return uv;
}

float get_grayscale_from_texture(vec3 UV, float m) {
    if (m == 1.0) {
        return 0.0;
    }

    vec3 uv = UV;
    // FITS data pixels are reversed along the y axis
    if (tex_storing_fits == 1) {
        uv = reverse_uv(UV);
    }

    float x = get_pixels(uv);

    float alpha = x * scale + offset;
    float h = transfer_func(H, alpha, min_value, max_value);

    return h;
}