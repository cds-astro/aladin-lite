#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
precision highp usampler2D;
precision mediump int;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform usampler2D tex;
uniform float opacity;

@include "../hips/color.glsl"

void main() {
    uvec4 color = texture(tex, frag_uv);
    out_frag_color = apply_colormap_to_grayscale(float(color.r), float(color.a));
    out_frag_color.a = out_frag_color.a * opacity;
}