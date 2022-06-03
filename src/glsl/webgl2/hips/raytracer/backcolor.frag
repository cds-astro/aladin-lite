#version 300 es
precision highp float;
precision highp sampler2D;
precision highp usampler2D;
precision highp isampler2D;
precision highp int;

out vec4 out_frag_color;

void main() {
    out_frag_color = vec4(0.19215686274, 0.49019607843, 0.55294117647, 1.0);
}