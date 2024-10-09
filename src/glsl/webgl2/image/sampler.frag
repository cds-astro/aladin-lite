#version 300 es
precision highp float;
precision highp sampler2D;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

void main() {
    out_frag_color = texture(tex, vec2(frag_uv.x, 1.0 - frag_uv.y));
    out_frag_color.a = out_frag_color.a * opacity;
}