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

vec2 world2clip_aitoff(vec3 p) {
    float delta = asin(p.y);
    float theta = atan(p.x, p.z);

    float theta_by_two = theta * 0.5f;

    float alpha = acos(cos(delta)*cos(theta_by_two));
    float inv_sinc_alpha = 1.f;
    if (alpha > 1e-3f) {
        inv_sinc_alpha = alpha / sin(alpha);
    }

    // The minus is an astronomical convention.
    // longitudes are increasing from right to left
    float x = -2.f * inv_sinc_alpha * cos(delta) * sin(theta_by_two);
    float y = inv_sinc_alpha * sin(delta);

    return vec2(x / PI, y / PI);
}

bool is_included_inside_projection(vec2 pos_clip_space) {
    float px2 = pos_clip_space.x * pos_clip_space.x;
    float py2 = pos_clip_space.y * pos_clip_space.y;

    return (px2 * 0.25 + py2) <= 0.25;
}

/// View to world space transformation
/// 
/// This returns a normalized vector along its first 3 dimensions.
/// Its fourth component is set to 1.
/// 
/// The Aitoff projection maps screen coordinates from [-pi; pi] x [-pi/2; pi/2]
/// 
/// # Arguments
/// 
/// * `x` - in normalized device coordinates between [-1; 1]
/// * `y` - in normalized device coordinates between [-1; 1]
vec3 clip2world_aitoff(vec2 pos_clip_space) {
    if(!is_included_inside_projection(pos_clip_space)) {
        discard;
    }

    vec2 uv = vec2(pos_clip_space.x * PI * 0.5, pos_clip_space.y * PI);
    //da uv a lat/lon
    float c = length(uv);

    float phi = asin(uv.y * sin(c) / c);
    float theta = atan(uv.x * sin(c), c * cos(c)) * 2.0;

    return vec3(
        -sin(theta) * cos(phi),
        sin(phi),
        cos(theta) * cos(phi)
    );
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
    vec2 h_clip = world2clip_aitoff(h_world);
    
    return length(pos_clip - h_clip) * 2.0;
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

    float eps = 3.0 * clip_zoom_factor / window_size.x;
    return smoothstep(eps, 2.0*eps, v);
}

void main() {
    vec4 transparency = vec4(0.f, 0.f, 0.f, 0.f);

    vec3 pos_world = clip2world_aitoff(pos_clip);
    vec3 pos_model = vec3(world2model * vec4(pos_world, 1.f));

    float alpha = grid_alpha(pos_model);
    color = mix(grid_color, transparency, alpha);
}