"use strict";
module.exports = class Utils {
    
    static radecToPolar(t, s) {
        return { theta: Math.PI / 2 - (s / 180) * Math.PI, phi: (t / 180) * Math.PI };
    }

static polarToRadec(t, s) {
        return { ra: (180 * s) / Math.PI, dec: (180 * (Math.PI / 2 - t)) / Math.PI };
    }
    
static castToInt(t) {
        return t > 0 ? Math.floor(t) : Math.ceil(t);
    }
    
}