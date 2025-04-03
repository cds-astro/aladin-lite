#version 300 es
precision highp float;
layout (location = 0) in vec2 lonlat;

uniform mat3 u_2world;
uniform vec2 ndc_to_clip;
uniform float czf;

#include ../projection/projection.glsl;

void main() {
    // 1. Convert (lon, lat) into (x, y, z) space coo.
    vec3 p_xyz = lonlat2xyz(lonlat);
    // 2. Convert to the world coo system
    vec3 p_w = u_2world * p_xyz; 
    // 3. Process the projection
    vec2 p_clip = proj(p_w);

    vec2 p_ndc = p_clip / (ndc_to_clip * czf);
    gl_Position = vec4(p_ndc, 0.f, 1.f);
}