precision highp float;
attribute vec2 pos;

uniform vec2 u_screen_size;
uniform vec4 u_color;

varying vec4 v_rgba;

void main() {
  gl_Position = vec4(
      pos,
      0.0,
      1.0
  );

   v_rgba = u_color;
}