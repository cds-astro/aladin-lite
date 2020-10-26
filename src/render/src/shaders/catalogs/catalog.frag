#version 300 es
precision lowp float;

in vec2 out_uv;
in vec3 out_p;

out vec4 color;

uniform sampler2D kernel_texture;
uniform float max_density; // max number of sources in a kernel sized HEALPix cell at the current depth
uniform float fov;
void main() {
    /*if (out_p.z < 0.f) {
        discard;
    }*/

    //color += texture(kernel_texture, out_uv) / log2(max_density + 1.0);
    color += texture(kernel_texture, out_uv) / max(log2(fov*100.0), 1.0);
}