#version 300 es
precision lowp float;

in vec2 out_uv;

out vec4 color;

uniform sampler2D kernel_texture;
uniform float max_density;

void main() {
    color += texture(kernel_texture, out_uv) / log2(max_density + 1.0);

}