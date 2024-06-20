vec2 w2c_tan(vec3 p) {
    return vec2(-p.x, p.y) / (p.z*PI);
}