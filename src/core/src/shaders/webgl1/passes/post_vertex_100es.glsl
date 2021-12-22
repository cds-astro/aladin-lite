#ifdef WEBGL2
    #version 300 es
    precision mediump float;

    layout (location = 0) in vec2 a_pos;
    out vec2 v_tc;
#else
    precision mediump float;

    attribute vec2 a_pos;
    varying vec2 v_tc;
#endif

void main() {
    gl_Position = vec4(a_pos * 2. - 1., 0.0, 1.0);
    v_tc = a_pos;
}