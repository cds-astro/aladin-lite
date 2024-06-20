vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}