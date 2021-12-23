const float TWICE_PI = 6.28318530718;
const float PI = 3.141592653589793;

struct HashDxDy {
    int idx;
    float dx;
    float dy;
};

uniform sampler2D ang2pixd;
HashDxDy hash_with_dxdy(vec2 radec) {
    vec2 uv = vec2(radec.x/TWICE_PI + 1.0, (radec.y/PI) + 0.5);
    vec3 v = texture2D(ang2pixd, uv).rgb;

    return HashDxDy(
        int(v.x * 255.0),
        v.y,
        v.z
    );
}