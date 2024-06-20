#version 300 es
precision highp float;
precision mediump int;

//layout (location = 0) in vec3 position;
layout (location = 0) in vec2 ndc_pos;
layout (location = 1) in vec3 uv_start;
layout (location = 2) in vec3 uv_end;
layout (location = 3) in float time_tile_received;
layout (location = 4) in float m0;
layout (location = 5) in float m1;

out vec3 frag_uv_start;
out vec3 frag_uv_end;
out float frag_blending_factor;
out float m_start;
out float m_end;

// current time in ms
uniform float current_time;

void main() {
    gl_Position = vec4(ndc_pos, 0.0, 1.0);

    frag_uv_start = uv_start;
    frag_uv_end = uv_end;
    frag_blending_factor = min((current_time - time_tile_received) / 200.0, 1.0);
    m_start = m0;
    m_end = m1;
}