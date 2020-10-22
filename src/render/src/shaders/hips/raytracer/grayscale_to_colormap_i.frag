#version 300 es
precision highp float;
precision lowp sampler2D;
precision lowp isampler2D;
precision highp int;

in vec3 out_vert_pos;
in vec2 pos_clip;

out vec4 out_frag_color;

uniform int user_action;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    int empty;
};

uniform int current_depth;

uniform Tile textures_tiles[64];

uniform int num_textures;
uniform float opacity;
uniform float current_time; // current time in ms
struct TileColor {
    Tile tile;
    vec3 color;
    bool found;
};

@import ../color_i;
@import ./healpix;

TileColor get_tile_color(vec3 pos, int depth) {
    // Get the radec
    /*float ra = atan(pos.x, pos.z);
    float dec = atan(pos.y, length(pos.xz));
    vec2 radec = vec2(ra, dec);
    HashDxDy result = hash_with_dxdy2(depth, radec);*/

    HashDxDy result = hash_with_dxdy(depth, pos.zxy);
    int idx = result.idx;
    //int uniq = (1 << ((int(depth) + 1) << 1)) + int(idx);
    int uniq = (16 << (depth << 1)) | idx;

    vec2 uv = vec2(result.dy, result.dx);

    int a = 0;
    int b = num_textures;

    if (depth == 0) {
        b = 11;
    }

    int i = (b + a) >> 1;

    int h = int(log2(float(b))) + 1;
    // Binary search among the tile idx
    for(int step = 0; step < h; step++) {
        if (uniq == textures_tiles[i].uniq) {
            Tile tile = textures_tiles[i];
            if (tile.empty == 1) {
                return TileColor(tile, vec3(0.0), true);
            } else {
                int idx_texture = tile.texture_idx >> 6;
                int off = tile.texture_idx % 64;
                float idx_row = float(off >> 3); // in [0; 7]
                float idx_col = float(off % 8); // in [0; 7]

                vec2 offset = (vec2(idx_col, idx_row) + uv)/8.f;
                vec3 UV = vec3(offset, float(idx_texture));

                vec3 color = colormap_f(get_grayscale_from_texture(UV)).rgb;

                return TileColor(tile, color, true);
            }
        } else if (uniq < textures_tiles[i].uniq) {
            // go to left
            b = i - 1;
        } else {
            // go to right
            a = i + 1;
        }
        i = (a + b) >> 1;
    }

    // code unreachable
    Tile empty = Tile(0, -1, current_time, 1);
    return TileColor(empty, vec3(0.f), false);
}

const float duration = 500.f; // 500ms
uniform int max_depth; // max depth of the HiPS
/*
void main() {
    vec3 frag_pos = normalize(out_vert_pos);
    // Get the HEALPix cell idx and the uv in the texture

    TileColor current_tile = get_tile_color(frag_pos, current_depth);
    out_frag_color = vec4(current_tile.color, opacity);

    vec3 out_color = vec3(0.f);
    if (!current_tile.found) {
        int depth = 0;
        if (user_action == 1) {
            // zoom
            depth = max(0, current_depth - 1);
        } else {
            // unzoom
            depth = min(max_depth, current_depth + 1);
        }

        TileColor prev_tile = get_tile_color(frag_pos, depth);
        float alpha = clamp((current_time - prev_tile.tile.start_time) / duration, 0.f, 1.f);
        if (alpha == 1.f) {
            out_frag_color = vec4(prev_tile.color, opacity);
            return;
        }

        TileColor base_tile = get_tile_color(frag_pos, 0);

        out_color = mix(base_tile.color, prev_tile.color, alpha);
    } else {
        float alpha = clamp((current_time - current_tile.tile.start_time) / duration, 0.f, 1.f);
        
        // Little optimization: if the current tile is loaded since the time duration
        // then we do not need to evaluate the frag position for the previous/next depth
        if (alpha == 1.f) {
            out_frag_color = vec4(current_tile.color, opacity);
            return;
        }
        int depth = 0;
        if (user_action == 1) {
            // zoom
            depth = max(0, current_depth - 1);
        } else if (user_action == 2) {
            // unzoom
            depth = min(max_depth, current_depth + 1);
        }

        TileColor tile = get_tile_color(frag_pos, depth);
        if (!tile.found) {
            tile = get_tile_color(frag_pos, 0);
        }

        out_color = mix(tile.color, current_tile.color, alpha);
    }
    out_frag_color = vec4(out_color, opacity);
}*/
void main() {
    vec3 frag_pos = normalize(out_vert_pos);
    // Get the HEALPix cell idx and the uv in the texture

    TileColor current_tile = get_tile_color(frag_pos, current_depth);
    out_frag_color = vec4(current_tile.color, opacity);

    //if (!current_tile.found) {
        /*vec3 out_color = vec3(0.f);
        int depth = 0;
        if (user_action == 1) {
            // zoom
            depth = max(0, current_depth - 1);
        } else {
            // unzoom
            depth = min(max_depth, current_depth + 1);
        }

        TileColor prev_tile = get_tile_color(frag_pos, depth);
        float alpha = clamp((current_time - prev_tile.tile.start_time) / duration, 0.f, 1.f);
        if (alpha == 1.f) {
            out_frag_color = vec4(prev_tile.color, opacity);
            return;
        }

        TileColor base_tile = get_tile_color(frag_pos, 0);
        out_color = mix(base_tile.color, prev_tile.color, alpha);
        out_frag_color = vec4(out_color, opacity);*/
        //TileColor base_tile = get_tile_color(frag_pos, 0);
        //out_frag_color = vec4(base_tile.color, opacity);
        //out_frag_color = vec4(1.0, 0.0, 0.0, 1.0);
        //return;
    //}

    /*float alpha = clamp((current_time - current_tile.tile.start_time) / duration, 0.f, 1.f);
    
    // Little optimization: if the current tile is loaded since the time duration
    // then we do not need to evaluate the frag position for the previous/next depth
    if (alpha == 1.f) {
        out_frag_color = vec4(current_tile.color, opacity);
        return;
    }
    vec3 out_color = vec3(0.f);
    int depth = 0;
    if (user_action == 1) {
        // zoom
        depth = max(0, current_depth - 1);
    } else if (user_action == 2) {
        // unzoom
        depth = min(max_depth, current_depth + 1);
    }

    TileColor tile = get_tile_color(frag_pos, depth);
    if (!tile.found) {
        tile = get_tile_color(frag_pos, 0);
    }

    out_color = mix(tile.color, current_tile.color, alpha);
    out_frag_color = vec4(out_color, opacity);*/
}