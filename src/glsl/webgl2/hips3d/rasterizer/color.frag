#version 300 es
precision lowp float;
precision lowp sampler3D;
precision lowp isampler3D;
precision lowp usampler3D;

uniform sampler3D tex;

in vec3 frag_uv;

out vec4 out_frag_color;
uniform float opacity;

#include ../../hips/color.glsl;

void main() {
    vec4 color = get_color_from_texture(vec3(frag_uv.xy, mod(frag_uv.z, 32.0) / 32.0));

    out_frag_color = color;
    out_frag_color.a = opacity * out_frag_color.a;
}