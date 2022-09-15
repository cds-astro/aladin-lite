#version 300 es
precision highp float;
precision highp sampler2D;
precision highp usampler2D;
precision highp isampler2D;
precision highp int;

out vec4 out_frag_color;

uniform vec3 font_color;
uniform float opacity;

void main() {
    out_frag_color = vec4(font_color, opacity);
}