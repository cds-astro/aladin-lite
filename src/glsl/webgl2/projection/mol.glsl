const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
    // Initial guess so that for z ~= 1, gamma ~= PI/2.
    // Smooth function for small |z|, so no big deal having a bad init value.
    float x = 2.0 * asin(z);
    float f = x + sin(x) - cte; 
    int i = 0;
    while (abs(f) > eps && i < n_iter) {
        x -= f / (1.0 + cos(x));
        f = x + sin(x) - cte;
        i += 1;
    }

    return 0.5 * x;
}

vec2 w2c_mol(vec3 p) {
    float g = newton_solve(p.y);

    float sg = sin(g);
    float cg = cos(g);
    return vec2((atan(-p.x, p.z) * cg) / PI, sg);
}