attribute vec2 pos;
attribute vec2 tx;

varying vec4 v_rgba;
varying vec2 v_tc;

uniform vec2 u_screen_size;
uniform vec4 u_color;
uniform vec2 u_screen_pos;
uniform mat2 u_rot;
uniform float u_scale;
uniform float u_dpi;

void main() {
  vec2 p = u_rot * u_scale * u_dpi * pos;
  vec2 ndc_pos = vec2(
    2.0 * (p.x + u_screen_pos.x * u_dpi) / u_screen_size.x - 1.0,
    1.0 - 2.0 * (p.y + u_screen_pos.y * u_dpi) / u_screen_size.y
  );
  gl_Position = vec4(ndc_pos, 0.0, 1.0);

  v_rgba = u_color;
  v_tc = tx;
}