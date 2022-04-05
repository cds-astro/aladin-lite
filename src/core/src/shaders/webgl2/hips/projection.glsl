const float PI = 3.1415926535897932384626433832795f;

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

// HEALPix projection
const float TWICE_PI = 6.28318530718f;
const float FOUR_OVER_PI = 1.27323954474f;
const float TRANSITION_Z = 0.66666666666f;
const float TRANSITION_Z_INV = 1.5f;

int quarter(vec2 p) {
    int x_neg = int(p.x < 0.0f);
    int y_neg = int(p.y < 0.0f);
    int q = (x_neg + y_neg) | (y_neg << 1);
    return q;
}

float xpm1(vec2 p) {
    bool x_neg = (p.x < 0.0f);
    //debug_assert!(x_neg <= 1);
    bool y_neg = (p.y < 0.0f);
    //debug_assert!(y_neg <= 1);
    // The purpose it to have the same numerical precision for each base cell
    // by avoiding subtraction by 1 or 3 or 5 or 7
    float lon = atan(abs(p.y), abs(p.x));
    //debug_assert!(0.0 <= lon && lon <= PI / 2.0);
    float x02 = lon * FOUR_OVER_PI;
    //debug_assert!(0.0 <= x02 && x02 <= 2.0);
    if (x_neg != y_neg) { // Could be replaced by a sign copy from (x_neg ^ y_neg) << 32
        return 1.0f - x02;
    } else {
        return x02 - 1.0f;
    }
}

float one_minus_z_pos(vec3 p) {
    //debug_assert!(z > 0.0);
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256

    if (d2 < 1e-1f) { // <=> dec > 84.27 deg
        return d2 * (0.5f + d2 * (0.125f + d2 * (0.0625f + d2 * (0.0390625f + d2 * 0.02734375f))));
    }
    return 1.0f - p.z;
}

float one_minus_z_neg(vec3 p) {
    //debug_assert!(z < 0.0);
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256
    if (d2 < 1e-1f) { // <=> dec < -84.27 deg
        // 0.5 * d2 + 0.125 * d2 * d2
        return d2 * (0.5f + d2 * (0.125f + d2 * (0.0625f + d2 * (0.0390625f + d2 * 0.02734375f))));
    }
    return p.z + 1.0f;
}

vec2 world2clip_healpix(vec3 p) {
    //assert!(depth <= 14);
    //assert!(-1.0 <= x && x <= 1.0);
    //assert!(-1.0 <= y && y <= 1.0);
    //assert!(-1.0 <= z && z <= 1.0);
    //debug_assert!(1.0 - (x * x + y * y + z * z) < 1e-5);
    // A f32 mantissa contains 23 bits.
    // - it basically means that when storing (x, y) coordinates,
    //   we can go as deep as depth 24 (or maybe 25)
    /*float x_pm1 = xpm1(p.xy);
    int q = quarter(p.xy);

    vec2 p_proj = vec2(0.f);
    if (p.z > TRANSITION_Z) {
        // North polar cap, Collignon projection.
        // - set the origin to (PI/4, 0)
        float sqrt_3_one_min_z = sqrt(3.0f * one_minus_z_pos(p));
        int offset = 1 | (q << 1);
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z + float(offset) - 4.0, 2.0f - sqrt_3_one_min_z);
    } else if (p.z < -TRANSITION_Z) {
        // South polar cap, Collignon projection
        // - set the origin to (PI/4, -PI/2)
        float sqrt_3_one_min_z = sqrt(3.0f * one_minus_z_neg(p));
        int offset = 1 | (q << 1);
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z + float(offset) - 4.0, -2.0f + sqrt_3_one_min_z);
    } else {
        // Equatorial region, Cylindrical equal area projection
        // - set the origin to (PI/4, 0)               if q = 2
        // - set the origin to (PI/4, -PI/2)           if q = 0
        // - set the origin to (0, -TRANSITION_LAT)    if q = 3
        // - set the origin to (PI/2, -TRANSITION_LAT) if q = 1
        // let zero_or_one = (x_cea as u8) & 1;
        float y_pm1 = p.z * TRANSITION_Z_INV;
        // |\2/|
        // .3X1.
        // |/0\|
        int q01 = int(x_pm1 > y_pm1);  // 0/1
        //debug_assert!(q01 == 0 || q01 == 1);
        int q12 = int(x_pm1 >= -y_pm1); // 0\1
        //debug_assert!(q12 == 0 || q12 == 1);
        int q03 = 1 - q12; // 1\0
        //let q13 = q01 ^ q12; debug_assert!(q13 == 0 || q13 == 1);
        int q1 = q01 & q12; // = 1 if q1, 0 else
        //debug_assert!( q1 == 0 ||  q1 == 1);
        // x: xcea - 0 if q3 | xcea - 2 if q1 | xcea - 1 if q0 or q2
        //let x_proj = x_pm1 - ((q01 + q12) as i8 - 1) as f32;
        // y: y - 0 if q2 | y - 1 if q1 or q3 | y - 2 if q0 
        //let y_proj = y_pm1 + (q01 + q03) as f32;
        int offset = 1 | (q << 1);

        p_proj = vec2(
            x_pm1 - float(q01 + q12 - 1) + float(offset) - 4.0,
            y_pm1 + float(q01 + q03) - 1.0
        );
    }

    return p_proj * vec2(0.025, 0.05);*/
    float x_pm1 = xpm1(p.xy);
    int q = quarter(p.xy);
    vec2 p_proj = vec2(0.0);
    if (p.z > TRANSITION_Z) {
        // North polar cap, Collignon projection.
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_pos(p));
        int offset = 1 | (q << 1);
        if (offset > 4) {
            offset = offset - 8;
        }
        p_proj = vec2(
            x_pm1 * sqrt_3_one_min_z + float(offset), 
            2.0 - sqrt_3_one_min_z
        );
    } else if (p.z < -TRANSITION_Z) {
        // South polar cap, Collignon projection
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_neg(p));
        int offset = 1 | (q << 1);
        if (offset > 4) {
            offset = offset - 8;
        }
        p_proj = vec2(
            x_pm1 * sqrt_3_one_min_z + float(offset),
            -2.0 + sqrt_3_one_min_z
        );
    } else {
        // Equatorial region, Cylindrical equal area projection
        p_proj = vec2(
            atan(p.y, p.x) * FOUR_OVER_PI, p.z * TRANSITION_Z_INV
        );
    }

    return p_proj * vec2(0.025, 0.05);
}