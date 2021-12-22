#version 300 es
in vec3 out_vert_pos;
in vec2 out_clip_pos;
out vec4 out_frag_color;

precision highp float;
precision highp sampler2D;
precision highp usampler2D;
precision highp isampler2D;
precision highp int;

uniform int user_action;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform int current_depth;
uniform Tile textures_tiles[192];
uniform int num_tiles;

uniform float current_time; // current time in ms

@import ../color;
@import ./healpix;

uniform float opacity;

int binary_search_tile(int uniq) {
    int l = 0;
    int r = num_tiles - 1;

    while (l < r) {
        int a = (l + r) / 2;
        Tile tile = textures_tiles[a];
        if(tile.uniq == uniq) {
            return a;
        } else if(tile.uniq < uniq) {
            l = a + 1;
        } else {
            r = a - 1;
        }
    }

    return l;
}

vec4 get_tile_color(vec3 pos) {
    int d = current_depth;

    while (d >= 0) {
        HashDxDy result = hash_with_dxdy(d, pos.zxy);
        
        int idx = result.idx;
        vec2 uv = vec2(result.dy, result.dx);

        int uniq = (16 << (d << 1)) | idx;
        int tile_idx = binary_search_tile(uniq);
        Tile tile = textures_tiles[tile_idx];

        if(tile.uniq == uniq) {
            int idx_texture = tile.texture_idx >> 6;
            int off = tile.texture_idx & 0x3F;
            float idx_row = float(off >> 3); // in [0; 7]
            float idx_col = float(off & 0x7); // in [0; 7]

            vec2 offset = (vec2(idx_col, idx_row) + uv)*0.125;
            vec3 UV = vec3(offset, float(idx_texture));

            vec4 color = get_color_from_texture(UV);
            color.a = mix(color.a, blank_color.a, tile.empty);
            
            return color;
        }

        d = d - 1;
    }
}

const float duration = 500.f; // 500ms
uniform int max_depth; // max depth of the HiPS

uniform sampler2D position_tex;
uniform mat4 model;
void main() {
    vec3 n = vec3(0.0);
    if(current_depth < 2) {
        vec2 uv = out_clip_pos * 0.5 + 0.5;
        n = texture(position_tex, uv).rgb;
    } else {
        float x = out_clip_pos.x;
        float y = out_clip_pos.y;
        float x2 = x*x;
        float y2 = y*y;
        float x4 = x2*x2;
        float y4 = y2*y2;

        n = vec3(
            -x,
            y,
            -0.5*x2 - 0.5*y2 + 1.0
        );
    }

    vec3 frag_pos = vec3(model * vec4(n, 1.0));

    // Get the HEALPix cell idx and the uv in the texture
    vec4 c = get_tile_color(frag_pos);
    out_frag_color = vec4(c.rgb, opacity * c.a);
}