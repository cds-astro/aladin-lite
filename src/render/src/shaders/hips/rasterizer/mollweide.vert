#version 300 es
precision highp float;
precision lowp sampler2DArray;
precision highp int;

layout (location = 0) in vec2 lonlat;
layout (location = 1) in vec3 position;
layout (location = 2) in vec3 uv_start;
layout (location = 3) in vec3 uv_end;
layout (location = 4) in float time_tile_received;

out vec3 frag_uv_start;
out vec3 frag_uv_end;
out float frag_blending_factor;
out vec2 screen_pos;

uniform mat4 inv_model;
uniform vec2 ndc_to_clip;
uniform float clip_zoom_factor;
// current time in ms
uniform float current_time;

const float PI = 3.1415926535897932384626433832795f;

vec2 world2clip_mollweide(vec3 p) {
    // X in [-1, 1]
    // Y in [-1/2; 1/2] and scaled by the screen width/height ratio
    int max_iter = 10;

    float delta = asin(p.y);
    float theta = atan(p.x, p.z);

    float cst = PI * sin(delta);

    float phi = delta;
    float f = phi + sin(phi) - cst;

    int k = 0;
    while (abs(f) > 1e-4 && k < max_iter) {
        phi = phi - f / (1.f + cos(phi));
        f = phi + sin(phi) - cst;

        k = k + 1;
    }

    phi = phi * 0.5f;

    // The minus is an astronomical convention.
    // longitudes are increasing from right to left
    float x = -(theta / PI) * cos(phi);
    float y = 0.5f * sin(phi);

    return vec2(x, y);
}

void main() {
    vec3 world_pos = vec3(inv_model * vec4(position, 1.f));
    gl_Position = vec4(world2clip_mollweide(world_pos) / (ndc_to_clip * clip_zoom_factor), 0.0, 1.0);

    screen_pos = gl_Position.xy;
    frag_uv_start = uv_start;
    frag_uv_end = uv_end;
    frag_blending_factor = min((current_time - time_tile_received) / 500.f, 1.f);
}