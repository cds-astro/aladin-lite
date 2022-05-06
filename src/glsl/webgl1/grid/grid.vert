precision mediump float;

attribute vec2 position;

varying vec2 pos_clip;

uniform vec2 ndc_to_clip;
uniform float czf;

void main() {
    pos_clip = position * (ndc_to_clip * czf);

    gl_Position = vec4(position, 0.0, 1.0);
}