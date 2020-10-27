#version 300 es
precision highp float;
precision lowp sampler2D;
precision lowp isampler2D;
precision highp int;

in vec3 out_vert_pos;
in vec2 pos_clip;
in vec2 out_lonlat;

out vec4 out_frag_color;

uniform int user_action;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    int empty;
};

uniform int current_depth;

uniform Tile textures_tiles[12];

uniform float current_time; // current time in ms
struct TileColor {
    Tile tile;
    vec3 color;
    bool found;
};

@import ../color;
@import ./healpix;

uniform vec3 C;
uniform float K;

uniform float opacity;

TileColor get_tile_color(vec3 pos) {
    // Get the radec
    /*float ra = atan(pos.x, pos.z);
    float dec = atan(pos.y, length(pos.xz));
    vec2 radec = vec2(ra, dec);
    HashDxDy result = hash_with_dxdy2(depth, radec);*/

    HashDxDy result = hash_with_dxdy(0, pos.zxy);
    //HashDxDy result = hash_with_dxdy2(out_lonlat);
    int idx = result.idx;
    vec2 uv = vec2(result.dy, result.dx);

    Tile tile = textures_tiles[idx];

    if (tile.empty == 1) {
        return TileColor(tile, vec3(0.0), true);
    } else {
        int idx_texture = tile.texture_idx >> 6;
        int off = tile.texture_idx & 0x3F;
        float idx_row = float(off >> 3); // in [0; 7]
        float idx_col = float(off & 0x7); // in [0; 7]

        vec2 offset = (vec2(idx_col, idx_row) + uv)*0.125;
        vec3 UV = vec3(offset, float(idx_texture));

        vec3 color = K * C * get_grayscale_from_texture(UV);

        return TileColor(tile, color, true);
    }
}

const float duration = 500.f; // 500ms
uniform int max_depth; // max depth of the HiPS

void main() {
    vec3 frag_pos = normalize(out_vert_pos);
    // Get the HEALPix cell idx and the uv in the texture

    TileColor current_tile = get_tile_color(frag_pos);
    out_frag_color = vec4(current_tile.color, opacity);
}