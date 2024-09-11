//=================================
// Class Coo
//=================================
import { Format } from "./coo";

/*
/**
 * Creates an angle of the Aladin interactive sky atlas.
 * @class
 * @constructs Angle
 * @param {number} angle - angle in degrees
 * @param {number} prec - precision
 * (8: 1/1000th sec, 7: 1/100th sec, 6: 1/10th sec, 5: sec, 4: 1/10th min, 3: min, 2: 1/10th deg, 1: deg
 */
export let Angle = function(angle, prec) {
	this.angle = angle;

    if (prec === undefined || prec === null) {
        prec = 0;
    }
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

    /**
     * @memberof Angle
     * 
     * @param {string} str - A string in the form [<deg>°<minutes>'<seconds>"]. [hms] form is not supported
     * @returns {boolean} - Whether the string has been successfully parsed
     */

    parse: function(str) {
        // check for degrees
        let idx = str.indexOf('°');

        let angleDeg = NaN;
        if (idx > 0) {
            const deg = parseFloat(str.substring(0, idx));
            if (!Number.isFinite(deg)) {
                return false
            }

            angleDeg = deg;

            str = str.substring(idx + 1)
        }

        idx = str.indexOf('\'');
        if (idx > 0) {
            const minutes = parseFloat(str.substring(0, idx))

            if (!Number.isFinite(minutes)) {
                return false
            }

            if (!Number.isFinite(angleDeg)) {
                angleDeg = 0;
            }
            angleDeg += minutes / 60.0

            str = str.substring(idx + 1);
        }

        idx = str.indexOf('"');
        if (idx > 0) {
            const seconds = parseFloat(str.substring(0, idx))

            if (!Number.isFinite(seconds)) {
                return false;
            }

            if (!Number.isFinite(angleDeg)) {
                angleDeg = 0;
            }

            angleDeg += seconds / 3600.0
        }

        if (Number.isFinite(angleDeg)) {
            this.angle = angleDeg;
            return true;
        } else {
            return false
        }
    },

    degrees: function() {
        return this.angle;
    },

    radians: function() {
        return this.angle * Math.PI / 180.0;
    }
}