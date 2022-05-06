#version 300 es
precision lowp float;
precision lowp sampler2DArray;

layout (location = 0) in vec2 pos;
layout (location = 1) in vec2 uv;
// Per instance attributes
layout (location = 2) in vec2 center_letter;
layout (location = 3) in vec2 size_letter;
layout (location = 4) in vec2 pos_uv;
layout (location = 5) in vec2 size_uv;
layout (location = 6) in float idx_page;

out vec3 out_uv;

uniform vec2 window_size;
uniform float scaling;

vec2 screen_to_ndc(vec2 p) {
    // Change of origin
    vec2 origin = p - window_size/2.0;

    // Scale to fit in [-1, 1]
    return vec2(2.0 * (origin.x/window_size.x), -2.0 * (origin.y/window_size.y));
}

void main() {
    vec2 ndc_pos = screen_to_ndc(center_letter + pos*32.0);

    gl_Position = vec4(ndc_pos, 0.f, 1.f);
    out_uv = vec3(uv, idx_page);
}