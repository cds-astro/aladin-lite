#version 300 es
precision mediump float;
precision mediump int;

layout (location = 0) in vec2 pos_clip_space;
layout (location = 1) in vec2 lonlat;
layout (location = 2) in vec3 pos_world_space;

out vec3 out_vert_pos;
out vec2 out_lonlat;

uniform mat4 model;
uniform vec2 ndc_to_clip;
uniform float czf_hi;
uniform float czf_low;


void main() {
    gl_Position = vec4(pos_clip_space / (ndc_to_clip * (czf_hi + czf_low)), 0.0, 1.0);
    //out_vert_pos = vec3(inverse(gal_to_icrs) * model * vec4(pos_world_space, 1.f));
    out_vert_pos = vec3(model * vec4(pos_world_space, 1.f));
    out_lonlat = vec2(atan(out_vert_pos.x, out_vert_pos.z), asin(out_vert_pos.y));
}