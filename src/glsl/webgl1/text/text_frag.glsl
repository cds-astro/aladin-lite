precision highp float;

varying vec4 v_rgba;
varying vec2 v_tc;
varying vec4 color;

uniform sampler2D u_sampler_font;

void main() {
    // The texture is set up with `SRGB8_ALPHA8`, so no need to decode here!
    float alpha = texture2D(u_sampler_font, v_tc).r;

    // Multiply vertex color with texture color (in linear space).
    // Linear color is written and blended in Framebuffer and converted to sRGB later
    gl_FragColor = v_rgba * alpha;
    gl_FragColor.a = alpha;
}