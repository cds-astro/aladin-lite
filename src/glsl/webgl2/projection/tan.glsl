vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}