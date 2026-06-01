#version 300 es
precision highp float;
precision highp sampler2D;
precision highp int;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

#include ./color.glsl;

void main() {
    // FITS y axis looks down
    vec2 uv = frag_uv;
    uv.y = 1.0 - uv.y;

    out_frag_color = uv2c_i16(frag_uv);
    out_frag_color.a = out_frag_color.a * opacity;
}