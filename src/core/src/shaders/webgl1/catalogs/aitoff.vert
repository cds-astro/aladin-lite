precision lowp float;

attribute vec2 offset;
attribute vec2 uv;
attribute vec3 center;

uniform float current_time;
uniform mat4 model;
uniform mat4 inv_model;

uniform vec2 ndc_to_clip;
uniform float czf;
uniform vec2 kernel_size;

varying vec2 out_uv;
varying vec3 out_p;

@import ../hips/projection;

void main() {
    vec3 p = vec3(inv_model * vec4(center, 1.0));
    //p = check_inversed_longitude(p);

    vec2 center_pos_clip_space = world2clip_aitoff(p);

    vec2 pos_clip_space = center_pos_clip_space;
    gl_Position = vec4((pos_clip_space / (ndc_to_clip * czf)) + offset * kernel_size , 0.0, 1.0);

    out_uv = uv;
    out_p = p;
}