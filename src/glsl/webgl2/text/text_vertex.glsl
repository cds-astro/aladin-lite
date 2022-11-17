#version 300 es
layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 tx;
layout (location = 2) in vec2 u_screen_pos;
layout (location = 3) in float rot;

out vec3 v_rgb;
out vec2 v_tc;

uniform vec2 u_screen_size;
uniform float u_scale;
uniform float u_dpi;

void main() {
  // The dpi accounts for the screen_position (given in px)
  // and also for the scale
  float st = sin(rot);
  float ct = cos(rot);
  mat2 u_rot = mat2(ct, st, -st, ct);
  vec2 p = u_rot * u_scale * u_dpi * pos;
  gl_Position = vec4(
    2.0 * (p.x + u_screen_pos.x * u_dpi) / u_screen_size.x - 1.0,
    1.0 - 2.0 * (p.y + u_screen_pos.y * u_dpi) / u_screen_size.y,
    0.0,
    1.0
  );

  v_tc = tx;
}