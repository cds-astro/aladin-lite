precision lowp float;
precision lowp sampler2D;

varying vec2 out_uv;

uniform sampler2D texture_fbo;
uniform float alpha;

@import ./colormap;

void main() {
    float opacity = texture2D(texture_fbo, out_uv).r;

    float o = smoothstep(0.0, 0.1, opacity);

    vec4 color = colormap_f(opacity);
    color.a = o * alpha;

    gl_FragColor = color;
}