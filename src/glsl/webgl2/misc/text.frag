#version 300 es
precision lowp float;
precision lowp sampler2DArray;

uniform vec4 text_color;
uniform sampler2DArray font_textures;

in vec3 out_uv;
out vec4 color;

void main() {
    vec3 uv = vec3(out_uv.x, 1.f - out_uv.y, out_uv.z);
    vec4 mask = texture(font_textures, uv);
    color = text_color * mask;
    //color = vec4(1.0, 0.0, 0.0, 1.0);
}