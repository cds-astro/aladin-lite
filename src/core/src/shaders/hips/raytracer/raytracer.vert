#version 300 es
precision highp float;
precision highp int;

layout (location = 0) in vec2 pos_clip_space;
layout (location = 1) in vec2 lonlat;
layout (location = 2) in vec3 pos_world_space;

out vec3 out_vert_pos;
out vec2 out_clip_pos;

uniform mat4 model;
uniform vec2 ndc_to_clip;
uniform float czf;

void main() {
    gl_Position = vec4(pos_clip_space / (ndc_to_clip * czf), 0.0, 1.0);
    //out_vert_pos = vec3(inverse(gal_to_icrs) * model * vec4(pos_world_space, 1.f));
    out_vert_pos = vec3(model * vec4(pos_world_space, 1.f));
    out_clip_pos = pos_clip_space;
}