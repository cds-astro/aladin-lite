vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}