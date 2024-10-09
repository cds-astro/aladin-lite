#version 300 es
precision lowp float;
precision lowp sampler2DArray;
precision lowp usampler2DArray;
precision lowp isampler2DArray;
precision mediump int;

uniform sampler2DArray tex;

in vec2 out_clip_pos;
in vec3 frag_pos;
out vec4 out_frag_color;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[12];

#include ../color.glsl;
#include ../../projection/hpx.glsl;

uniform float opacity;

vec4 get_tile_color(vec3 pos) {
    HashDxDy result = hash_with_dxdy(0, pos.zxy);

    int idx = result.idx;
    vec2 uv = vec2(result.dy, result.dx);
    Tile tile = textures_tiles[idx];

    vec2 offset = uv;
    vec3 UV = vec3(offset, float(tile.texture_idx));

    vec4 color = get_color_from_texture(UV);
    color.a *= (1.0 - tile.empty);
    return color;
}

void main() {
    // Get the HEALPix cell idx and the uv in the texture
    vec4 c = get_tile_color(normalize(frag_pos));
    out_frag_color = vec4(c.rgb, opacity * c.a);
}