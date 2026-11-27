#version 300 es

layout (location = 0) in vec2 hi_a_lonlat;
layout (location = 1) in vec2 lo_a_lonlat;
layout (location = 2) in vec2 hi_b_lonlat;
layout (location = 3) in vec2 lo_b_lonlat;
layout (location = 4) in vec2 vertex;

uniform mat3 u_2world;
uniform vec2 center_lonlat;
uniform float pos_angle;
uniform float cclat;
uniform vec2 ndc_to_clip;
uniform float czf;
uniform float u_width;
uniform float u_height;
uniform float u_thickness;

out vec2 l;

#include ../projection/projection.glsl;

float sin_dl_0(float x) {
    float x2 = x*x;
    float x3 = x*x2;
    float x5 = x3*x2;
    return x - (x3 / 6.0) + x5 / 120.0;
}

float cos_dl_0(float x) {
    float x2 = x*x;
    float x4 = x2*x2;
    return 1.0 - (x2 / 2.0) + (x4 / 24.0);
}

vec3 lonlat_to_xyz_local(vec2 dlonlat) {
    float dlon = dlonlat.x;
    float dlat = dlonlat.y;

    float slon = sin_dl_0(dlon);
    float clon = cos_dl_0(dlon);

    float slat = sin_dl_0(dlat);
    float clat = cos_dl_0(dlat);

    return vec3(
        clat * slon,
        slat,
        clat * clon
    );
}

const float ONE_MIN_RAD = 0.0002908882;
const float ONE_MIN_RAD_SQ = ONE_MIN_RAD * ONE_MIN_RAD;

void main() {
    vec3 p_a_w = vec3(0.0);
    vec3 p_b_w = vec3(0.0);

    vec2 da = hi_a_lonlat - center_lonlat;
    vec2 db = hi_b_lonlat - center_lonlat;

    if (dot(da, da) < ONE_MIN_RAD_SQ && dot(db, db) < ONE_MIN_RAD_SQ) {
        // First we compute a differential lon and lat with respect to the camera viewing direction
        vec2 dlonlat_a = hi_a_lonlat - center_lonlat + lo_a_lonlat;
        vec2 dlonlat_b = hi_b_lonlat - center_lonlat + lo_b_lonlat;

        // Keep the dlonlat ~ small i.e. between [-PI; PI] instead of [0; 2PI] 
        if (dlonlat_a.x > PI)
            dlonlat_a.x -= 2.0 * PI;

        if (dlonlat_b.x > PI)
            dlonlat_b.x -= 2.0 * PI;


        // There is a scaling factor to apply to the longitude with high latitudes
        dlonlat_a.x *= cclat;
        dlonlat_b.x *= cclat;

        mat2 pos_angle_mat = mat2(cos(pos_angle), -sin(pos_angle), sin(pos_angle), cos(pos_angle));

        // Convert to local xyz (valid for small dlonlat)
        p_a_w = lonlat_to_xyz_local(pos_angle_mat * dlonlat_a);
        p_b_w = lonlat_to_xyz_local(pos_angle_mat * dlonlat_b);
    } else {
        vec3 p_a_xyz = lonlat2xyz(hi_a_lonlat);
        vec3 p_b_xyz = lonlat2xyz(hi_b_lonlat);
        // 2. Convert to the world coo system
        p_a_w = u_2world * p_a_xyz; 
        p_b_w = u_2world * p_b_xyz;
    }

    // Projection in the local space
    vec2 p_a_clip = proj(p_a_w);
    vec2 p_b_clip = proj(p_b_w);

    // Use one shared projected delta.
    // This avoids independently recomputing the
    // subtraction in different coordinate systems.
    vec2 delta_clip = p_b_clip - p_a_clip;


    // Convert endpoints to NDC.
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