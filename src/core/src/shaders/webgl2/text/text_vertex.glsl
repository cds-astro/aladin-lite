#version 300 es
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tx;

out vec4 v_rgba;
out vec2 v_tc;

uniform vec2 u_screen_size;
uniform vec4 u_color;
uniform vec2 u_screen_pos;
uniform mat2 u_rot;
uniform float u_scale;

void main() {
  vec2 p = u_rot * u_scale * pos;
  gl_Position = vec4(
        p,
        0.0,
        1.0
    );

    v_rgba = u_color;
    v_tc = tx;
}