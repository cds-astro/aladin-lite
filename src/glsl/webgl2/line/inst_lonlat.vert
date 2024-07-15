#version 300 es
precision highp float;
layout (location = 0) in vec2 p_a_lonlat;
layout (location = 1) in vec2 p_b_lonlat;
layout (location = 2) in vec2 vertex;

uniform mat4 u_2world;
uniform vec2 ndc_to_clip;
uniform float czf;
uniform float u_width;
uniform float u_height;
uniform float u_thickness;

out float l;

#include ../projection/projection.glsl;

void main() {
    // 1. Convert (lon, lat) into (x, y, z) space coo.
    vec3 p_a_xyz = lonlat2xyz(p_a_lonlat);
    vec3 p_b_xyz = lonlat2xyz(p_b_lonlat);
    // 2. Convert to the world coo system
    vec4 p_a_w = u_2world * vec4(p_a_xyz, 1.0); 
    vec4 p_b_w = u_2world * vec4(p_b_xyz, 1.0);
    // 3. Process the projection
    vec2 p_a_clip = proj(p_a_w.xyz);
    vec2 p_b_clip = proj(p_b_w.xyz);

    vec2 da = p_a_clip - p_b_clip;
    l = da.x*da.x + da.y*da.y;

    vec2 p_a_ndc = p_a_clip / (ndc_to_clip * czf);
    vec2 p_b_ndc = p_b_clip / (ndc_to_clip * czf);

    // 4. Determine the final position 
    vec2 x_b = p_b_ndc - p_a_ndc;
    vec2 y_b = normalize(vec2(-x_b.y, x_b.x));

    float ndc2pix = 2.0 / u_width;
    vec2 p_ndc = p_a_ndc + x_b * vertex.x + u_thickness * y_b * vertex.y * vec2(1.0, u_width/u_height) * ndc2pix;
    gl_Position = vec4(p_ndc, 0.f, 1.f);
}