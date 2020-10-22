#version 300 es
precision highp float;
precision lowp sampler2D;
precision lowp isampler2D;
precision highp int;

layout (location = 0) in vec2 pos_clip_space;
layout (location = 1) in vec3 pos_world_space;

out vec3 out_vert_pos;

uniform mat4 model;
uniform vec2 ndc_to_clip;
uniform float clip_zoom_factor;

void main() {
    gl_Position = vec4(pos_clip_space / (ndc_to_clip * clip_zoom_factor), 0.0, 1.0);
    //out_vert_pos = vec3(inverse(gal_to_icrs) * model * vec4(pos_world_space, 1.f));
    out_vert_pos = vec3(model * vec4(pos_world_space, 1.f));
}