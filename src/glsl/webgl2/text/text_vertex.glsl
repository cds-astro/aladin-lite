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
uniform float u_dpi;

void main() {
  // The dpi accounts for the screen_position (given in px)
  // and also for the scale
  vec2 p = u_rot * u_scale * u_dpi * pos;
  gl_Position = vec4(
    2.0 * (p.x + u_screen_pos.x * u_dpi) / u_screen_size.x - 1.0,
    1.0 - 2.0 * (p.y + u_screen_pos.y * u_dpi) / u_screen_size.y,
    0.0,
    1.0
  );

  v_rgba = u_color;
  v_tc = tx;
}