#version 300 es
precision lowp float;
precision lowp sampler2D;

in vec2 out_uv;
out vec4 color;

uniform sampler2D texture_fbo;
uniform float alpha;

#include ./colormap.glsl;

void main() {
    float opacity = texture(texture_fbo, out_uv).r;

    float o = smoothstep(0.0, 0.1, opacity);

    color = colormap_f(opacity);
    color.a = o * alpha;
}