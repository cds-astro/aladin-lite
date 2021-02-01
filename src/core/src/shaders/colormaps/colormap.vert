#version 300 es
precision lowp float;

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;

out vec2 out_uv;

void main() {
    gl_Position = vec4(position, 0.f, 1.f);
    out_uv = uv;
}