precision mediump float;

varying vec2 pos_clip;

uniform vec4 color;
uniform mat4 model;
uniform mat4 to_icrs;
uniform mat4 to_galactic;
uniform mat4 inv_model;
uniform float czf;

uniform float meridians[20];
uniform int num_meridians;
uniform float parallels[10];
uniform int num_parallels;

uniform vec2 window_size;

@import ../hips/projection;


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

    vec3 world = vec3(
        sin(theta) * cos(phi),
        sin(phi),
        cos(theta) * cos(phi)
    );
    return world;
}

float d_isolon(vec3 pos_model, float theta) {
    vec3 posmodel = pos_model;
    vec3 n = vec3(cos(theta), 0.0, -sin(theta));
    // Discard the (theta + PI) meridian
    vec3 e_xz = vec3(-n.z, 0.0, n.x);
    if (dot(posmodel, e_xz) < 0.0) {
        return 1e3;
    }

    float d = abs(dot(n, posmodel));

    vec3 h_model = normalize(posmodel - n*d);
    vec3 h_world = vec3(inv_model * to_icrs * vec4(h_model, 1.0));
    h_world = check_inversed_longitude(h_world);

    // Project to screen x and h and compute the distance
    // between the two
    vec2 h_clip = world2clip_aitoff(h_world);
    
    return length(pos_clip - h_clip) * 2.0;
}
float d_isolat(vec3 pos_model, float delta) {
    vec3 posmodel = pos_model;
    float y = atan(posmodel.y, length(pos_model.xz));
    float d = abs(y - delta);
    return d;
}

float grid_alpha(vec3 p) {
    float v = 1e10;
    float delta = asin(p.y);
    float theta = atan(p.x, p.z);

    float m = 0.0;
    float mdist = 10.0;
    for (int i = 0; i < 20; i++) {
        float tmp = meridians[i];
        if (tmp > PI) {
            tmp -= 2.0 * PI;
        }
        float d = abs(theta - tmp);
        if (d < mdist) {
            mdist = d;
            m = tmp;
        }
        if(i == num_meridians - 1) {
            break;
        }
    }

    float par = 0.0;
    float pdist = 10.0;
    for (int i = 0; i < 10; i++) {
        float d = abs(delta - parallels[i]);
        if (d < pdist) {
            pdist = d;
            par = parallels[i];
        }
        if(i == num_parallels - 1) {
            break;
        }
    }

    /*float a = 0.0;
    if (mdist < pdist) {
        a = d_isolon(p, m);
    } else {
        a = d_isolat(p, par);
    }
    v = min(a, v);*/
    v = min(d_isolon(p, m), v);
    v = min(d_isolat(p, par), v);

    float eps = 3.0 * czf / window_size.x;
    return smoothstep(eps, 2.0*eps, v);
}

void main() {
    vec4 transparency = vec4(0.0);

    vec3 pos_world = clip2world_aitoff(pos_clip);
    pos_world = check_inversed_longitude(pos_world);

    vec3 pos_model = vec3(to_galactic * model * vec4(pos_world, 1.0));

    float alpha = grid_alpha(pos_model);
    vec4 color = mix(color, transparency, alpha);
    gl_FragColor = color;
}