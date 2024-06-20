#version 300 es
precision lowp float;
layout (location = 0) in vec2 p_a;
layout (location = 1) in vec2 p_b;
layout (location = 2) in vec2 vertex;

out float l;

uniform float u_width;

void main() {
    vec2 x_b = p_b - p_a;
    vec2 y_b = normalize(vec2(-x_b.y, x_b.x));

    vec2 p = p_a + x_b * vertex.x + y_b * u_width * 0.001 * vertex.y;
    gl_Position = vec4(p, 0.f, 1.f);
}