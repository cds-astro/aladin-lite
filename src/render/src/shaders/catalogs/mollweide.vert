#version 300 es
precision lowp float;
layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;

layout (location = 2) in vec3 center;
layout (location = 3) in vec2 center_lonlat;

uniform float current_time;
uniform mat4 model;

uniform vec2 ndc_to_clip;
uniform float clip_zoom_factor;
uniform vec2 kernel_size;

const float PI = 3.1415926535897932384626433832795f;

out vec2 out_uv;
out vec3 out_p;

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
    vec3 p = vec3(model * vec4(center, 1.0f));
    vec2 center_pos_clip_space = world2clip_mollweide(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * clip_zoom_factor)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}