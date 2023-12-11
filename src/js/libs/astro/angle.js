//=================================
// Class Coo
//=================================
import { Format } from "./coo";
/**
 * Constructor
 * @param angle angle (precision in degrees)
 * @param prec precision
 * (8: 1/1000th sec, 7: 1/100th sec, 6: 1/10th sec, 5: sec, 4: 1/10th min, 3: min, 2: 1/10th deg, 1: deg
 */
export let Angle = function(angle, prec) {
	this.angle = angle;
	this.prec = prec;
};

Angle.prototype = {
	/**
	 * Format the angle
	 * @return the formatted angle
	 */
	format: function() {
        let d = this.angle;
        let suffix;
        let fov;
        if (Math.floor(d) == 0) {
            let m = d*60.0;
    
            if (Math.floor(m) == 0) {
                // sec
                suffix = '"';
                fov = m*60.0;
            } else {
                // min
                suffix = '\'';
                fov = m;
            }
        } else {
            // d
            suffix = '°';
            fov = d;
        }
    
        return Format.toDecimal(fov, this.prec) + suffix;
    },

    parse: function(str) {
        // check for degrees
        let idxUnit;
        idxUnit = str.indexOf('°');
        if (idxUnit > 0) {
            this.angle = +str.substring(0, idxUnit)
            return true;
        }

        idxUnit = str.indexOf('\'');
        if (idxUnit > 0) {
            this.angle = (+str.substring(0, idxUnit)) / 60.0
            return true;
        }

        idxUnit = str.indexOf('"');
        if (idxUnit > 0) {
            this.angle = (+str.substring(0, idxUnit)) / 3600.0
            return true;
        }

        return false
    },

    degrees: function() {
        return this.angle;
    },

    radians: function() {
        return this.angle * Math.PI / 180.0;
    }
}