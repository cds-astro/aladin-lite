precision lowp float;

attribute vec2 ndc_pos;

void main() {
    gl_Position = vec4(ndc_pos, 0.0, 1.0);
}