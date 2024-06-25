
vec2 w2c_ait(vec3 p) {
    float r = length(p.zx);
    float w = sqrt((r * (r + p.z)) * 0.5f); // = cos(b) cos(l/2)
    w = sqrt((1.0 + w) * 0.5f);             // = 1 / gamma
    float y2d = p.y / w;

    float x2d = 0.0;
    if (abs(p.x) < 5e-3) {
        float x_over_r = p.x/r;
        x2d = -p.x * (1.0 - x_over_r*x_over_r/21.0) / w;
    } else {
        w = sqrt((r*r - r*p.z) * 2.0) / w; // = 2 * gamma * cos(b) sin(l/2)
        x2d = sign(-p.x) * w;
    }

    return vec2(x2d * 0.5, y2d) / SQRT_2;
}
