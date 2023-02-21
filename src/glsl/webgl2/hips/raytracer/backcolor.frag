#version 300 es
precision highp float;
precision highp sampler2D;
precision highp usampler2D;
precision highp isampler2D;
precision mediump int;

out vec4 out_frag_color;

uniform vec3 color;

void main() {
    out_frag_color = vec4(color, 1.0);
}