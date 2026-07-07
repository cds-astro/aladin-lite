#version 300 es

layout (location = 0) in vec3 a_xyz;
layout (location = 1) in vec3 b_xyz;
layout (location = 2) in vec2 vertex;

uniform mat3 u_2world;
uniform vec2 ndc_to_clip;
uniform float czf;
uniform float u_width;
uniform float u_height;
uniform float u_thickness;

out vec2 l;

#include ../projection/projection.glsl;


// High/low reconstruction.

void main() {

    // 2. World coordinates.
    vec3 p_a_w = u_2world * a_xyz;
    vec3 p_b_w = u_2world * b_xyz;


    // 3. Projection.
    vec2 p_a_clip = proj(p_a_w);
    vec2 p_b_clip = proj(p_b_w);

    // Use one shared projected delta.
    // This avoids independently recomputing the
    // subtraction in different coordinate systems.
    vec2 delta_clip = p_b_clip - p_a_clip;


    // 4. Convert endpoints to NDC.
    vec2 clip_to_ndc = 1.0 / (ndc_to_clip * czf);

    vec2 p_a_ndc = p_a_clip * clip_to_ndc;
    vec2 p_b_ndc = p_b_clip * clip_to_ndc;


    // Compute the NDC segment direction from
    // the same delta used above.
    vec2 x_b = delta_clip * clip_to_ndc;

    // Protect against degenerate or extremely small segments.
    float segment_length_sq = dot(x_b, x_b);

    vec2 y_b = vec2(0.0, 0.0);

    if (segment_length_sq > 1e-20)
    {
        y_b = normalize(vec2(-x_b.y, x_b.x));
    }


    // 5. Segment extrusion.
    float ndc2pix = 2.0 / u_width;

    vec2 p_ndc_x = x_b * vertex.x;

    vec2 p_ndc_y =
        (u_thickness + 2.0)
        * y_b
        * vertex.y
        * vec2(1.0, u_width / u_height)
        * ndc2pix;


    vec2 p_ndc =
        p_a_ndc
        + p_ndc_x
        + p_ndc_y;


    gl_Position = vec4(p_ndc.xy, 0.0, 1.0);


    // Use the same projected delta for line length.
    l = vec2(
        dot(delta_clip, delta_clip),
        vertex.y
    );
}