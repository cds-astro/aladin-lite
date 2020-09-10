#version 300 es
precision lowp float;

out vec4 color;
in vec2 pos_clip;

uniform vec4 grid_color;
uniform mat4 world2model;
uniform mat4 model2world;
uniform float clip_zoom_factor;

uniform float meridians[20];
uniform int num_meridians;
uniform float parallels[10];
uniform int num_parallels;
uniform vec2 window_size;

const float PI = 3.141592653589793f;

vec2 world2clip_orthographic(vec3 p) {
    return vec2(-p.x, p.y);
}

vec3 clip2world_orthographic(vec2 pos_clip_space) {
    float z = 1.f - dot(pos_clip_space, pos_clip_space);
    if (z > 0.f) {
        return vec3(-pos_clip_space.x, pos_clip_space.y, sqrt(z));
    } else {
        discard;
    }
}

float d_isolon(vec3 pos_model, float theta) {
    vec3 n = vec3(cos(theta), 0.0, -sin(theta));
    // Discard the (theta + PI) meridian
    vec3 e_xz = vec3(-n.z, 0.0, n.x);
    if (dot(pos_model, e_xz) < 0.0) {
        return 1e3;
    }

    float d = abs(dot(n, pos_model));

    vec3 h_model = normalize(pos_model - n*d);
    vec3 h_world = vec3(model2world * vec4(h_model, 1.f));

    // Project to screen x and h and compute the distance
    // between the two
    vec2 h_clip = world2clip_orthographic(h_world);
    
    return length(pos_clip - h_clip);
}
float d_isolat(vec3 pos_model, float delta) {
    float y = atan(pos_model.y, length(pos_model.xz));
    float d = abs(y - delta);
    return d;
}

float grid_alpha(vec3 pos_model) {
    float v = 1e10;
    
    for (int i = 0; i < num_meridians; i++) {
        float a = d_isolon(pos_model, meridians[i]);

        v = min(a, v);
    }
    
    for (int i = 0; i < num_parallels; i++) {
        float a = d_isolat(pos_model, parallels[i]);

        v = min(a, v);
    }

    float eps = clip_zoom_factor / window_size.x;
    return smoothstep(eps, 2.0*eps, v);
}

void main() {
    vec4 transparency = vec4(0.f, 0.f, 0.f, 0.f);

    vec3 pos_world = clip2world_orthographic(pos_clip);
    vec3 pos_model = vec3(world2model * vec4(pos_world, 1.f));

    float alpha = grid_alpha(pos_model);
    color = mix(grid_color, transparency, alpha);
}