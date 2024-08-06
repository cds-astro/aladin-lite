#version 300 es
precision highp float;
precision mediump int;

layout (location = 0) in vec2 pos_clip_space;
layout (location = 1) in vec3 pos_world_space;

out vec2 out_clip_pos;
out vec3 frag_pos;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform mat4 model;

void main() {
    vec2 uv = pos_clip_space * 0.5 + 0.5;
    //world_pos = check_inversed_longitude(world_pos);

    frag_pos = vec3(model * vec4(pos_world_space, 1.0));

    gl_Position = vec4(pos_clip_space / (ndc_to_clip * czf), 0.0, 1.0);
    out_clip_pos = pos_clip_space;
}