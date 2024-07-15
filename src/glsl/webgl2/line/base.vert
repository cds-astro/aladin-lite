#version 300 es
precision highp float;
layout (location = 0) in vec2 ndc_pos;

out float l;

void main() {
  gl_Position = vec4(
        ndc_pos,
        0.0,
        1.0
    );
}