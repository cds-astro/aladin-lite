#version 300 es
precision highp float;
precision highp sampler2D;
precision highp usampler2D;
precision highp isampler2D;
precision mediump int;

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

    int idx_texture = tile.texture_idx >> 6;
    int off = tile.texture_idx & 0x3F;
    float idx_row = float(off >> 3); // in [0; 7]
    float idx_col = float(off & 0x7); // in [0; 7]

    vec2 offset = (vec2(idx_col, idx_row) + uv)*0.125;
    vec3 UV = vec3(offset, float(idx_texture));

    vec4 color = get_color_from_texture(UV);
    color.a *= (1.0 - tile.empty);
    return color;
}

uniform sampler2D position_tex;
uniform mat4 model;

void main() {
    // Get the HEALPix cell idx and the uv in the texture
    vec4 c = get_tile_color(normalize(frag_pos));
    out_frag_color = vec4(c.rgb, opacity * c.a);
}