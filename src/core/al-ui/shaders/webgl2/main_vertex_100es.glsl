#version 300 es
precision highp float;

layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tx;
layout (location = 2) in vec4 color;

out vec4 v_rgba;
out vec2 v_tc;

uniform vec2 u_screen_size;

void main() {
  gl_Position = vec4(
                     2.0 * pos.x / u_screen_size.x - 1.0,
                     1.0 - 2.0 * pos.y / u_screen_size.y,
                     0.0,
                     1.0);
  v_rgba = color;
  v_tc = tx;
}