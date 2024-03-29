#version 300 es
precision lowp float;

out vec4 frag_color;

uniform vec3 color;
uniform float opacity;

const float PI = 3.141592653589793f;

void main() {
    frag_color = vec4(color, opacity);
}