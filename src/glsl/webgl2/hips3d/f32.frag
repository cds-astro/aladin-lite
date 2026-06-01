#version 300 es
precision highp float;
precision highp sampler3D;

uniform sampler3D tex;

in vec3 frag_uv;

out vec4 out_frag_color;

#include ../hips/color.glsl;

uniform float opacity;

void main() {
    vec3 uv = vec3(frag_uv.xyz);
    uv.y = 1.0 - uv.y;

    vec4 color = uvw2c_f32(uv);

    out_frag_color = color;
    out_frag_color.a = out_frag_color.a * opacity;
}