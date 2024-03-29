precision highp float;
precision highp sampler2D;
precision highp int;

varying vec3 out_vert_pos;
varying vec2 out_clip_pos;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[192];
uniform int num_tiles;

uniform float current_time; // current time in ms

@import ../color;
@import ./healpix;

uniform float opacity;

Tile get_tile(int idx) {
    for(int i = 0; i < 12; i++) {
        if( i == idx ) {
            return textures_tiles[i];
        }
    }
}

Tile binary_search_tile(int uniq) {
    int l = 0;
    int r = 11;
    for (int v = 0; v <= 5; v++) {
        int mid = (l + r) / 2;

        Tile tile = get_tile(mid);
        if(tile.uniq == uniq) {
            return tile;
        } else if(tile.uniq < uniq) {
            l = mid + 1;
        } else {
            r = mid - 1;
        }

        // before exiting the loop
        if (l >= r) {
            return get_tile(l);
        }
    }
}

vec4 get_tile_color(vec3 pos) {
    float delta = asin(pos.y);
    float theta = atan(pos.x, pos.z);
    HashDxDy result = hash_with_dxdy(vec2(theta, delta));

    int idx = result.idx;
    vec2 uv = vec2(result.dy, result.dx);
    //return vec4(uv, 1.0, 1.0);

    int uniq = 16 + idx; 
    Tile tile = binary_search_tile(uniq);

    int idx_texture = tile.texture_idx / 64;
    int off = tile.texture_idx - idx_texture * 64;

    int idx_row = off / 8; // in [0; 7]
    int idx_col = off - idx_row * 8; // in [0; 7]

    vec2 offset = (vec2(idx_col, idx_row) + uv) * 0.125;
    vec3 UV = vec3(offset.x, offset.y, 0.0);
    
    vec4 color = get_color_from_texture(UV);
    color.a *= (1.0 - float(tile.empty));
    return color;
}

uniform sampler2D position_tex;
uniform mat4 model;
void main() {
    vec2 uv = out_clip_pos * 0.5 + 0.5;
    vec3 n = texture2D(position_tex, uv).rgb;
    /* Taylor DL
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
    */

    vec3 frag_pos = vec3(model * vec4(n, 1.0));

    // Get the HEALPix cell idx and the uv in the texture
    vec4 c = get_tile_color(frag_pos);
    gl_FragColor = vec4(c.rgb, c.a * opacity);
}