precision lowp float;
precision lowp sampler2D;

attribute vec2 position;
attribute vec2 uv;

varying vec2 out_uv;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    out_uv = uv;
}