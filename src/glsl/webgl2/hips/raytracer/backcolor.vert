#version 300 es
precision lowp float;
precision mediump int;

layout (location = 0) in vec2 pos_clip_space;

uniform vec2 ndc_to_clip;
uniform float czf;

void main() {
    gl_Position = vec4(pos_clip_space / (ndc_to_clip * czf), 0.0, 1.0);
}