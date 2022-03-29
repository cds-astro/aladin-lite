const float PI = 3.1415926535897932384626433832795f;

uniform int inversed_longitude;

const mat3 inverseLongitude = mat3(
    -1.0, 0.0, 0.0,
    0.0, 1.0, 0.0,
    0.0, 0.0, 1.0
);

const mat4 GAL2J2000 = mat4(
    -0.4448296299195045,
    0.7469822444763707,
    0.4941094279435681,
    0.0,

    -0.1980763734646737,
    0.4559837762325372,
    -0.8676661489811610,
    0.0,

    -0.873437090247923,
    -0.4838350155267381,
    -0.0548755604024359,
    0.0,

    0.0,
    0.0,
    0.0,
    1.0
);

const mat4 J20002GAL = mat4(
    -0.4448296299195045,
    -0.1980763734646737,
    -0.873437090247923,
    0.0,

    0.7469822444763707,
    0.4559837762325372,
    -0.4838350155267381,
    0.0,

    0.4941094279435681,
    -0.8676661489811610,
    -0.0548755604024359,
    0.0,

    0.0,
    0.0,
    0.0,
    1.0
);

vec3 check_inversed_longitude(vec3 p) {
    if (inversed_longitude == 1) {
        return inverseLongitude * p;
    } else {
        return p;
    }
}

vec2 world2clip_orthographic(vec3 p) {
    return vec2(-p.x, p.y);
}

vec2 world2clip_aitoff(vec3 p) {
    float delta = asin(p.y);
    float theta = atan(-p.x, p.z);

    float theta_by_two = theta * 0.5f;

    float alpha = acos(cos(delta)*cos(theta_by_two));
    float inv_sinc_alpha = 1.f;
    if (alpha > 1e-3f) {
        inv_sinc_alpha = alpha / sin(alpha);
    }

    // The minus is an astronomical convention.
    // longitudes are increasing from right to left
    float x = 2.f * inv_sinc_alpha * cos(delta) * sin(theta_by_two);
    float y = inv_sinc_alpha * sin(delta);

    return vec2(x / PI, y / PI);
}

vec2 world2clip_mollweide(vec3 p) {
    // X in [-1, 1]
    // Y in [-1/2; 1/2] and scaled by the screen width/height ratio
    int max_iter = 10;

    float delta = asin(p.y);
    float theta = atan(p.x, p.z);

    float cst = PI * sin(delta);

    float phi = delta;
    float f = phi + sin(phi) - cst;

    int k = 0;
    while (abs(f) > 1e-6 && k < max_iter) {
        phi = phi - f / (1.f + cos(phi));
        f = phi + sin(phi) - cst;

        k = k + 1;
    }

    phi = phi * 0.5f;

    // The minus is an astronomical convention.
    // longitudes are increasing from right to left
    float x = (-theta / PI) * cos(phi);
    float y = 0.5f * sin(phi);

    return vec2(x, y);
}

vec2 world2clip_mercator(vec3 p) {
    // X in [-1, 1]
    // Y in [-1/2; 1/2] and scaled by the screen width/height ratio

    float delta = asin(p.y);
    float theta = atan(-p.x, p.z);

    float x = theta / PI;
    float y = asinh(tan(delta / PI));

    return vec2(x, y);
}

float arc_sinc(float x) {
    if (x > 1e-4) {
        return asin(x) / x;
    } else {
        // If a is mall, use Taylor expension of asin(a) / a
        // a = 1e-4 => a^4 = 1.e-16
        float x2 = x*x;
        return 1.0 + x2 * (1.0 + x2 * 9.0 / 20.0) / 6.0;
    }
}

vec2 world2clip_arc(vec3 p) {
    if (p.z > -1.0) {
        // Distance in the Euclidean plane (xy)
        // Angular distance is acos(x), but for small separation, asin(r)
        // is more accurate.
        float r = length(p.xy);
        if (p.z > 0.0) { // Angular distance < PI/2, angular distance = asin(r)
            r = arc_sinc(r);
        } else { // Angular distance > PI/2, angular distance = acos(x)
            r = acos(p.z) / r;
        }
        float x = p.x * r;
        float y = p.y * r;

        return vec2(-x / PI, y / PI);
    } else {
        return vec2(1.0, 0.0);
    }
}

vec2 world2clip_gnomonic(vec3 p) {
    if (p.z <= 1e-2) { // Back hemisphere (x < 0) + diverges near x=0
        return vec2(1.0, 0.0);
    } else {
        return vec2((-p.x/p.z) / PI , (p.y/p.z) / PI);
    }
}