// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU General Public License version 3.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, version 3 of the License.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    The GNU General Public License is available in COPYING file
//    along with Aladin Lite.
//

/******************************************************************************
 * Aladin Lite project
 * 
 * File ColorCfg
 * 
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 * 
 *****************************************************************************/
 export let ColorCfg = (function() {
    /** Constructor
     * cooFrame and maxOrder can be set to null
     * They will be determined by reading the properties file
     */
    function ColorCfg(options) {
        // Opacity of the survey/image
        this.opacity = (options && options.opacity) || 1.0;

        // Colormap config options
        this.colormap = (options && options.colormap) || "native";
        this.colormap = formatColormap(this.colormap);

        this.stretch = (options && options.stretch) || "linear";
        this.stretch = this.stretch.toLowerCase();
        this.reversed = false;

        if (options && options.reversed === true) {
            this.reversed = true;
        }

        this.minCut = (options && options.minCut) || 0.0;
        this.maxCut = (options && options.maxCut) || 1.0;

        this.additiveBlending = options && options.additive;
        if (this.additiveBlending === undefined)  {
            this.additiveBlending = false;
        }

        // A default value for gamma correction
        this.kGamma = (options && options.gamma) || 1.0;
        this.kSaturation = (options && options.saturation) || 0.0;
        this.kBrightness = (options && options.brightness) || 0.0;
        this.kContrast = (options && options.contrast) || 0.0;
    };

    ColorCfg.prototype.get = function() {
        let blend = {
            srcColorFactor: 'SrcAlpha',
            dstColorFactor: 'OneMinusSrcAlpha',
            func: 'FuncAdd' 
        };

        if (this.additiveBlending) {
            blend = {
                srcColorFactor: 'SrcAlpha',
                dstColorFactor: 'One',
                func: 'FuncAdd' 
            }
        }

        // Reset the whole meta object
        return {
            blendCfg: blend,
            opacity: this.opacity,
            color: {
                // Tonal corrections constants
                kGamma: this.kGamma,
                kSaturation: this.kSaturation,
                kBrightness: this.kBrightness,
                kContrast: this.kContrast,

                stretch: this.stretch,
                minCut: this.minCut,
                maxCut: this.maxCut,
                reversed: this.reversed,
                cmapName: this.colormap,
            }
        };
    }

    // @api
    ColorCfg.prototype.setBrightness = function(kBrightness) {
        kBrightness = +kBrightness || 0.0; // coerce to number
        this.kBrightness = Math.max(-1, Math.min(kBrightness, 1));
    };

    // @api
    ColorCfg.prototype.getBrightness = function() {
        return this.kBrightness;
    };

    // @api
    ColorCfg.prototype.setContrast = function(kContrast) {
        kContrast = +kContrast || 0.0; // coerce to number
        this.kContrast = Math.max(-1, Math.min(kContrast, 1));
    };

    // @api
    ColorCfg.prototype.getContrast = function() {
        return this.kContrast;
    };

    // @api
    ColorCfg.prototype.setSaturation = function(kSaturation) {
        kSaturation = +kSaturation || 0.0; // coerce to number

        this.kSaturation = Math.max(-1, Math.min(kSaturation, 1));
    };

    // @api
    ColorCfg.prototype.getSaturation = function() {
        return this.kSaturation;
    };

    // @api
    ColorCfg.prototype.setGamma = function(gamma) {
        gamma = +gamma; // coerce to number
        this.kGamma = Math.max(0.1, Math.min(gamma, 10));
    };

    // @api
    ColorCfg.prototype.getGamma = function() {
        return this.kGamma;
    };

    // @api
    ColorCfg.prototype.setOpacity = function(opacity) {
        opacity = +opacity; // coerce to number
        this.opacity = Math.max(0, Math.min(opacity, 1));
    };

    // @oldapi
    ColorCfg.prototype.setAlpha = ColorCfg.prototype.setOpacity;

    // @api
    ColorCfg.prototype.getOpacity = function() {
        return this.opacity;
    };

    // @api
    ColorCfg.prototype.getAlpha = ColorCfg.prototype.getOpacity;

    // @api
    ColorCfg.prototype.setBlendingConfig = function(additive = false) {
        this.additiveBlending = additive;
    };

    ColorCfg.prototype.getBlendingConfig = function() {
        return this.additiveBlending;
    };

    var formatColormap = function(colormap) {
        /// colormap
        // Make it case insensitive
        colormap = colormap.toLowerCase();
        if (!ColorCfg.COLORMAPS.includes(colormap)) {
            console.warn("The colormap \'" + colormap + "\' is not supported. You should use one of the following: " + ColorCfg.COLORMAPS + "\n\'grayscale\' has been chosen by default.")
            // If the user specify a colormap that is not supported,
            // then set it to grayscale
            colormap = "grayscale";
        }

        return colormap;
    }

    // @api
    // Optional arguments, 
    ColorCfg.prototype.setColormap = function(colormap = "native", options = {}) {
        /// colormap
        // Make it case insensitive
        let cmap = formatColormap(colormap);

        /// stretch
        let stretch = (options && options.stretch) || this.stretch || "linear";
        stretch = stretch.toLowerCase();

        /// reversed
        let reversed = false;
        if (options && options.reversed === true) {
            reversed = true;
        }

        this.colormap = cmap;
        this.stretch = stretch;
        this.reversed = reversed;
    }

    // @api
    ColorCfg.prototype.getColormap = function() {
        return this.colormap;
    };

    // @api
    ColorCfg.prototype.setCuts = function(lowCut, highCut) {
        this.minCut = lowCut;
        this.maxCut = highCut;
    };

    ColorCfg.COLORMAPS = [];

    return ColorCfg;
 })();
