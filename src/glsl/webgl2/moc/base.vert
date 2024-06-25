#version 300 es
precision highp float;
layout (location = 0) in vec2 lonlat;

uniform mat4 u_2world;
uniform vec2 ndc_to_clip;
uniform float czf;

#include ../projection/projection.glsl;

void main() {
    // 1. Convert (lon, lat) into (x, y, z) space coo.
    vec3 p_xyz = lonlat2xyz(lonlat);
    // 2. Convert to the world coo system
    vec4 p_w = u_2world * vec4(p_xyz, 1.0); 
    // 3. Process the projection
    vec2 p_clip = proj(p_w.xyz);

    vec2 p_ndc = p_clip / (ndc_to_clip * czf);
    gl_Position = vec4(p_ndc, 0.f, 1.f);
}