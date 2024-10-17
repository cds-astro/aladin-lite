#version 300 es
precision lowp float;

layout (location = 0) in vec2 lonlat;
layout (location = 1) in vec3 uv;

out vec3 frag_uv;

// current time in ms
uniform mat4 inv_model;
uniform vec2 ndc_to_clip;
uniform float czf;

#include ../../projection/projection.glsl;

void main() {
    vec3 p_xyz = lonlat2xyz(lonlat);
    vec4 p_w = inv_model * vec4(p_xyz, 1.0); 
    // 3. Process the projection
    vec2 p_clip = proj(p_w.xyz);

    vec2 p_ndc = p_clip / (ndc_to_clip * czf);
    gl_Position = vec4(p_ndc, 0.0, 1.0);

    frag_uv = uv;
}