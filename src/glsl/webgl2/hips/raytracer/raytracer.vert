#version 300 es
precision highp float;
precision mediump int;

layout (location = 0) in vec2 pos_clip_space;
out vec2 out_clip_pos;
out vec3 frag_pos;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform mat4 model;

uniform sampler2D position_tex;

void main() {
    vec2 uv = pos_clip_space * 0.5 + 0.5;
    vec3 world_pos = texture(position_tex, uv).rgb;
    //world_pos = check_inversed_longitude(world_pos);

    frag_pos = vec3(model * vec4(world_pos, 1.0));

    gl_Position = vec4(pos_clip_space / (ndc_to_clip * czf), 0.0, 1.0);
    out_clip_pos = pos_clip_space;
}