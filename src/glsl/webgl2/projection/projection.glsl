/// Projections
const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

#include ./sin.glsl;
#include ./ait.glsl;
#include ./mol.glsl;
#include ./tan.glsl;
#include ./stg.glsl;
#include ./zea.glsl;
#include ./mer.glsl;

vec3 lonlat2xyz(vec2 lonlat) {
    float t = lonlat.x;
    float tc = cos(t);
    float ts = sin(t);

    float d = lonlat.y;
    float dc = cos(d);
    float ds = sin(d);

    return vec3(dc * ts, ds, dc * tc);
}

uniform int u_proj;

vec2 proj(vec3 p) {
    if (u_proj == 0) {
        /* TAN,      Gnomonic projection        */
        return w2c_tan(p);
    } else if (u_proj == 1) {
        /* STG,	     Stereographic projection   */
        return w2c_stg(p);
    } else if (u_proj == 2) {
        /* SIN,	     Orthographic		        */
        return w2c_sin(p);
    } else if (u_proj == 3) {
        /* ZEA,	     Equal-area 		        */
        return w2c_zea(p);
    } else if (u_proj == 4) {
        // Pseudo-cylindrical projections
        /* AIT,      Aitoff                     */
        return w2c_ait(p);
    } else if (u_proj == 5) {
        // MOL,      Mollweide                  */
        return w2c_mol(p);
    } else {
        // Cylindrical projections
        // MER,      Mercator                   */
        return w2c_mer(p);
    }
}