// This file apply the tonal corrections given in this article:
// https://timseverien.com/posts/2020-06-19-colour-correction-with-webgl/

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
    // https://www.w3.org/TR/WCAG21/#dfn-relative-luminance
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
