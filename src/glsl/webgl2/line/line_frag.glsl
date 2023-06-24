#version 300 es

precision highp float;
in vec4 v_rgba;

out vec4 color;

void main() {
    // Multiply vertex color with texture color (in linear space).
    // Linear color is written and blended in Framebuffer and converted to sRGB later
    color = v_rgba;
}