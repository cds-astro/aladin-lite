vec2 w2c_zea(vec3 p) {
    // Whole sphere, r <= 2 (equal area)
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}