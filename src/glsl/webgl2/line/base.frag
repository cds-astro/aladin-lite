#version 300 es

precision lowp float;
out vec4 color;
in float l;

uniform vec4 u_color;

void main() {
    // Multiply vertex color with texture color (in linear space).
    // Linear color is written and blended in Framebuffer and converted to sRGB later
    if (l > 0.025) {
        discard;
    } else {
        color = u_color;
    }
}