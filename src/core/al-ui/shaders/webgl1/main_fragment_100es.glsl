precision highp float;
varying vec4 v_rgba;
varying vec2 v_tc;

uniform sampler2D u_sampler;

void main() {
  // The texture is set up with `SRGB8_ALPHA8`, so no need to decode here!
  vec4 texture_rgba = texture2D(u_sampler, v_tc);

  // Multiply vertex color with texture color (in linear space).
  // Linear color is written and blended in Framebuffer and converted to sRGB later
  gl_FragColor = v_rgba * texture_rgba;
  //color = vec4(1.0, 0.0, 0., 1.);
  //color = v_rgba;
}