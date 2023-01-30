precision lowp float;

varying vec2 out_uv;
varying vec3 out_p;

uniform sampler2D kernel_texture;
uniform float fov;
uniform float strength;
void main() {
    if (out_p.z < 0.0) {
        discard;
    }

    vec4 color = texture2D(kernel_texture, out_uv).rgba / max(log2(fov*100.0), 1.0);
    color.r *= strength;

    gl_FragColor = color;
}