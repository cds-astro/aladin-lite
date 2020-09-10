// Blue & Pastel & Red
float colormap_red(float x) {
    if (x < 0.1131206452846527) {
        return (-9.40943766883858E+02 * x - 1.84146720562529E+02) * x + 3.28713709677420E+01;
    } else if (x < 0.5116005837917328) {
        return 0.0;
    } else if (x < 0.5705677568912506) {
        return (-2.22507913165263E+03 * x + 2.76053354341733E+03) * x - 8.29909138655453E+02;
    } else if (x < 0.622047244) {
        return (-1.84774532967032E+04 * x + 2.30647002747253E+04) * x - 7.12389120879120E+03;
    } else if (x < 0.7922459542751312) {
        return ((((1.29456468589020E+06 * x - 4.64095889653844E+06) * x + 6.62951004830418E+06) * x - 4.71587036142377E+06) * x + 1.67048886368434E+06) * x - 2.35682532934682E+05;
    } else {
        return 3.34889230769210E+02 * x - 1.41006123680226E+02;
    }
}

float colormap_green(float x) {
    if (x < 0.114394336938858) {
        return 0.0;
    } else if (x < 0.4417250454425812) {
        return (9.43393359191585E+02 * x + 1.86774918014536E+02) * x - 3.37113020096108E+01;
    } else if (x < 0.4964917968308496) {
        return 3.11150000000070E+02 * x + 9.54249999999731E+01;
    } else if (x < 0.6259051214039278) {
        return -1.03272635599706E+03 * x + 7.62648586707481E+02;
    } else if (x < 0.8049814403057098) {
        return -2.92799028677160E+02 * x + 2.99524283071235E+02;
    } else {
        return (1.34145201311283E+03 * x - 2.75066701126586E+03) * x + 1.40880802982723E+03;
    }
}

float colormap_blue(float x) {
    if (x < 0.4424893036638088) {
        return 3.09636968527514E+02 * x + 9.62203074056821E+01;
    } else if (x < 0.5) {
        return -4.59921428571535E+02 * x + 4.36741666666678E+02;
    } else if (x < 0.5691165986930345) {
        return -1.81364912280674E+03 * x + 1.05392982456125E+03;
    } else if (x < 0.6279306709766388) {
        return 1.83776470588197E+02 * x - 8.28382352940910E+01;
    } else {
        return ((-1.14087926835422E+04 * x + 2.47091243363548E+04) * x - 1.80428756181930E+04) * x + 4.44421976986281E+03;
    }
}

vec4 bluepastelred_f(float x) {
    float r = clamp(colormap_red(x) / 255.0, 0.0, 1.0);
    float g = clamp(colormap_green(x) / 255.0, 0.0, 1.0);
    float b = clamp(colormap_blue(x) / 255.0, 0.0, 1.0);
    return vec4(r, g, b, 1.0);
}

// Red
float c_red(float x) {
    return 1.448953446096850 * x - 5.02253539008443e-1;
}

float c_green(float x) {
    return 1.889376646180860 * x - 2.272028094820020e2;
}

float c_blue(float x) {
    return 3.92613636363636 * x - 7.46528409090909e+2;
}

vec4 red_f(float x) {
    float t = x * 255.0;
    float r = clamp(c_red(t) / 255.0, 0.0, 1.0);
    float g = clamp(c_green(t) / 255.0, 0.0, 1.0);
    float b = clamp(c_blue(t) / 255.0, 0.0, 1.0);

    return vec4(r, g, b, 1.0);
}
// Gray
vec4 blackw_f(float x) {
    float d = clamp(x, 0.0, 1.0);
    return vec4(d, d, d, 1.0);
}
// IDLCBGnBu
float cbgnbu_red(float x) {
    float v = ((((-2.83671754639782E+03 * x + 6.51753994553536E+03) * x - 5.00110948171466E+03) * x + 1.30359712298773E+03) * x - 2.89958300810074E+02) * x + 2.48458039402758E+02;
    if (v < 8.0) {
        return 8.0;
    } else {
        return v;
    }
}

float cbgnbu_green(float x) {
    return (((((-1.36304822155833E+03 * x + 4.37691418182849E+03) * x - 5.01802019417285E+03) * x + 2.39971481269598E+03) * x - 5.65401491984724E+02) * x - 1.48189675724133E+01) * x + 2.50507618187374E+02;
}

float cbgnbu_blue(float x) {
    if (x < 0.3756393599187693) {
        return (9.62948273917718E+01 * x - 1.96136874142438E+02) * x + 2.41033490809633E+02;
    } else if (x < 0.6215448666633865) {
        return 1.21184043778803E+02 * x + 1.35422939068100E+02;
    } else if (x < 0.8830064316178203) {
        return -1.53052165744713E+02 * x + 3.05873047350666E+02;
    } else {
        return -3.49468965517114E+02 * x + 4.79310344827486E+02;
    }
}

vec4 cbgnbu_f(float x) {
    float r = clamp(cbgnbu_red(x) / 255.0, 0.0, 1.0);
    float g = clamp(cbgnbu_green(x) / 255.0, 0.0, 1.0);
    float b = clamp(cbgnbu_blue(x) / 255.0, 0.0, 1.0);
    return vec4(r, g, b, 1.0);
}
// IDLCBYIGnBu
float CBYIGnBu_red(float x) {
    if (x < 0.2523055374622345) {
        return (-5.80630393656902E+02 * x - 8.20261301968494E+01) * x + 2.53829637096771E+02;
    } else if (x < 0.6267540156841278) {
        return (((-4.07958939010649E+03 * x + 8.13296992114899E+03) * x - 5.30725139102868E+03) * x + 8.58474724851723E+02) * x + 2.03329669375107E+02;
    } else if (x < 0.8763731146612115) {
        return 3.28717357910916E+01 * x + 8.82117255504255E+00;
    } else {
        return -2.29186583577707E+02 * x + 2.38482038123159E+02;
    }
}

