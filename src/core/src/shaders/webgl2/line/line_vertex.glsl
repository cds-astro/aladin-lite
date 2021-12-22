#version 300 es
precision highp float;
layout (location = 0) in vec2 pos;

uniform vec2 u_screen_size;
uniform vec4 u_color;

out vec4 v_rgba;

void main() {
  gl_Position = vec4(
        2.0 * (pos.x + u_screen_pos.x) / u_screen_size.x - 1.0,
        1.0 - 2.0 * (pos.y + u_screen_pos.y) / u_screen_size.y,
        0.0,
        1.0
    );

    v_rgba = u_color;
}