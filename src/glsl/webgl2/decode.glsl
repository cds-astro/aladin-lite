// Utils methods for decoding texture bytes to f32, i32, i16, u8
highp float decode_f32(highp vec4 rgba) {
    highp float Sign = 1.0 - step(128.0,rgba[0])*2.0;
    highp float Exponent = 2.0 * mod(rgba[0],128.0) + step(128.0,rgba[1]) - 127.0; 
    if (abs(Exponent + 127.0) < 1e-3) {
        return 0.0;
    }
    highp float Mantissa = mod(rgba[1],128.0)*65536.0 + rgba[2]*256.0 +rgba[3] + float(0x800000);
    highp float Result =  Sign * exp2(Exponent) * (Mantissa * exp2(-23.0 )); 
    return Result;
}

int decode_i32(vec4 rgba) {
    int r = int(rgba.r * 255.0 + 0.5);
    int g = int(rgba.g * 255.0 + 0.5);
    int b = int(rgba.b * 255.0 + 0.5);
    int a = int(rgba.a * 255.0 + 0.5);

    // GLSL int automatically handle the top-most sign bit (two's complement behaviour)
    int value = (r << 24) | (g << 16) | (b << 8) | a; // Combine into a 16-bit integer

    return value;
}

int decode_i16(vec2 rg) {
    int r = int(rg.r * 255.0 + 0.5);
    int g = int(rg.g * 255.0 + 0.5);

    int value = (r << 8) | g; // Combine into a 16-bit integer

    // Convert from unsigned to signed 16-bit
    if (value >= 32768) {
        value -= 65536;
    }

    return value;
}

uint decode_u8(float r) {
    uint value = uint(r * 255.0 + 0.5);
    return value;
}



