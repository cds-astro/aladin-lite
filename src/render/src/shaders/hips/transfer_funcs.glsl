float linear_f(float x, float min_value, float max_value) {
    if (x < min_value) {
        return 0.0;
    } else if (x > max_value) {
        return 1.0;
    } else {
        return (x - min_value)/(max_value - min_value);
    }
}

float sqrt_f(float x, float min_value, float max_value) {
    if (x < min_value) {
        return 0.0;
    } else if (x > max_value) {
        return 1.0;
    } else {
        return sqrt((x - min_value)/(max_value - min_value));
    }
}

float log_f(float x, float min_value, float max_value) {
    if (x < min_value) {
        return 0.0;
    } else if (x > max_value) {
        return 1.0;
    } else {
        float d = (x - min_value)/(max_value - min_value);
        float a = 1000.0;
        return log(a*d + 1.0)/log(a);
    }
}

float asinh_f(float x, float min_value, float max_value) {
    if (x < min_value) {
        return 0.0;
    } else if (x > max_value) {
        return 1.0;
    } else {
        float d = (x - min_value)/(max_value - min_value);
        return asinh(10.0*d)/3.0;
    }
}

float pow2_f(float x, float min_value, float max_value) {
    if (x < min_value) {
        return 0.0;
    } else if (x > max_value) {
        return 1.0;
    } else {
        float d = (x - min_value)/(max_value - min_value);
        return d*d;
    }
}

float transfer_func(int H, float x, float min_value, float max_value) {
    if (H == 0) {
        return linear_f(x, min_value, max_value);
    } else if (H == 1) {
        return sqrt_f(x, min_value, max_value);
    } else if (H == 2) {
        return log_f(x, min_value, max_value);
    } else if (H == 3) {
        return asinh_f(x, min_value, max_value);
    } else {
        return pow2_f(x, min_value, max_value);
    }
}
