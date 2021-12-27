precision mediump float;
precision mediump int;

attribute vec2 lonlat;
//layout (location = 1) in vec3 position;
attribute vec2 ndc_pos;
attribute vec3 uv_start;
attribute vec3 uv_end;
attribute float time_tile_received;
attribute float m0;
attribute float m1;

varying vec3 frag_uv_start;
varying vec3 frag_uv_end;
varying float frag_blending_factor;
varying float m_start;
varying float m_end;

uniform mat4 inv_model;
uniform vec2 ndc_to_clip;

// current time in ms
uniform float current_time;

@import ../projection;

void main() {
    /*
    vec3 world_pos = vec3(inv_model * vec4(position, 1.f));
    world_pos = check_inversed_longitude(world_pos);

    vec2 ndc_pos = world2clip_orthographic(world_pos) / (ndc_to_clip * czf);
    */
    gl_Position = vec4(ndc_pos, 0.0, 1.0);

    frag_uv_start = uv_start;
    frag_uv_end = uv_end;

    frag_blending_factor = min((current_time - time_tile_received) / 500.0, 1.0);
    m_start = m0;
    m_end = m1;
}