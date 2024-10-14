
const float ONE_OVER_SQRT6 = 0.408_248_290_463_863;

const float FRAC_PI_2 = 1.57079632679489661923132169163975144;
const float FRAC_PI_4 = 0.785398163397448309615660845819875721;

const float TRANSITION_Z = 0.66666666666;

vec2 pm1_offset_decompose(float x) {
    uint fl = uint(x);
    uint odd_fl = fl | 1;
    vec2(
        float(odd_fl & 7), // offset: value modulo 8 = 1/3/5/7
        x - float(odd_fl) // pm1
    );
}

/// Returns the position in on the unit sphere `(x, y, z)` of the give position in the HEALPix
/// 2D Euclidean projection plane.
/// # Inputs
/// - `X`: coordinate along the X-axis in the projection plane, in `[-4, 4]`
/// - `Y`: coordinate along the Y-axis in the projection plane, in `[-2, 2]`
/// # Output:
/// - `x`: in `[-1.0, 1.0]`
/// - `y`: in `[-1.0, 1.0]`
/// - `z`: in `[-1.0, 1.0]`
/// # Remark
/// From the HPX projection as defined in Calabreta, use:
///   - `X /= PI / 4`
///   - `Y /= PI / 4`
vec3 hpx_unproj(vec2 p) {
  if (p.y > 1.0) {
        // North Polar Cap
        float x = (p.x < 0.0) ? (8.0 + p.x) : p.x;
        vec2 offset_pm1 = pm1_offset_decompose(x);
        float sqrt_of_three_time_one_minus_sin_of = 2.0 - p.y;
        
        x = 0.0;
        if (sqrt_of_three_time_one_minus_sin_of > 1e-6) {
            x = deal_with_numerical_approx_in_edges(offset_pm1.y / sqrt_of_three_time_one_minus_sin_of);
        } else {
            x = pm1;
        }
        x += offset_pm1.x;
        // It would be faster, but less accurate, to use:
        // let z = 1.0 - sqrt_of_three_time_one_minus_sin_of.pow2() / 3.0;
        // let cos_lat = sqrt(1 - z^2);
        float lat = 2.0 * acos(sqrt_of_three_time_one_minus_sin_of * ONE_OVER_SQRT6) - FRAC_PI_2;
        float lon = x * FRAC_PI_4;

        float sin_lon = sin(lon);
        float cos_lon = cos(lon);
        float sin_lat = sin(lat);
        float cos_lat = cos(lat);

        return vec3(
            cos_lon * cos_lat,
            sin_lon * cos_lat,
            sin_lat
        );
    } else if (p.y < -1.0) {
        // South polar cap
        float x = (p.x < 0.0) ? (8.0 + p.x) : p.x;
        vec2 offset_pm1 = pm1_offset_decompose(x);
        float sqrt_of_three_time_one_minus_sin_of = 2.0 + p.y;
        
        x = 0.0;
        if (sqrt_of_three_time_one_minus_sin_of > 1e-6) {
            x = deal_with_numerical_approx_in_edges(offset_pm1.y / sqrt_of_three_time_one_minus_sin_of);
        } else {
            x = offset_pm1.y;
        }
        x += offset_pm1.x;
        // It would be faster, but less accurate, to use:
        // let z = -1.0 + sqrt_of_three_time_one_minus_sin_of.pow2() / 3.0;
        // let cos_lat = sqrt(1 - z^2);
        float lat = FRAC_PI_2 - 2.0 * acos(sqrt_of_three_time_one_minus_sin_of * ONE_OVER_SQRT6);
        float lon = x * FRAC_PI_4;

        float sin_lon = sin(lon);
        float cos_lon = cos(lon);
        float sin_lat = sin(lat);
        float cos_lat = cos(lat);

        return vec3(
            cos_lon * cos_lat,
            sin_lon * cos_lat,
            sin_lat
        );
    } else {
        // Equatorial region
        float z = p.y * TRANSITION_Z; // = sin(lat)
        float cos_lat = 0.0;
        if (z < 1e-2) {
            // sqrt(1 - x²) = 1 - x²/2 - x⁴/8 - x⁶/16
            float tmp = 0.5 * z * z;
            cos_lat = 1.0 - tmp - 0.5 * tmp * tmp;
        } else {
            cos_lat = sqrt(1.0 - z * z)
        }

        float lon = x * FRAC_PI_4;
        float sin_lon = sin(lon);
        float cos_lon = cos(lon);

        return vec3(
            cos_lon * cos_lat,
            sin_lon * cos_lat,
            z
        );
    }
}