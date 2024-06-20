#version 300 es
precision highp float;
layout (location = 0) in vec2 p_a_lonlat;
layout (location = 1) in vec2 p_b_lonlat;
layout (location = 2) in vec2 vertex;

uniform mat4 u_2world;
uniform vec2 ndc_to_clip;
uniform float czf;
uniform float u_width;
uniform int u_proj;

out float l;

#include ../projection/projection.glsl;

vec3 lonlat2xyz(vec2 lonlat) {
    float t = lonlat.x;
    float tc = cos(t);
    float ts = sin(t);

    float d = lonlat.y;
    float dc = cos(d);
    float ds = sin(d);

    return vec3(dc * ts, ds, dc * tc);
}

vec2 proj(vec3 p) {
    if (u_proj == 0) {
        /* TAN,      Gnomonic projection        */
        return w2c_tan(p);
    } else if (u_proj == 1) {
        /* STG,	     Stereographic projection   */
        return w2c_stg(p);
    } else if (u_proj == 2) {
        /* SIN,	     Orthographic		        */
        return w2c_sin(p);
    } else if (u_proj == 3) {
        /* ZEA,	     Equal-area 		        */
        return w2c_zea(p);
    } else if (u_proj == 4) {
        // Pseudo-cylindrical projections
        /* AIT,      Aitoff                     */
        return w2c_ait(p);
    } else if (u_proj == 5) {
        // MOL,      Mollweide                  */
        return w2c_mol(p);
    } else {
        // Cylindrical projections
        // MER,      Mercator                   */
        return w2c_mer(p);
    }
}

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

    vec2 p_ndc = p_a_ndc + x_b * vertex.x + y_b * u_width * 0.001 * vertex.y;
    gl_Position = vec4(p_ndc, 0.f, 1.f);
}