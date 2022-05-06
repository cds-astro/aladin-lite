#version 300 es
precision highp float;

in vec4 v_rgba;
in vec2 v_tc;
out vec4 color;

uniform sampler2D u_sampler_font;

void main() {
    // The texture is set up with `SRGB8_ALPHA8`, so no need to decode here!
    float alpha = texture(u_sampler_font, v_tc).r;

    // Multiply vertex color with texture color (in linear space).
    // Linear color is written and blended in Framebuffer and converted to sRGB later
    color = v_rgba * alpha;
    color.a = alpha;
}