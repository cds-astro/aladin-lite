#version 300 es
precision highp float;
precision highp sampler3D;
precision highp isampler3D;
precision highp usampler3D;

uniform sampler3D tex;

in vec3 frag_uv;

out vec4 out_frag_color;
uniform float opacity;

#include ../hips/color.glsl;

void main() {
    vec3 uv = vec3(frag_uv.xyz);
    vec4 color = uvw2c_ra(uv);

    out_frag_color = color;
    out_frag_color.a = opacity * out_frag_color.a;
}