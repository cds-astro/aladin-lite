#version 300 es
precision mediump float;
precision mediump int;

layout (location = 0) in vec2 lonlat;
//layout (location = 1) in vec3 position;
layout (location = 1) in vec2 ndc_pos;
layout (location = 2) in vec3 uv_start;
layout (location = 3) in vec3 uv_end;
layout (location = 4) in float time_tile_received;

out vec3 frag_uv_start;
out vec3 frag_uv_end;
out float frag_blending_factor;

uniform mat4 inv_model;
uniform vec2 ndc_to_clip;
uniform float czf_hi;
uniform float czf_low;
// current time in ms
uniform float current_time;

@import ../projection;

void main() {
    /*
    vec3 world_pos = vec3(inv_model * vec4(position, 1.f));
    world_pos = check_inversed_longitude(world_pos);

    gl_Position = vec4(world2clip_arc(world_pos) / (ndc_to_clip * (czf_hi + czf_low)), 0.0, 1.0);
    */
    gl_Position = vec4(ndc_pos, 0.0, 1.0);

    frag_uv_start = uv_start;
    frag_uv_end = uv_end;
    frag_blending_factor = min((current_time - time_tile_received) / 500.f, 1.f);
}