float CBYIGnBu_green(float x) {
    if (x < 0.4578040540218353) {
        return ((4.49001704856054E+02 * x - 5.56217473429394E+02) * x + 2.09812296466262E+01) * x + 2.52987561849833E+02;
    } else {
        return ((1.28031059709139E+03 * x - 2.71007279113343E+03) * x + 1.52699334501816E+03) * x - 6.48190622715140E+01;
    }
}

float CBYIGnBu_blue(float x) {
    if (x < 0.1239372193813324) {
        return (1.10092779856059E+02 * x - 3.41564374557536E+02) * x + 2.17553885630496E+02;
    } else if (x < 0.7535201013088226) {
        return ((((3.86204601547122E+03 * x - 8.79126469446648E+03) * x + 6.80922226393264E+03) * x - 2.24007302003438E+03) * x + 3.51344388740066E+02) * x + 1.56774650431396E+02;
    } else {
        return (((((-7.46693234167480E+06 * x + 3.93327773566702E+07) * x - 8.61050867447971E+07) * x + 1.00269040461745E+08) * x - 6.55080846112976E+07) * x + 2.27664953009389E+07) * x - 3.28811994253461E+06;
    }
}

vec4 CBYIGnBu_f(float x) {
    float r = clamp(CBYIGnBu_red(x) / 255.0, 0.0, 1.0);
    float g = clamp(CBYIGnBu_green(x) / 255.0, 0.0, 1.0);
    float b = clamp(CBYIGnBu_blue(x) / 255.0, 0.0, 1.0);
    return vec4(r, g, b, 1.0);
}
// IDLCBBrBG
float cbbrbg_red(float x) {
    if (x < 0.4128910005092621) {
        return (-6.30796693758704E+02 * x + 6.59139629181867E+02) * x + 8.16592339699109E+01;
    } else if (x < 0.5004365747118258) {
        return -1.99292307692284E+01 * x + 2.54503076923075E+02;
    } else if (x < 0.6000321805477142) {
        return -4.46903540903651E+02 * x + 4.68176638176691E+02;
    } else {
        return ((2.43537534073204E+03 * x - 5.03831150657605E+03) * x + 2.73595321475367E+03) * x - 1.53778856560153E+02;
    }
}

float cbbrbg_green(float x) {
    if (x < 0.3067105114459991) {
        return (((((-1.43558931121826E+06 * x + 1.21789289489746E+06) * x - 3.88754308517456E+05) * x + 5.87745165729522E+04) * x - 3.61237992835044E+03) * x + 4.00139210969209E+02) * x + 4.80612502318691E+01;
    } else if (x < 0.4045854562297116) {
        return 3.64978461538455E+02 * x + 8.50984615384636E+01;
    } else if (x < 0.5035906732082367) {
        return 1.25827692307720E+02 * x + 1.81855384615367E+02;
    } else {
        return ((((-2.83948052403926E+04 * x + 1.08768529946603E+05) * x - 1.62569302478295E+05) * x + 1.17919256227845E+05) * x - 4.16776268978779E+04) * x + 6.01529271177582E+03;
    }
}

float cbbrbg_blue(float x) {
    if (x < 0.1012683545126085) {
        return 5.85993431855501E+01 * x + 4.56403940886700E+00;
    } else if (x < 0.2050940692424774) {
        return 3.51072173913048E+02 * x - 2.50542028985514E+01;
    } else if (x < 0.5022056996822357) {
        return (-7.65121475963620E+02 * x + 1.20827362856208E+03) * x - 1.68677387505814E+02;
    } else if (x < 0.5970333516597748) {
        return -1.62299487179500E+02 * x + 3.26660512820525E+02;
    } else {
        return ((1.27993125066091E+03 * x - 3.19799978871341E+03) * x + 2.16242391471484E+03) * x - 1.93738146367890E+02;
    }
}

vec4 cbbrbg_f(float x) {
    float r = clamp(cbbrbg_red(x) / 255.0, 0.0, 1.0);
    float g = clamp(cbbrbg_green(x) / 255.0, 0.0, 1.0);
    float b = clamp(cbbrbg_blue(x) / 255.0, 0.0, 1.0);
    return vec4(r, g, b, 1.0);
}
uniform int colormap;
/*
BlackWhiteLinear = 0,
RedTemperature = 1,
IDLCBGnBu = 2,
IDLCBYIGnBu = 3,
BluePastelRed = 4,
IDLCBBrBG = 5,
*/
vec4 colormap_f(float x) {
    // BlackWhiteLinear = 0,
    if (colormap == 0) {
        return blackw_f(x);
    // RedTemperature = 1,
    } else if (colormap == 1) {
        return red_f(x);
    // IDLCBGnBu = 2,
    } else if (colormap == 2) {
        return cbgnbu_f(x);
    // IDLCBYIGnBu = 3,
    } else if (colormap == 3) {
        return CBYIGnBu_f(x);
    // BluePastelRed = 4,
    } else if (colormap == 4) {
        return bluepastelred_f(x);
    // IDLCBBrBG = 5,
    } else {
        return cbbrbg_f(x);
    }
}