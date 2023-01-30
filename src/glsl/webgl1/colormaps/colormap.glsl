uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;
// can be either 0 or 1
uniform float reversed;

vec4 colormap_f(float x) {
    x = mix(x, 1.0 - x, reversed);
    float id = (colormap_id + 0.5) / num_colormaps;

    return texture2D(colormaps, vec2(x, id));
}
