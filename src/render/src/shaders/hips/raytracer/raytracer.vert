#version 300 es
precision highp float;
precision lowp sampler2DArray;
precision highp int;

layout (location = 0) in vec2 pos_clip_space;
layout (location = 1) in vec3 pos_world_space;

out vec3 out_vert_pos;
out vec2 pos_clip;

uniform mat4 model;
uniform vec2 ndc_to_clip;
uniform float clip_zoom_factor;


const mat4 gal_to_icrs = mat4(
    -0.4838350155487132, 
    0.7469822444972189, 
    0.4559837761750669,
    0.0,

    -0.0548755604162154,
    0.4941094278755837, 
    -0.8676661490190047,
    0.0,

    -0.8734370902348850, 
    -0.4448296299600112, 
    -0.1980763734312015,
    0.0,



    0.0,
    0.0,
    0.0,
    1.0
);
void main() {
    gl_Position = vec4(pos_clip_space / (ndc_to_clip * clip_zoom_factor), 0.0, 1.0);
    pos_clip = pos_clip_space;
    out_vert_pos = vec3(inverse(gal_to_icrs) * model * vec4(pos_world_space, 1.f));
}