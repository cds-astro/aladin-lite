#version 300 es
precision lowp float;
precision lowp sampler3D;
precision lowp isampler3D;
precision lowp usampler3D;

uniform isampler3D tex;

in vec3 frag_uv;

out vec4 out_frag_color;

#include ../../hips/color_i.glsl;

uniform float opacity;

void main() {
    vec4 color = get_colormap_from_grayscale_texture(frag_uv);

    out_frag_color = color;
    out_frag_color.a = out_frag_color.a * opacity;
}