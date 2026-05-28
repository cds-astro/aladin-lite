#version 300 es
precision highp float;
precision highp sampler2DArray;
precision highp sampler2DArray;
precision highp isampler2DArray;
precision highp int;

in vec3 frag_pos;
in vec2 out_clip_pos;
out vec4 out_frag_color;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[12];

uniform float opacity;
uniform sampler2DArray tex;

#include ../color.glsl;
#include ../../projection/hpx_proj.glsl;
#include ./utils.glsl;

void main() {
    vec3 uv = xyz2uv(normalize(frag_pos));
    vec4 c = uvw2cmap_rgba(uv);

    out_frag_color = c;
    out_frag_color.a = out_frag_color.a * opacity;
}