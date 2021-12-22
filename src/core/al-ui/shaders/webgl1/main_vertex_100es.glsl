precision highp float;

attribute vec2 pos;
attribute vec2 tx;
attribute vec4 color;

varying vec4 v_rgba;
varying vec2 v_tc;

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