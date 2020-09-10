#version 300 es
precision lowp float;

layout (location = 0) in vec2 position;

out vec2 pos_clip;

uniform vec2 ndc_to_clip;
uniform float clip_zoom_factor;

void main() {
    pos_clip = position * (ndc_to_clip * clip_zoom_factor);

    gl_Position = vec4(position, 0.0, 1.0);
}