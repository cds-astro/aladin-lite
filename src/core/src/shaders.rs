use std::collections::HashMap;
#[allow(dead_code)]
pub fn get_all() -> HashMap<&'static str, &'static str> {
    let mut out = HashMap::new();
    out.insert(
        r"hips_rasterizer_rgba.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray tex;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;
uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

void main() {
    vec4 color_start = uvw2c_rgba(frag_uv_start);
    vec4 color_end = uvw2c_rgba(frag_uv_end);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = opacity * out_frag_color.a;
}"#,
    );
    out.insert(
        r"hips_rasterizer_raster.vert",
        r#"#version 300 es
precision highp float;

layout (location = 0) in vec3 xyz;
layout (location = 1) in vec3 uv_start;
layout (location = 2) in vec3 uv_end;
layout (location = 3) in float time_tile_received;

out vec3 frag_uv_start;
out vec3 frag_uv_end;
out float frag_blending_factor;

uniform mat3 inv_model;
uniform vec2 ndc_to_clip;
uniform float czf;
uniform float current_time;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}

void main() {
    vec3 p_w = inv_model * xyz; 
    vec2 p_clip = proj(p_w);

    vec2 p_ndc = p_clip / (ndc_to_clip * czf);
    gl_Position = vec4(p_ndc, 0.0, 1.0);

    frag_uv_start = uv_start;
    frag_uv_end = uv_end;
    frag_blending_factor = min((current_time - time_tile_received) / 200.0, 1.0);
}"#,
    );
    out.insert(
        r"image_base.vert",
        r#"#version 300 es
precision highp float;

layout (location = 0) in vec2 ndc_pos;
layout (location = 1) in vec2 uv;

out vec2 frag_uv;

void main() {
    gl_Position = vec4(ndc_pos, 0.0, 1.0);
    frag_uv = uv;
}"#,
    );
    out.insert(
        r"line_base.vert",
        r#"#version 300 es
precision highp float;
layout (location = 0) in vec2 ndc_pos;

out float l;

void main() {
  gl_Position = vec4(
        ndc_pos,
        0.0,
        1.0
    );
}"#,
    );
    out.insert(
        r"catalogs_ortho.frag",
        r#"#version 300 es
precision highp float;

in vec2 out_uv;
in vec3 out_p;

out vec4 color;

uniform sampler2D kernel_texture;
uniform float max_density; // max number of sources in a kernel sized HEALPix cell at the current depth
uniform float fov;
uniform float strength;
void main() {
    if (out_p.z < 0.f) {
        discard;
    }

    color = texture(kernel_texture, out_uv) / max(log2(fov*100.0), 1.0);
    color.r *= strength;
}"#,
    );
    out.insert(
        r"hips3d_f32.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler3D;

uniform sampler3D tex;

in vec3 frag_uv;

out vec4 out_frag_color;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

uniform float opacity;

void main() {
    vec3 uv = vec3(frag_uv.xyz);
    uv.y = 1.0 - uv.y;

    vec4 color = uvw2c_f32(uv);

    out_frag_color = color;
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"fits_i32.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp int;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}





vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uv2c_f32(vec2 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uv2c_i32(vec2 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return val2c(val);
}

vec4 uv2c_i16(vec2 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return val2c(val);
}

vec4 uv2c_u8(vec2 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

void main() {
    vec2 uv = frag_uv;
    uv.y = 1.0 - uv.y;

    out_frag_color = uv2c_i32(frag_uv);
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips_rasterizer_f32.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray tex;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

uniform float opacity;

void main() {
    vec3 uv0 = frag_uv_start;
    vec3 uv1 = frag_uv_end;
    uv0.y = 1.0 - uv0.y;
    uv1.y = 1.0 - uv1.y;

    vec4 color_start = uvw2c_f32(uv0);
    vec4 color_end = uvw2c_f32(uv1);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"line_base.frag",
        r#"#version 300 es

precision highp float;

out vec4 color;
in vec2 l;

uniform vec4 u_color;
uniform float u_thickness;
uniform float u_width;
uniform float u_height;

void main() {
    if (l.x > 0.05) {
        discard;
    } else {
        color = u_color;

        float dist = abs((u_thickness + 2.0) * l.y);

        float half_thickness = (u_thickness + 2.0) * 0.5;
        color.a = color.a * (1.0 - smoothstep(half_thickness - 1.0, half_thickness, dist));
    }
}"#,
    );
    out.insert(
        r"moc_base.vert",
        r#"#version 300 es
precision highp float;
layout (location = 0) in vec2 lonlat;

uniform mat3 u_2world;
uniform vec2 ndc_to_clip;
uniform float czf;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}

void main() {
    vec3 p_xyz = lonlat2xyz(lonlat);
    vec3 p_w = u_2world * p_xyz; 
    vec2 p_clip = proj(p_w);

    vec2 p_ndc = p_clip / (ndc_to_clip * czf);
    gl_Position = vec4(p_ndc, 0.f, 1.f);
}"#,
    );
    out.insert(
        r"hips_rasterizer_rgba2cmap.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray tex;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;
uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

void main() {
    vec4 color_start = uvw2cmap_rgba(frag_uv_start);
    vec4 color_end = uvw2cmap_rgba(frag_uv_end);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = opacity * out_frag_color.a;
}"#,
    );
    out.insert(
        r"hips_rasterizer_i16.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray tex;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

uniform float opacity;

void main() {
    vec3 uv0 = frag_uv_start;
    vec3 uv1 = frag_uv_end;
    uv0.y = 1.0 - uv0.y;
    uv1.y = 1.0 - uv1.y;

    vec4 color_start = uvw2c_i16(uv0);
    vec4 color_end = uvw2c_i16(uv1);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"passes_post_vertex_100es.vert",
        r#"#version 300 es
precision highp float;

layout (location = 0) in vec2 a_pos;
out vec2 v_tc;

void main() {
    gl_Position = vec4(a_pos * 2. - 1., 0.0, 1.0);
    v_tc = a_pos;
}"#,
    );
    out.insert(
        r"hips_raytracer_i16.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;
precision highp int;

uniform sampler2DArray tex;

in vec3 frag_pos;
in vec2 out_clip_pos;
out vec4 out_frag_color;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[12];

uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}
const float TWICE_PI = 6.28318530718;
const float PI = 3.141592653589793;
const float FOUR_OVER_PI = 1.27323954474;
const float TRANSITION_Z = 0.66666666666;
const float TRANSITION_Z_INV = 1.5;

int quarter(vec2 p) {
    int x_neg = int(p.x < 0.0);
    int y_neg = int(p.y < 0.0);
    int q = (x_neg + y_neg) | (y_neg << 1);
    return q;
}

float xpm1(vec2 p) {
    bool x_neg = (p.x < 0.0);
    bool y_neg = (p.y < 0.0);
    float lon = atan(abs(p.y), abs(p.x));
    float x02 = lon * FOUR_OVER_PI;
    if (x_neg != y_neg) { // Could be replaced by a sign copy from (x_neg ^ y_neg) << 32
        return 1.0 - x02;
    } else {
        return x02 - 1.0;
    }
}

float one_minus_z_pos(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256

    if (d2 < 1e-1) { // <=> dec > 84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return 1.0f - p.z;
}

float one_minus_z_neg(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256
    if (d2 < 1e-1f) { // <=> dec < -84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return p.z + 1.0;
}

int ij2z(int i, int j) {
    int i4 = i | (j << 2);

    int j4 = (i4 ^ (i4 >> 1)) & 0x22222222;
    int i5 = i4 ^ j4 ^ (j4 << 1);

    return i5;
}

struct HashDxDy {
    int idx;
    float dx;
    float dy;
};

uniform sampler2D ang2pixd;
HashDxDy hash_with_dxdy2(vec2 radec) {
    vec2 aa = vec2(radec.x/TWICE_PI + 1.0, (radec.y/PI) + 0.5);
    vec3 v = texture(ang2pixd, aa).rgb;
    return HashDxDy(
        int(v.x * 255.0),
        v.y,
        v.z
    );
}
HashDxDy hash_with_dxdy(int depth, vec3 p) {
    
    int nside = 1 << depth;
    float half_nside = float(nside) * 0.5;

    float x_pm1 = xpm1(p.xy);
    int q = quarter(p.xy);

    int d0h = 0;
    vec2 p_proj = vec2(0.0);
    if (p.z > TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_pos(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, 2.0 - sqrt_3_one_min_z);
        d0h = q;
    } else if (p.z < -TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_neg(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, sqrt_3_one_min_z);
        d0h = q + 8;
    } else {
        float y_pm1 = p.z * TRANSITION_Z_INV;
        int q01 = int(x_pm1 > y_pm1);  // 0/1
        int q12 = int(x_pm1 >= -y_pm1); // 0\1
        int q03 = 1 - q12; // 1\0
        int q1 = q01 & q12; // = 1 if q1, 0 else
        p_proj = vec2(
            x_pm1 - float(q01 + q12 - 1),
            y_pm1 + float(q01 + q03)
        );
        d0h = ((q01 + q03) << 2) + ((q + q1) & 3);
    }

    float x = (half_nside * (p_proj.x + p_proj.y));
    float y = (half_nside * (p_proj.y - p_proj.x));
    int i = int(x);
    int j = int(y);

    return HashDxDy(
        (d0h << (depth << 1)) + ij2z(i, j),
        x - float(i),
        y - float(j)
    );
}
vec3 xyz2uv(vec3 xyz) {
    HashDxDy result = hash_with_dxdy(0, xyz.zxy);

    int idx = result.idx;
    vec2 offset = vec2(result.dy, result.dx);
    Tile tile = textures_tiles[idx];

    return vec3(offset, float(tile.texture_idx));
}

void main() {
    vec3 uv = xyz2uv(normalize(frag_pos));

    uv.y = 1.0 - uv.y;
    vec4 c = uvw2c_i16(uv);


    out_frag_color = c;
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips3d_u8.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler3D;

uniform sampler3D tex;

in vec3 frag_uv;

out vec4 out_frag_color;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

uniform float opacity;

void main() {
    vec3 uv = vec3(frag_uv.xyz);
    uv.y = 1.0 - uv.y;

    vec4 color = uvw2c_u8(uv);

    out_frag_color = color;
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"catalogs_tan.vert",
        r#"#version 300 es
precision highp float;

layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 center;

uniform float current_time;
uniform mat3 inv_model;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform vec2 kernel_size;

out vec2 out_uv;
out vec3 out_p;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}


void main() {
    vec3 p = inv_model * center;

    vec2 center_pos_clip_space = world2clip_gnomonic(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * czf)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}"#,
    );
    out.insert(
        r"hips_raytracer_rgba.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;
precision highp int;

uniform sampler2DArray tex;

in vec2 out_clip_pos;
in vec3 frag_pos;
out vec4 out_frag_color;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[12];

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}
const float TWICE_PI = 6.28318530718;
const float PI = 3.141592653589793;
const float FOUR_OVER_PI = 1.27323954474;
const float TRANSITION_Z = 0.66666666666;
const float TRANSITION_Z_INV = 1.5;

int quarter(vec2 p) {
    int x_neg = int(p.x < 0.0);
    int y_neg = int(p.y < 0.0);
    int q = (x_neg + y_neg) | (y_neg << 1);
    return q;
}

float xpm1(vec2 p) {
    bool x_neg = (p.x < 0.0);
    bool y_neg = (p.y < 0.0);
    float lon = atan(abs(p.y), abs(p.x));
    float x02 = lon * FOUR_OVER_PI;
    if (x_neg != y_neg) { // Could be replaced by a sign copy from (x_neg ^ y_neg) << 32
        return 1.0 - x02;
    } else {
        return x02 - 1.0;
    }
}

float one_minus_z_pos(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256

    if (d2 < 1e-1) { // <=> dec > 84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return 1.0f - p.z;
}

float one_minus_z_neg(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256
    if (d2 < 1e-1f) { // <=> dec < -84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return p.z + 1.0;
}

int ij2z(int i, int j) {
    int i4 = i | (j << 2);

    int j4 = (i4 ^ (i4 >> 1)) & 0x22222222;
    int i5 = i4 ^ j4 ^ (j4 << 1);

    return i5;
}

struct HashDxDy {
    int idx;
    float dx;
    float dy;
};

uniform sampler2D ang2pixd;
HashDxDy hash_with_dxdy2(vec2 radec) {
    vec2 aa = vec2(radec.x/TWICE_PI + 1.0, (radec.y/PI) + 0.5);
    vec3 v = texture(ang2pixd, aa).rgb;
    return HashDxDy(
        int(v.x * 255.0),
        v.y,
        v.z
    );
}
HashDxDy hash_with_dxdy(int depth, vec3 p) {
    
    int nside = 1 << depth;
    float half_nside = float(nside) * 0.5;

    float x_pm1 = xpm1(p.xy);
    int q = quarter(p.xy);

    int d0h = 0;
    vec2 p_proj = vec2(0.0);
    if (p.z > TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_pos(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, 2.0 - sqrt_3_one_min_z);
        d0h = q;
    } else if (p.z < -TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_neg(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, sqrt_3_one_min_z);
        d0h = q + 8;
    } else {
        float y_pm1 = p.z * TRANSITION_Z_INV;
        int q01 = int(x_pm1 > y_pm1);  // 0/1
        int q12 = int(x_pm1 >= -y_pm1); // 0\1
        int q03 = 1 - q12; // 1\0
        int q1 = q01 & q12; // = 1 if q1, 0 else
        p_proj = vec2(
            x_pm1 - float(q01 + q12 - 1),
            y_pm1 + float(q01 + q03)
        );
        d0h = ((q01 + q03) << 2) + ((q + q1) & 3);
    }

    float x = (half_nside * (p_proj.x + p_proj.y));
    float y = (half_nside * (p_proj.y - p_proj.x));
    int i = int(x);
    int j = int(y);

    return HashDxDy(
        (d0h << (depth << 1)) + ij2z(i, j),
        x - float(i),
        y - float(j)
    );
}
vec3 xyz2uv(vec3 xyz) {
    HashDxDy result = hash_with_dxdy(0, xyz.zxy);

    int idx = result.idx;
    vec2 offset = vec2(result.dy, result.dx);
    Tile tile = textures_tiles[idx];

    return vec3(offset, float(tile.texture_idx));
}

uniform float opacity;
uniform vec4 no_tile_color;

void main() {
    vec3 uv = xyz2uv(normalize(frag_pos));
    vec4 c = uvw2c_rgba(uv);

    out_frag_color = c;
    out_frag_color = vec4(c.rgb, opacity * c.a);
}"#,
    );
    out.insert(
        r"fits_u8.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp int;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}





vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uv2c_f32(vec2 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uv2c_i32(vec2 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return val2c(val);
}

vec4 uv2c_i16(vec2 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return val2c(val);
}

vec4 uv2c_u8(vec2 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

void main() {
    vec2 uv = frag_uv;
    uv.y = 1.0 - uv.y;

    out_frag_color = uv2c_u8(frag_uv);
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips_raytracer_i32.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;
precision highp usampler2DArray;
precision highp isampler2DArray;
precision highp int;

uniform sampler2DArray tex;

in vec3 frag_pos;
in vec2 out_clip_pos;
out vec4 out_frag_color;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[12];

uniform float opacity;

const float TWICE_PI = 6.28318530718;
const float PI = 3.141592653589793;
const float FOUR_OVER_PI = 1.27323954474;
const float TRANSITION_Z = 0.66666666666;
const float TRANSITION_Z_INV = 1.5;

int quarter(vec2 p) {
    int x_neg = int(p.x < 0.0);
    int y_neg = int(p.y < 0.0);
    int q = (x_neg + y_neg) | (y_neg << 1);
    return q;
}

float xpm1(vec2 p) {
    bool x_neg = (p.x < 0.0);
    bool y_neg = (p.y < 0.0);
    float lon = atan(abs(p.y), abs(p.x));
    float x02 = lon * FOUR_OVER_PI;
    if (x_neg != y_neg) { // Could be replaced by a sign copy from (x_neg ^ y_neg) << 32
        return 1.0 - x02;
    } else {
        return x02 - 1.0;
    }
}

float one_minus_z_pos(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256

    if (d2 < 1e-1) { // <=> dec > 84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return 1.0f - p.z;
}

float one_minus_z_neg(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256
    if (d2 < 1e-1f) { // <=> dec < -84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return p.z + 1.0;
}

int ij2z(int i, int j) {
    int i4 = i | (j << 2);

    int j4 = (i4 ^ (i4 >> 1)) & 0x22222222;
    int i5 = i4 ^ j4 ^ (j4 << 1);

    return i5;
}

struct HashDxDy {
    int idx;
    float dx;
    float dy;
};

uniform sampler2D ang2pixd;
HashDxDy hash_with_dxdy2(vec2 radec) {
    vec2 aa = vec2(radec.x/TWICE_PI + 1.0, (radec.y/PI) + 0.5);
    vec3 v = texture(ang2pixd, aa).rgb;
    return HashDxDy(
        int(v.x * 255.0),
        v.y,
        v.z
    );
}
HashDxDy hash_with_dxdy(int depth, vec3 p) {
    
    int nside = 1 << depth;
    float half_nside = float(nside) * 0.5;

    float x_pm1 = xpm1(p.xy);
    int q = quarter(p.xy);

    int d0h = 0;
    vec2 p_proj = vec2(0.0);
    if (p.z > TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_pos(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, 2.0 - sqrt_3_one_min_z);
        d0h = q;
    } else if (p.z < -TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_neg(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, sqrt_3_one_min_z);
        d0h = q + 8;
    } else {
        float y_pm1 = p.z * TRANSITION_Z_INV;
        int q01 = int(x_pm1 > y_pm1);  // 0/1
        int q12 = int(x_pm1 >= -y_pm1); // 0\1
        int q03 = 1 - q12; // 1\0
        int q1 = q01 & q12; // = 1 if q1, 0 else
        p_proj = vec2(
            x_pm1 - float(q01 + q12 - 1),
            y_pm1 + float(q01 + q03)
        );
        d0h = ((q01 + q03) << 2) + ((q + q1) & 3);
    }

    float x = (half_nside * (p_proj.x + p_proj.y));
    float y = (half_nside * (p_proj.y - p_proj.x));
    int i = int(x);
    int j = int(y);

    return HashDxDy(
        (d0h << (depth << 1)) + ij2z(i, j),
        x - float(i),
        y - float(j)
    );
}
vec3 xyz2uv(vec3 xyz) {
    HashDxDy result = hash_with_dxdy(0, xyz.zxy);

    int idx = result.idx;
    vec2 offset = vec2(result.dy, result.dx);
    Tile tile = textures_tiles[idx];

    return vec3(offset, float(tile.texture_idx));
}
uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

void main() {
    vec3 uv = xyz2uv(normalize(frag_pos));

    uv.y = 1.0 - uv.y;
    vec4 c = uvw2c_i32(uv);

    out_frag_color = c;
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips3d_i32.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler3D;
precision highp isampler3D;
precision highp usampler3D;

uniform sampler3D tex;

in vec3 frag_uv;

out vec4 out_frag_color;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

uniform float opacity;

void main() {
    vec3 uv = vec3(frag_uv.xyz);
    uv.y = 1.0 - uv.y;

    vec4 color = uvw2c_i32(uv);

    out_frag_color = color;
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips_raytracer_rgba2cmap.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;
precision highp sampler2DArray;
precision highp isampler2DArray;
precision highp int;

in vec3 frag_pos;
in vec2 out_clip_pos;
out vec4 out_frag_color;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[12];

uniform float opacity;
uniform sampler2DArray tex;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}
const float TWICE_PI = 6.28318530718;
const float PI = 3.141592653589793;
const float FOUR_OVER_PI = 1.27323954474;
const float TRANSITION_Z = 0.66666666666;
const float TRANSITION_Z_INV = 1.5;

int quarter(vec2 p) {
    int x_neg = int(p.x < 0.0);
    int y_neg = int(p.y < 0.0);
    int q = (x_neg + y_neg) | (y_neg << 1);
    return q;
}

float xpm1(vec2 p) {
    bool x_neg = (p.x < 0.0);
    bool y_neg = (p.y < 0.0);
    float lon = atan(abs(p.y), abs(p.x));
    float x02 = lon * FOUR_OVER_PI;
    if (x_neg != y_neg) { // Could be replaced by a sign copy from (x_neg ^ y_neg) << 32
        return 1.0 - x02;
    } else {
        return x02 - 1.0;
    }
}

float one_minus_z_pos(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256

    if (d2 < 1e-1) { // <=> dec > 84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return 1.0f - p.z;
}

float one_minus_z_neg(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256
    if (d2 < 1e-1f) { // <=> dec < -84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return p.z + 1.0;
}

int ij2z(int i, int j) {
    int i4 = i | (j << 2);

    int j4 = (i4 ^ (i4 >> 1)) & 0x22222222;
    int i5 = i4 ^ j4 ^ (j4 << 1);

    return i5;
}

struct HashDxDy {
    int idx;
    float dx;
    float dy;
};

uniform sampler2D ang2pixd;
HashDxDy hash_with_dxdy2(vec2 radec) {
    vec2 aa = vec2(radec.x/TWICE_PI + 1.0, (radec.y/PI) + 0.5);
    vec3 v = texture(ang2pixd, aa).rgb;
    return HashDxDy(
        int(v.x * 255.0),
        v.y,
        v.z
    );
}
HashDxDy hash_with_dxdy(int depth, vec3 p) {
    
    int nside = 1 << depth;
    float half_nside = float(nside) * 0.5;

    float x_pm1 = xpm1(p.xy);
    int q = quarter(p.xy);

    int d0h = 0;
    vec2 p_proj = vec2(0.0);
    if (p.z > TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_pos(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, 2.0 - sqrt_3_one_min_z);
        d0h = q;
    } else if (p.z < -TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_neg(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, sqrt_3_one_min_z);
        d0h = q + 8;
    } else {
        float y_pm1 = p.z * TRANSITION_Z_INV;
        int q01 = int(x_pm1 > y_pm1);  // 0/1
        int q12 = int(x_pm1 >= -y_pm1); // 0\1
        int q03 = 1 - q12; // 1\0
        int q1 = q01 & q12; // = 1 if q1, 0 else
        p_proj = vec2(
            x_pm1 - float(q01 + q12 - 1),
            y_pm1 + float(q01 + q03)
        );
        d0h = ((q01 + q03) << 2) + ((q + q1) & 3);
    }

    float x = (half_nside * (p_proj.x + p_proj.y));
    float y = (half_nside * (p_proj.y - p_proj.x));
    int i = int(x);
    int j = int(y);

    return HashDxDy(
        (d0h << (depth << 1)) + ij2z(i, j),
        x - float(i),
        y - float(j)
    );
}
vec3 xyz2uv(vec3 xyz) {
    HashDxDy result = hash_with_dxdy(0, xyz.zxy);

    int idx = result.idx;
    vec2 offset = vec2(result.dy, result.dx);
    Tile tile = textures_tiles[idx];

    return vec3(offset, float(tile.texture_idx));
}

void main() {
    vec3 uv = xyz2uv(normalize(frag_pos));
    vec4 c = uvw2cmap_rgba(uv);

    out_frag_color = c;
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips_raytracer_f32.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;
precision highp int;

in vec3 frag_pos;
in vec2 out_clip_pos;
out vec4 out_frag_color;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[12];

uniform float opacity;
uniform sampler2DArray tex;

struct TileColor {
    Tile tile;
    vec4 color;
    bool found;
};

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}
const float TWICE_PI = 6.28318530718;
const float PI = 3.141592653589793;
const float FOUR_OVER_PI = 1.27323954474;
const float TRANSITION_Z = 0.66666666666;
const float TRANSITION_Z_INV = 1.5;

int quarter(vec2 p) {
    int x_neg = int(p.x < 0.0);
    int y_neg = int(p.y < 0.0);
    int q = (x_neg + y_neg) | (y_neg << 1);
    return q;
}

float xpm1(vec2 p) {
    bool x_neg = (p.x < 0.0);
    bool y_neg = (p.y < 0.0);
    float lon = atan(abs(p.y), abs(p.x));
    float x02 = lon * FOUR_OVER_PI;
    if (x_neg != y_neg) { // Could be replaced by a sign copy from (x_neg ^ y_neg) << 32
        return 1.0 - x02;
    } else {
        return x02 - 1.0;
    }
}

float one_minus_z_pos(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256

    if (d2 < 1e-1) { // <=> dec > 84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return 1.0f - p.z;
}

float one_minus_z_neg(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256
    if (d2 < 1e-1f) { // <=> dec < -84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return p.z + 1.0;
}

int ij2z(int i, int j) {
    int i4 = i | (j << 2);

    int j4 = (i4 ^ (i4 >> 1)) & 0x22222222;
    int i5 = i4 ^ j4 ^ (j4 << 1);

    return i5;
}

struct HashDxDy {
    int idx;
    float dx;
    float dy;
};

uniform sampler2D ang2pixd;
HashDxDy hash_with_dxdy2(vec2 radec) {
    vec2 aa = vec2(radec.x/TWICE_PI + 1.0, (radec.y/PI) + 0.5);
    vec3 v = texture(ang2pixd, aa).rgb;
    return HashDxDy(
        int(v.x * 255.0),
        v.y,
        v.z
    );
}
HashDxDy hash_with_dxdy(int depth, vec3 p) {
    
    int nside = 1 << depth;
    float half_nside = float(nside) * 0.5;

    float x_pm1 = xpm1(p.xy);
    int q = quarter(p.xy);

    int d0h = 0;
    vec2 p_proj = vec2(0.0);
    if (p.z > TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_pos(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, 2.0 - sqrt_3_one_min_z);
        d0h = q;
    } else if (p.z < -TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_neg(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, sqrt_3_one_min_z);
        d0h = q + 8;
    } else {
        float y_pm1 = p.z * TRANSITION_Z_INV;
        int q01 = int(x_pm1 > y_pm1);  // 0/1
        int q12 = int(x_pm1 >= -y_pm1); // 0\1
        int q03 = 1 - q12; // 1\0
        int q1 = q01 & q12; // = 1 if q1, 0 else
        p_proj = vec2(
            x_pm1 - float(q01 + q12 - 1),
            y_pm1 + float(q01 + q03)
        );
        d0h = ((q01 + q03) << 2) + ((q + q1) & 3);
    }

    float x = (half_nside * (p_proj.x + p_proj.y));
    float y = (half_nside * (p_proj.y - p_proj.x));
    int i = int(x);
    int j = int(y);

    return HashDxDy(
        (d0h << (depth << 1)) + ij2z(i, j),
        x - float(i),
        y - float(j)
    );
}
vec3 xyz2uv(vec3 xyz) {
    HashDxDy result = hash_with_dxdy(0, xyz.zxy);

    int idx = result.idx;
    vec2 offset = vec2(result.dy, result.dx);
    Tile tile = textures_tiles[idx];

    return vec3(offset, float(tile.texture_idx));
}

void main() {
    vec3 uv = xyz2uv(normalize(frag_pos));

    uv.y = 1.0 - uv.y;
    vec4 c = uvw2c_f32(uv);


    out_frag_color = c;
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"image_sampler.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2D;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

void main() {
    out_frag_color = texture(tex, frag_uv);
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"catalogs_healpix.vert",
        r#"#version 300 es
precision highp float;

layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 center;

uniform float current_time;
uniform mat3 inv_model;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform vec2 kernel_size;

out vec2 out_uv;
out vec3 out_p;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}


void main() {
    vec3 p = inv_model * center;

    vec2 center_pos_clip_space = world2clip_healpix(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * czf)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}"#,
    );
    out.insert(
        r"passes_post_fragment_100es.frag",
        r#"#version 300 es
precision highp float;

in vec2 v_tc;
out vec4 color;

uniform sampler2D fbo_tex;

vec3 srgb_from_linear(vec3 rgb) {
    bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
    vec3 lower = rgb * vec3(3294.6);
    vec3 higher = vec3(269.025) * pow(rgb, vec3(1.0 / 2.4)) - vec3(14.025);
    return mix(higher, lower, vec3(cutoff));
}

vec4 srgba_from_linear(vec4 rgba) {
    return vec4(srgb_from_linear(rgba.rgb), 255.0 * rgba.a);
}

void main() {
    color = texture(fbo_tex, v_tc);

}"#,
    );
    out.insert(
        r"fits_i16.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp int;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}





vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uv2c_f32(vec2 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uv2c_i32(vec2 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return val2c(val);
}

vec4 uv2c_i16(vec2 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return val2c(val);
}

vec4 uv2c_u8(vec2 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

void main() {
    vec2 uv = frag_uv;
    uv.y = 1.0 - uv.y;

    out_frag_color = uv2c_i16(frag_uv);
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips3d_raster.vert",
        r#"#version 300 es
precision highp float;

layout (location = 0) in vec2 lonlat;
layout (location = 1) in vec3 uv;

out vec3 frag_uv;

uniform mat3 inv_model;
uniform vec2 ndc_to_clip;
uniform float czf;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}

void main() {
    vec3 p_xyz = lonlat2xyz(lonlat);
    vec3 p_w = inv_model * p_xyz; 
    vec2 p_clip = proj(p_w.xyz);

    vec2 p_ndc = p_clip / (ndc_to_clip * czf);
    gl_Position = vec4(p_ndc, 0.0, 1.0);

    frag_uv = uv;
}"#,
    );
    out.insert(
        r"line_inst_lonlat.vert",
        r#"#version 300 es
precision highp float;
layout (location = 0) in vec2 p_a_lonlat;
layout (location = 1) in vec2 p_b_lonlat;
layout (location = 2) in vec2 vertex;

uniform mat3 u_2world;
uniform vec2 ndc_to_clip;
uniform float czf;
uniform float u_width;
uniform float u_height;
uniform float u_thickness;

out vec2 l;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}

void main() {
    vec3 p_a_xyz = lonlat2xyz(p_a_lonlat);
    vec3 p_b_xyz = lonlat2xyz(p_b_lonlat);
    vec3 p_a_w = u_2world * p_a_xyz; 
    vec3 p_b_w = u_2world * p_b_xyz;
    vec2 p_a_clip = proj(p_a_w);
    vec2 p_b_clip = proj(p_b_w);

    vec2 da = p_a_clip - p_b_clip;

    vec2 p_a_ndc = p_a_clip / (ndc_to_clip * czf);
    vec2 p_b_ndc = p_b_clip / (ndc_to_clip * czf);

    vec2 x_b = p_b_ndc - p_a_ndc;
    vec2 y_b = normalize(vec2(-x_b.y, x_b.x));

    float ndc2pix = 2.0 / u_width;

    vec2 p_ndc_x = x_b * vertex.x;
    vec2 p_ndc_y = (u_thickness + 2.0) * y_b * vertex.y * vec2(1.0, u_width/u_height) * ndc2pix;

    vec2 p_ndc = p_a_ndc + p_ndc_x + p_ndc_y;
    gl_Position = vec4(p_ndc, 0.f, 1.f);

    l = vec2(dot(da, da), vertex.y);
}"#,
    );
    out.insert(
        r"line_inst_ndc.vert",
        r#"#version 300 es
precision highp float;
layout (location = 0) in vec2 p_a;
layout (location = 1) in vec2 p_b;
layout (location = 2) in vec2 vertex;

out vec2 l;

uniform float u_width;
uniform float u_height;
uniform float u_thickness;

void main() {
    vec2 x_b = p_b - p_a;
    vec2 y_b = normalize(vec2(-x_b.y, x_b.x));

    float ndc2pix = 2.0 / u_width;

    vec2 p = p_a + x_b * vertex.x + (u_thickness + 2.0) * y_b * vertex.y * vec2(1.0, u_width/u_height) * ndc2pix;
    gl_Position = vec4(p, 0.f, 1.f);
    l = vec2(0.0, vertex.y);
}"#,
    );
    out.insert(
        r"fits_f32.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp int;

out vec4 out_frag_color;
in vec2 frag_uv;

uniform sampler2D tex;
uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}





vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uv2c_f32(vec2 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uv2c_i32(vec2 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return val2c(val);
}

vec4 uv2c_i16(vec2 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return val2c(val);
}

vec4 uv2c_u8(vec2 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

void main() {
    vec2 uv = frag_uv;
    uv.y = 1.0 - uv.y;

    out_frag_color = uv2c_f32(frag_uv);
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips_rasterizer_i32.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray tex;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

uniform float opacity;

void main() {
    vec3 uv0 = frag_uv_start;
    vec3 uv1 = frag_uv_end;
    uv0.y = 1.0 - uv0.y;
    uv1.y = 1.0 - uv1.y;

    vec4 color_start = uvw2c_i32(uv0);
    vec4 color_end = uvw2c_i32(uv1);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips_rasterizer_u8.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;

uniform sampler2DArray tex;

in vec3 frag_uv_start;
in vec3 frag_uv_end;
in float frag_blending_factor;

out vec4 out_frag_color;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

uniform float opacity;

void main() {
    vec3 uv0 = frag_uv_start;
    vec3 uv1 = frag_uv_end;
    uv0.y = 1.0 - uv0.y;
    uv1.y = 1.0 - uv1.y;

    vec4 color_start = uvw2c_u8(uv0);
    vec4 color_end = uvw2c_u8(uv1);

    out_frag_color = mix(color_start, color_end, frag_blending_factor);
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"hips_raytracer_backcolor.frag",
        r#"#version 300 es
precision highp float;
precision highp int;

out vec4 out_frag_color;

uniform vec3 color;

void main() {
    out_frag_color = vec4(color, 1.0);
}"#,
    );
    out.insert(
        r"hips_raytracer_u8.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2DArray;
precision highp usampler2DArray;
precision highp isampler2DArray;
precision highp int;

uniform sampler2DArray tex;

in vec3 frag_pos;
in vec2 out_clip_pos;
out vec4 out_frag_color;

struct Tile {
    int uniq; // Healpix cell
    int texture_idx; // Index in the texture buffer
    float start_time; // Absolute time that the load has been done in ms
    float empty;
};

uniform Tile textures_tiles[12];

uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}
const float TWICE_PI = 6.28318530718;
const float PI = 3.141592653589793;
const float FOUR_OVER_PI = 1.27323954474;
const float TRANSITION_Z = 0.66666666666;
const float TRANSITION_Z_INV = 1.5;

int quarter(vec2 p) {
    int x_neg = int(p.x < 0.0);
    int y_neg = int(p.y < 0.0);
    int q = (x_neg + y_neg) | (y_neg << 1);
    return q;
}

float xpm1(vec2 p) {
    bool x_neg = (p.x < 0.0);
    bool y_neg = (p.y < 0.0);
    float lon = atan(abs(p.y), abs(p.x));
    float x02 = lon * FOUR_OVER_PI;
    if (x_neg != y_neg) { // Could be replaced by a sign copy from (x_neg ^ y_neg) << 32
        return 1.0 - x02;
    } else {
        return x02 - 1.0;
    }
}

float one_minus_z_pos(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256

    if (d2 < 1e-1) { // <=> dec > 84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return 1.0f - p.z;
}

float one_minus_z_neg(vec3 p) {
    float d2 = dot(p.xy, p.xy); // z = sqrt(1 - d2) AND sqrt(1 - x) = 1 - x / 2 - x^2 / 8 - x^3 / 16 - 5 x^4/128 - 7 * x^5/256
    if (d2 < 1e-1f) { // <=> dec < -84.27 deg
        return d2 * (0.5 + d2 * (0.125 + d2 * (0.0625 + d2 * (0.0390625 + d2 * 0.02734375))));
    }
    return p.z + 1.0;
}

int ij2z(int i, int j) {
    int i4 = i | (j << 2);

    int j4 = (i4 ^ (i4 >> 1)) & 0x22222222;
    int i5 = i4 ^ j4 ^ (j4 << 1);

    return i5;
}

struct HashDxDy {
    int idx;
    float dx;
    float dy;
};

uniform sampler2D ang2pixd;
HashDxDy hash_with_dxdy2(vec2 radec) {
    vec2 aa = vec2(radec.x/TWICE_PI + 1.0, (radec.y/PI) + 0.5);
    vec3 v = texture(ang2pixd, aa).rgb;
    return HashDxDy(
        int(v.x * 255.0),
        v.y,
        v.z
    );
}
HashDxDy hash_with_dxdy(int depth, vec3 p) {
    
    int nside = 1 << depth;
    float half_nside = float(nside) * 0.5;

    float x_pm1 = xpm1(p.xy);
    int q = quarter(p.xy);

    int d0h = 0;
    vec2 p_proj = vec2(0.0);
    if (p.z > TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_pos(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, 2.0 - sqrt_3_one_min_z);
        d0h = q;
    } else if (p.z < -TRANSITION_Z) {
        float sqrt_3_one_min_z = sqrt(3.0 * one_minus_z_neg(p));
        p_proj = vec2(x_pm1 * sqrt_3_one_min_z, sqrt_3_one_min_z);
        d0h = q + 8;
    } else {
        float y_pm1 = p.z * TRANSITION_Z_INV;
        int q01 = int(x_pm1 > y_pm1);  // 0/1
        int q12 = int(x_pm1 >= -y_pm1); // 0\1
        int q03 = 1 - q12; // 1\0
        int q1 = q01 & q12; // = 1 if q1, 0 else
        p_proj = vec2(
            x_pm1 - float(q01 + q12 - 1),
            y_pm1 + float(q01 + q03)
        );
        d0h = ((q01 + q03) << 2) + ((q + q1) & 3);
    }

    float x = (half_nside * (p_proj.x + p_proj.y));
    float y = (half_nside * (p_proj.y - p_proj.x));
    int i = int(x);
    int j = int(y);

    return HashDxDy(
        (d0h << (depth << 1)) + ij2z(i, j),
        x - float(i),
        y - float(j)
    );
}
vec3 xyz2uv(vec3 xyz) {
    HashDxDy result = hash_with_dxdy(0, xyz.zxy);

    int idx = result.idx;
    vec2 offset = vec2(result.dy, result.dx);
    Tile tile = textures_tiles[idx];

    return vec3(offset, float(tile.texture_idx));
}

void main() {
    vec3 uv = xyz2uv(normalize(frag_pos));

    uv.y = 1.0 - uv.y;
    vec4 c = uvw2c_u8(uv);


    out_frag_color = c;
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"catalogs_aitoff.vert",
        r#"#version 300 es
precision highp float;
layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 center;

uniform float current_time;
uniform mat3 inv_model;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform vec2 kernel_size;

out vec2 out_uv;
out vec3 out_p;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}

void main() {
    vec3 p = inv_model * center;

    vec2 center_pos_clip_space = world2clip_aitoff(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * czf)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}"#,
    );
    out.insert(
        r"moc_base.frag",
        r#"#version 300 es

precision highp float;
out vec4 color;

uniform vec4 u_color;

void main() {
    color = u_color;
}"#,
    );
    out.insert(
        r"colormaps_colormap.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler2D;

in vec2 out_uv;
out vec4 color;

uniform sampler2D texture_fbo;
uniform float alpha;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}

void main() {
    float opacity = texture(texture_fbo, out_uv).r;

    float o = smoothstep(0.0, 0.1, opacity);

    color = colormap_f(opacity);
    color.a = o * alpha;
}"#,
    );
    out.insert(
        r"hips_raytracer_raytracer.vert",
        r#"#version 300 es
precision highp float;
precision highp int;

layout (location = 0) in vec2 pos_clip_space;
layout (location = 1) in vec3 pos_world_space;

out vec2 out_clip_pos;
out vec3 frag_pos;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform mat3 model;

void main() {
    vec2 uv = pos_clip_space * 0.5 + 0.5;

    frag_pos = model * pos_world_space;

    gl_Position = vec4(pos_clip_space / (ndc_to_clip * czf), 0.0, 1.0);
    out_clip_pos = pos_clip_space;
}"#,
    );
    out.insert(
        r"catalogs_ortho.vert",
        r#"#version 300 es
precision highp float;
layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 center;

uniform float current_time;
uniform mat3 inv_model;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform vec2 kernel_size;

out vec2 out_uv;
out vec3 out_p;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}


void main() {
    vec3 p = inv_model * center;

    vec2 center_pos_clip_space = world2clip_orthographic(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * czf)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}"#,
    );
    out.insert(
        r"catalogs_mollweide.vert",
        r#"#version 300 es
precision highp float;
layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 center;

uniform float current_time;
uniform mat3 inv_model;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform vec2 kernel_size;

out vec2 out_uv;
out vec3 out_p;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}


void main() {
    vec3 p = inv_model * center;

    vec2 center_pos_clip_space = world2clip_mollweide(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * czf)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}"#,
    );
    out.insert(
        r"catalogs_catalog.frag",
        r#"#version 300 es
precision highp float;

in vec2 out_uv;
in vec3 out_p;

out vec4 color;

uniform sampler2D kernel_texture;
uniform float max_density; // max number of sources in a kernel sized HEALPix cell at the current depth
uniform float fov;
uniform float strength;
void main() {
    color = texture(kernel_texture, out_uv) / max(log2(fov*100.0), 1.0);
    color.r *= strength;
}"#,
    );
    out.insert(
        r"hips_raytracer_backcolor.vert",
        r#"#version 300 es
precision highp float;
precision highp int;

layout (location = 0) in vec2 pos_clip_space;

uniform vec2 ndc_to_clip;
uniform float czf;

void main() {
    gl_Position = vec4(pos_clip_space / (ndc_to_clip * czf), 0.0, 1.0);
}"#,
    );
    out.insert(
        r"fits_base.vert",
        r#"#version 300 es
precision highp float;
precision highp int;

layout (location = 0) in vec2 ndc_pos;
layout (location = 1) in vec2 uv;

out vec2 frag_uv;

void main() {
    gl_Position = vec4(ndc_pos, 0.0, 1.0);
    frag_uv = uv;
}"#,
    );
    out.insert(
        r"catalogs_mercator.vert",
        r#"#version 300 es
precision highp float;
layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 center;

uniform float current_time;
uniform mat3 inv_model;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform vec2 kernel_size;

out vec2 out_uv;
out vec3 out_p;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}


void main() {
    vec3 p = inv_model * center;

    vec2 center_pos_clip_space = world2clip_mercator(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * czf)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}"#,
    );
    out.insert(
        r"hips3d_i16.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler3D;
precision highp isampler3D;
precision highp usampler3D;

uniform sampler3D tex;

in vec3 frag_uv;

out vec4 out_frag_color;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

uniform float opacity;

void main() {
    vec3 uv = vec3(frag_uv.xyz);
    uv.y = 1.0 - uv.y;

    vec4 color = uvw2c_i16(uv);

    out_frag_color = color;
    out_frag_color.a = out_frag_color.a * opacity;
}"#,
    );
    out.insert(
        r"catalogs_arc.vert",
        r#"#version 300 es
precision highp float;
layout (location = 0) in vec2 offset;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 center;

uniform float current_time;
uniform mat3 inv_model;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform vec2 kernel_size;

out vec2 out_uv;
out vec3 out_p;

const float PI = 3.141592653589793;
const float SQRT_2 = 1.41421356237309504880168872420969808;

vec2 w2c_sin(vec3 p) {
    vec2 q = vec2(-p.x, p.y);
    return p.z >= 0.f ? q : normalize(q);
}

vec2 w2c_sin_no_backface(vec3 p) {
    return vec2(-p.x, p.y);
}

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
const float eps = 1.25e-8;
const int n_iter = 100;

float newton_solve(float z) {
    float cte = PI * z;
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
vec2 w2c_tan(vec3 p) {
    p.z = max(p.z, 1e-2);
    return vec2(-p.x, p.y) / (p.z*PI);
}
vec2 w2c_stg(vec3 p) {
    float w = (1.0 + p.z) * 0.5;
    return vec2(-p.x, p.y) / (PI * w);
}
vec2 w2c_zea(vec3 p) {
    float w = sqrt(0.5 + 0.5 * p.z); // <=> sqrt[(1 + x) / 2]
    return vec2(-p.x, p.y) * 0.5 / w;
}
vec2 w2c_mer(vec3 p) {
    return vec2(atan(-p.x, p.z), atanh(p.y)) / PI;
}

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
        return w2c_tan(p);
    } else if (u_proj == 1) {
        return w2c_stg(p);
    } else if (u_proj == 2) {
        return w2c_sin(p);
    } else if (u_proj == 3) {
        return w2c_zea(p);
    } else if (u_proj == 4) {
        return w2c_ait(p);
    } else if (u_proj == 5) {
        return w2c_mol(p);
    } else {
        return w2c_mer(p);
    }
}

void main() {
    vec3 p = inv_model * center;

    vec2 center_pos_clip_space = world2clip_arc(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * czf)) + offset * kernel_size , 0.f, 1.f);

    out_uv = uv;
    out_p = p;
}"#,
    );
    out.insert(
        r"colormaps_colormap.vert",
        r#"#version 300 es
precision highp float;
precision highp sampler2D;

layout (location = 0) in vec2 position;
layout (location = 1) in vec2 uv;

out vec2 out_uv;

void main() {
    gl_Position = vec4(position, 0.f, 1.f);
    out_uv = uv;
}"#,
    );
    out.insert(
        r"hips3d_red.frag",
        r#"#version 300 es
precision highp float;
precision highp sampler3D;
precision highp isampler3D;
precision highp usampler3D;

uniform sampler3D tex;

in vec3 frag_uv;

out vec4 out_frag_color;
uniform float opacity;

uniform float scale;
uniform float offset;
uniform float blank;
uniform float min_value;
uniform float max_value;
uniform int H;
uniform float reversed;

uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
float linear_f(float x, float min_value, float max_value) {
    return clamp((x - min_value)/(max_value - min_value), 0.0, 1.0);
}

float sqrt_f(float x, float min_value, float max_value) {
    float a = linear_f(x, min_value, max_value);
    return sqrt(a);
}

float log_f(float x, float min_value, float max_value) {
    float y = linear_f(x, min_value, max_value);
    float a = 1000.0;
    return log(a*y + 1.0)/log(a);
}

float asinh_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return asinh(10.0*d)/3.0;
}

float pow2_f(float x, float min_value, float max_value) {
    float d = linear_f(x, min_value, max_value);
    return d*d;
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}

uniform float k_gamma;
uniform float k_saturation;
uniform float k_contrast;
uniform float k_brightness;
uniform float k_exposure;

vec4 apply_gamma(vec4 ic, float g) {
    float new_r = pow(ic.r, g);
    float new_g = pow(ic.g, g);
    float new_b = pow(ic.b, g);

    return vec4(new_r, new_g, new_b, ic.a);
}

vec4 apply_saturation(vec4 color, float value) {
    const vec3 luminosity_factor = vec3(0.2126, 0.7152, 0.0722);
    vec3 grayscale = vec3(dot(color.rgb, luminosity_factor));
    
    return vec4(mix(grayscale, color.rgb, 1.0 + value), color.a);
}

vec4 apply_contrast(vec4 color, float value) {
    return vec4(0.5 + (1.0 + value) * (color.rgb - 0.5), color.a);
}

vec4 apply_brightness(vec4 color, float value) {
    return vec4(color.rgb + value, color.a);
}

vec4 apply_exposure(vec4 color, float value) {
    return vec4((1.0 + value) * color.rgb, color.a);
}

vec4 apply_tonal(vec4 color) {
  return apply_gamma(
    apply_saturation(
      apply_contrast(
        apply_brightness(color, k_brightness),
        k_contrast
      ),
      k_saturation
    ),
    k_gamma
  );
}
vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}




vec4 uvw2c_r(vec3 uv) {    
    vec2 va = texture(tex, uv).ra;

    va.x = transfer_func(H, va.x, min_value, max_value);

    va.x = mix(va.x, 1.0 - va.x, reversed);

    vec4 c = colormap_f(va.x);
    return apply_tonal(c);
}

vec4 uvw2c_rgba(vec3 uv) {
    vec4 c = texture(tex, uv).rgba;

    c.r = transfer_func(H, c.r, min_value, max_value);
    c.g = transfer_func(H, c.g, min_value, max_value);
    c.b = transfer_func(H, c.b, min_value, max_value);

    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 uvw2c_ra(vec3 uv) {
    vec2 c = texture(tex, uv).rg;

    c.r = transfer_func(H, c.r, min_value, max_value);

    c.r = mix(c.r, 1.0 - c.r, reversed);

    vec3 color = colormap_f(c.r).rgb;

    return apply_tonal(vec4(color, c.g));
}

vec4 uvw2cmap_rgba(vec3 uv) {    
    float v = texture(tex, uv).r;
    v = transfer_func(H, v, min_value, max_value);
    vec4 c = colormap_f(v);
    c.rgb = mix(c.rgb, 1.0 - c.rgb, reversed);

    return apply_tonal(c);
}

vec4 val2c_f32(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(isinf(x) || isnan(x)));
    return apply_tonal(new_color);
}

vec4 val2c(float x) {
    float alpha = x * scale + offset;
    alpha = transfer_func(H, alpha, min_value, max_value);

    alpha = mix(alpha, 1.0 - alpha, reversed);

    vec4 new_color = mix(colormap_f(alpha), vec4(0.0), float(x == blank));
    return apply_tonal(new_color);
}

vec4 uvw2c_f32(vec3 uv) {
    float val = decode_f32(texture(tex, uv).rgba*255.0);
    return val2c_f32(val);
}

vec4 uvw2c_i32(vec3 uv) {
    float val = float(decode_i32(texture(tex, uv).rgba));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_i16(vec3 uv) {
    float val = float(decode_i16(texture(tex, uv).rg));
    return mix(val2c(val), vec4(0.0), float(val == -1.0));
}

vec4 uvw2c_u8(vec3 uv) {
    float val = float(decode_u8(texture(tex, uv).r));
    return val2c(val);
}

void main() {
    vec3 uv = vec3(frag_uv.xyz);
    vec4 color = uvw2c_ra(uv);

    out_frag_color = color;
    out_frag_color.a = opacity * out_frag_color.a;
}"#,
    );
    out
}
