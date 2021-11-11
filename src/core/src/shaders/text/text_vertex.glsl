#version 300 es
precision highp float;
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tx;

uniform vec2 u_screen_size;
uniform vec4 u_color;
uniform vec2 u_screen_pos;
uniform mat2 u_rot;

out vec4 v_rgba;
out vec2 v_tc;

void main() {
  vec2 p = u_rot * pos;
  gl_Position = vec4(
        2.0 * (p.x + u_screen_pos.x) / u_screen_size.x - 1.0,
        1.0 - 2.0 * (p.y + u_screen_pos.y) / u_screen_size.y,
        0.0,
        1.0
    );

    v_rgba = u_color;
    v_tc = tx;
}