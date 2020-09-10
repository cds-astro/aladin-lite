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


/// World to screen space transformation
/// X is between [-1, 1]
/// Y is between [-0.5, 0.5]
/// 
/// # Arguments
/// 
/// * `pos_world_space` - Position in the world space. Must be a normalized vector
vec2 world2clip_mollweide(vec3 p) {
    float lat = asin(p.y);
    float lon = atan(p.x, p.z);
    // X in [-1, 1]
    // Y in [-1/2; 1/2] and scaled by the screen width/height ratio
    const float eps = 1e-3;
    const int max_iter = 10;

    float cst = PI * sin(lat);

    float theta = lat;
    float f = theta + sin(theta) - cst;

    int k = 0;
    while (abs(f) > eps && k < max_iter) {
        theta = theta - f / (1.0 + cos(theta));
        f = theta + sin(theta) - cst;

        k += 1;
    }

    theta = theta * 0.5;

    // The minus is an astronomical convention.
    // longitudes are increasing from right to left
    return vec2(
        -(lon / PI) * cos(theta),
        0.5 * sin(theta)
    );
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
vec3 clip2world_mollweide(vec2 pos_clip_space) {
    if (!is_included_inside_projection(pos_clip_space)) {
        discard;
    }

    float y2 = pos_clip_space.y * pos_clip_space.y;
    float k = sqrt(1.0 - 4.0 * y2);

    float theta = PI * pos_clip_space.x / k;
    float delta = asin((2.0 * asin(2.0 * pos_clip_space.y) + 4.0 * pos_clip_space.y * k) / PI);
    
    // The minus is an astronomical convention.
    // longitudes are increasing from right to left
    return vec3(-sin(theta) * cos(delta), sin(delta), cos(theta) * cos(delta));
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
    vec2 h_clip = world2clip_mollweide(h_world);
    
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

    vec3 pos_world = clip2world_mollweide(pos_clip);
    vec3 pos_model = vec3(world2model * vec4(pos_world, 1.f));

    float alpha = grid_alpha(pos_model);
    color = mix(grid_color, transparency, alpha);
}