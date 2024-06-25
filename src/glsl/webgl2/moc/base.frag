#version 300 es

precision lowp float;
out vec4 color;

uniform vec4 u_color;

void main() {
    // Multiply vertex color with texture color (in linear space).
    // Linear color is written and blended in Framebuffer and converted to sRGB later
    color = u_color;
}