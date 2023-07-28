#version 300 es
precision lowp float;
layout (location = 0) in vec2 ndc_pos;

uniform vec4 u_color;

out vec4 v_rgba;

void main() {
  gl_Position = vec4(
        ndc_pos,
        0.0,
        1.0
    );

    v_rgba = u_color;
}