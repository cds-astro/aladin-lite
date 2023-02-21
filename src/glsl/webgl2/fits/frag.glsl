#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
precision mediump int;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

@include "../hips/color.glsl"

void main() {
    if (frag_uv.x >= 0.0 && frag_uv.x <= 1.0 && frag_uv.y >= 0.0 && frag_uv.y <= 1.0) {
        vec4 color = texture(tex, frag_uv);
        out_frag_color = apply_colormap_to_grayscale(color.r, color.a);
        out_frag_color.a = out_frag_color.a * opacity;
    } else {
        discard;
    }
}