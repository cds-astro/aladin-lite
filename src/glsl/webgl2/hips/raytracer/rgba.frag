#version 300 es
precision highp float;
precision highp sampler2DArray;
precision highp int;

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
#include ../../projection/hpx_proj.glsl;
#include ./utils.glsl;

uniform float opacity;
uniform vec4 no_tile_color;

void main() {
    vec3 uv = xyz2uv(normalize(frag_pos));
    vec4 c = uvw2c_rgba(uv);

    //c = mix(c, no_tile_color, tile.empty);
    out_frag_color = c;
    out_frag_color = vec4(c.rgb, opacity * c.a);
}