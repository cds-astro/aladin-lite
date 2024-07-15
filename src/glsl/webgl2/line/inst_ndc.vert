#version 300 es
precision highp float;
layout (location = 0) in vec2 p_a;
layout (location = 1) in vec2 p_b;
layout (location = 2) in vec2 vertex;

out float l;

uniform float u_width;
uniform float u_height;
uniform float u_thickness;

void main() {
    vec2 x_b = p_b - p_a;
    vec2 y_b = normalize(vec2(-x_b.y, x_b.x));

    float ndc2pix = 2.0 / u_width;

    vec2 p = p_a + x_b * vertex.x + u_thickness * y_b * vertex.y * vec2(1.0, u_width/u_height) * ndc2pix;
    gl_Position = vec4(p, 0.f, 1.f);
}