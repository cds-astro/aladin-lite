uniform sampler2D colormaps;
uniform int num_colormaps;
uniform int colormap_id;
// can be either 0 or 1
uniform int reversed;

vec4 colormap_f(float x) {
    x = mix(x, 1.0 - x, float(reversed));
    float id = (float(colormap_id) + 0.5) / float(num_colormaps);

    return texture2D(colormaps, vec2(x, id));
}
