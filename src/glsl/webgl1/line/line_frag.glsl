precision highp float;
varying vec4 v_rgba;

void main() {
    // Multiply vertex color with texture color (in linear space).
    // Linear color is written and blended in Framebuffer and converted to sRGB later
    gl_FragColor = v_rgba;
}