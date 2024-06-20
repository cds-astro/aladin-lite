vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}