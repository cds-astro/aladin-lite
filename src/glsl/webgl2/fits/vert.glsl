#version 300 es
precision highp float;
precision mediump int;

layout (location = 0) in vec2 ndc_pos;
layout (location = 1) in vec2 uv;

out vec2 frag_uv;

void main() {
    gl_Position = vec4(ndc_pos, 0.0, 1.0);
    frag_uv = uv;
}