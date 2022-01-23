precision lowp float;

varying vec2 out_uv;
varying vec3 out_p;

uniform sampler2D kernel_texture;
uniform float fov;
uniform float strength;
void main() {
    vec4 color = texture(kernel_texture, out_uv) / max(log2(fov*100.0), 1.0);
    color.r *= strength;

    gl_FragColor = color;
}