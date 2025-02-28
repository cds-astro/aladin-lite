#version 300 es

precision highp float;

out vec4 color;
in vec2 l;

uniform vec4 u_color;
uniform float u_thickness;
uniform float u_width;
uniform float u_height;

void main() {
    if (l.x > 0.05) {
        discard;
    } else {
        color = u_color;

        // distance from line to compute the anti-aliasing
        float dist = abs(u_thickness * l.y);
        color.a = color.a * (1.0 - smoothstep(u_thickness*0.5 - 1.0, u_thickness*0.5, dist));
    }
}