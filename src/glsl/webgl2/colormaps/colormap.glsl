uniform sampler2D colormaps;
uniform float num_colormaps;
uniform float colormap_id;
// can be either 0 or 1

vec4 colormap_f(float x) {
    float id = (colormap_id + 0.5) / num_colormaps;
    return texture(colormaps, vec2(x, id));
}
