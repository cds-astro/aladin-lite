vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}