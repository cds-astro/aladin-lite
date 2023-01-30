#version 300 es
precision lowp float;

in vec2 out_uv;
in vec3 out_p;

out vec4 color;

uniform sampler2D kernel_texture;
uniform float max_density; // max number of sources in a kernel sized HEALPix cell at the current depth
uniform float fov;
uniform float strength;
void main() {
    color = texture(kernel_texture, out_uv) / max(log2(fov*100.0), 1.0);
    color.r *= strength;
}