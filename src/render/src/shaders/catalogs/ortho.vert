#version 300 es
precision lowp float;
layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;

layout (location = 2) in vec3 center;
layout (location = 3) in vec2 center_lonlat;


uniform float current_time;
uniform mat4 model;

uniform vec2 ndc_to_clip;
uniform float clip_zoom_factor;
uniform vec2 kernel_size;

const float PI = 3.1415926535897932384626433832795f;

out vec2 out_uv;
out vec3 out_p;

vec2 world2clip_orthographic(vec3 p) {
    return vec2(-p.x, p.y);
}

void main() {
    vec3 p = vec3(model * vec4(center, 1.0f));
    vec2 center_pos_clip_space = world2clip_orthographic(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * clip_zoom_factor)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}