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
        this.colormap = this.colormap.toLowerCase();

        this.stretch = (options && options.stretch) || "linear";
        this.stretch = this.stretch.toLowerCase();
        this.reversed = false;
        // Keep the image tile format because we want the cuts
        this.imgFormat = options.imgFormat || 'png';

        if (options && options.reversed === true) {
            this.reversed = true;
        }

        this.minCut = {
            webp: 0.0,
            jpeg: 0.0,
            png: 0.0,
            fits: undefined // wait the default value coming from the properties
        };
        if (options && Number.isFinite(options.minCut)) {
            this.minCut[this.imgFormat] = options.minCut;
        }

        this.maxCut = {
            webp: 1.0,
            jpeg: 1.0,
            png: 1.0,
            fits: undefined // wait the default value coming from the properties
        };
        if (options && Number.isFinite(options.maxCut)) {
            this.maxCut[this.imgFormat] = options.maxCut;
        }

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
                minCut: this.minCut[this.imgFormat],
                maxCut: this.maxCut[this.imgFormat],
                reversed: this.reversed,
                cmapName: this.colormap,
            }
        };
    }
    
    ColorCfg.prototype.setOptions = function(options) {
        // Update the imgFormat 
        this.imgFormat = options.imgFormat || this.imgFormat;

        this.setColormap(options.colormap, options)

        this.setCuts(options.minCut, options.maxCut)

        this.setBrightness(options.brightness)
        this.setSaturation(options.saturation)
        this.setContrast(options.contrast)
        
        this.setGamma(options.gamma)

        this.setOpacity(options.opacity)

        this.setBlendingConfig(options.additive)
    }

    // @api
    ColorCfg.prototype.setBrightness = function(kBrightness) {
        if (kBrightness == null || kBrightness == undefined)
            return;

        kBrightness = +kBrightness || 0.0; // coerce to number
        this.kBrightness = Math.max(-1, Math.min(kBrightness, 1));
    };

    // @api
    ColorCfg.prototype.getBrightness = function() {
        return this.kBrightness;
    };

    // @api
    ColorCfg.prototype.setContrast = function(kContrast) {
        if (kContrast == null || kContrast == undefined)
            return;

        kContrast = +kContrast || 0.0; // coerce to number
        this.kContrast = Math.max(-1, Math.min(kContrast, 1));
    };

    // @api
    ColorCfg.prototype.getContrast = function() {
        return this.kContrast;
    };

    // @api
    ColorCfg.prototype.setSaturation = function(kSaturation) {
        if (kSaturation == null || kSaturation == undefined)
            return;

        kSaturation = +kSaturation || 0.0; // coerce to number

        this.kSaturation = Math.max(-1, Math.min(kSaturation, 1));
    };

    // @api
    ColorCfg.prototype.getSaturation = function() {
        return this.kSaturation;
    };

    // @api
    ColorCfg.prototype.setGamma = function(gamma) {
        if (gamma == null || gamma == undefined)
            return;

        gamma = +gamma; // coerce to number
        this.kGamma = Math.max(0.1, Math.min(gamma, 10));
    };

    // @api
    ColorCfg.prototype.getGamma = function() {
        return this.kGamma;
    };

    // @api
    ColorCfg.prototype.setOpacity = function(opacity) {
        if (opacity == null || opacity == undefined)
            return;

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
    ColorCfg.prototype.setBlendingConfig = function(additive) {
        if (additive === null || additive === undefined)
            return;

        this.additiveBlending = additive;
    };

    ColorCfg.prototype.getBlendingConfig = function() {
        return this.additiveBlending;
    };

    // @api
    // Optional arguments, 
    ColorCfg.prototype.setColormap = function(colormap, options) {
        /// colormap
        this.colormap = (colormap && colormap.toLowerCase()) || this.colormap;

        /// stretch
        let stretch = (options && options.stretch) || this.stretch || "linear";
        this.stretch = stretch.toLowerCase()

        /// reversed
        if (options && options.reversed !== undefined) {
            this.reversed = options.reversed;
        }
    }

    // @api
    ColorCfg.prototype.getColormap = function() {
        return this.colormap;
    };

    ColorCfg.prototype.getReversed = function() {
        return this.reversed;
    };

    // Sets the cuts for the current image format
    ColorCfg.prototype.setCuts = function(minCut, maxCut) {
        if (minCut instanceof Object) {
            // Mincut is given in the form of an javascript object with all the formats
            this.minCut = minCut
        } else if (minCut !== null && minCut !== undefined) {
            this.minCut[this.imgFormat] = minCut;
        }

        if (maxCut instanceof Object) {
            this.maxCut = maxCut;
        } else if (maxCut !== null && maxCut !== undefined) {
            this.maxCut[this.imgFormat] = maxCut;
        }
    };

    // Returns the cuts for the current image format
    ColorCfg.prototype.getCuts = function() {
        return [
            this.minCut[this.imgFormat],
            this.maxCut[this.imgFormat]
        ];
    };

    return ColorCfg;
 })();
