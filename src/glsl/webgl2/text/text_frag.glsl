#version 300 es
precision highp float;

in vec2 v_tc;
out vec4 color;

uniform sampler2D u_sampler_font;
uniform float u_opacity;
uniform vec3 u_color;

void main() {
    // The texture is set up with `SRGB8_ALPHA8`, so no need to decode here!
    float alpha = texture(u_sampler_font, v_tc).r;
    alpha = smoothstep(0.1, 0.9, alpha);

    // Multiply vertex color with texture color (in linear space).
    // Linear color is written and blended in Framebuffer and converted to sRGB later
    color = vec4(u_color, u_opacity * alpha);
    //color.a = color.a * alpha;
}