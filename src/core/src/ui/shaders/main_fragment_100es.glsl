#version 300 es

precision highp float;
uniform sampler2D u_sampler;
in vec4 v_rgba;
in vec2 v_tc;
out vec4 color;
void main() {
  // The texture is set up with `SRGB8_ALPHA8`, so no need to decode here!
  vec4 texture_rgba = texture(u_sampler, v_tc).rgba;

  // Multiply vertex color with texture color (in linear space).
  // Linear color is written and blended in Framebuffer and converted to sRGB later
  color = v_rgba * texture_rgba;
  //color = vec4(1.0, 0.0, 0., 1.);
  //color = v_rgba;
}