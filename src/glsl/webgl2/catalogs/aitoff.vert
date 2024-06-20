#version 300 es
precision lowp float;
layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 center;

uniform float current_time;
uniform mat4 model;
uniform mat4 inv_model;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform vec2 kernel_size;

out vec2 out_uv;
out vec3 out_p;

#include ../projection/projection.glsl;

void main() {
    vec3 p = vec3(inv_model * vec4(center, 1.0f));
    //p = check_inversed_longitude(p);

    vec2 center_pos_clip_space = world2clip_aitoff(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * czf)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}