precision mediump float;
precision mediump sampler2D;
precision mediump int;

varying vec2 out_clip_pos;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[12];

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
    for (int v = 0; v >= 0; v++) {
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
    return vec4(float(result.idx)/12., result.dx, result.dy, 1.0);
    int idx = result.idx;
    vec2 uv = vec2(result.dy, result.dx);
    int uniq = 16 + idx; 
    Tile tile = binary_search_tile(uniq);

    if(tile.uniq == uniq) {
        int idx_texture = tile.texture_idx / 64;
        int off = int(mod(float(tile.texture_idx), 64.0));
        float idx_row = float(off / 8); // in [0; 7]
        float idx_col = mod(float(off), 8.0); // in [0; 7]

        vec2 offset = (vec2(idx_col, idx_row) + uv)*0.125;
        vec3 UV = vec3(offset, float(idx_texture));

        vec4 color = mix(get_color_from_grayscale_texture(UV), blank_color, tile.empty);
        return color;
    }
}

uniform sampler2D position_tex;
uniform mat4 model;
void main() {
    vec2 uv = out_clip_pos * 0.5 + 0.5;
    vec3 n = texture2D(position_tex, uv).rgb;

    vec3 frag_pos = vec3(model * vec4(n, 1.0));

    vec4 c = get_tile_color(frag_pos);
    c.a = c.a * opacity;
    gl_FragColor = c;
}