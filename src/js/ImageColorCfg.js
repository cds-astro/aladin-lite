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
 * File HpxImageSurvey
 * 
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 * 
 *****************************************************************************/
 export let ImageColorCfg = (function() {
    /** Constructor
     * cooFrame and maxOrder can be set to null
     * They will be determined by reading the properties file
     */
    function ImageColorCfg(options) {
        // Name of the layer
        this.layer = null;

        // Opacity of the survey/image
        this.opacity = (options && options.opacity) || 1.0;

        // Colormap config options
        this.stretch = (options && options.stretch) || "linear";
        this.stretch = this.stretch.toLowerCase();

        this.minCut = (options && options.minCut) || 0.0;
        this.maxCut = (options && options.maxCut) || 1.0;
        this.colormap = (options && options.colormap) || "grayscale";
        
        this.reversed = options && options.reversed;
        if (this.reversed === undefined) {
            this.reversed = false;
        }

        this.additiveBlending = options.additive;
    };

    HpxImageSurvey.prototype.computeColorCfg = function() {
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

        // reset the whole meta object
        return {
            blendCfg: blend,
            opacity: this.opacity,
            color: {
                grayscale: {
                    stretch: stretch,
                    minCut: minCut,
                    maxCut: maxCut,
                    color: {
                        colormap: {
                            reversed: reversed,
                            name: colormap,
                        }
                    }
                }
            }
        };
    }

    // @api
    HpxImageSurvey.prototype.setOpacity = function(opacity) {
        opacity = +opacity; // coerce to number
        this.opacity = Math.max(0, Math.min(opacity, 1));
    };

    // @oldapi
    HpxImageSurvey.prototype.setAlpha = HpxImageSurvey.prototype.setOpacity;

    // @api
    HpxImageSurvey.prototype.getOpacity = function() {
        return this.opacity;
    };

    // @api
    HpxImageSurvey.prototype.setBlendingConfig = function(additive = false) {
        this.additiveBlending = additive;
    };

    // @api
    // Optional arguments, 
    HpxImageSurvey.prototype.setColormap = function(colormap, stretch = undefined, reversed = undefined) {
        /// colormap
        // If not defined we set the colormap to grayscale
        if (!colormap) {
            colormap = "grayscale";
        }

        if (colormap === "native") {
            colormap = "grayscale";
        }

        // Make it case insensitive
        colormap = colormap.toLowerCase();

        if (!ImageColorCfg.COLORMAPS.includes(colormap)) {
            console.warn("The colormap \'" + colormap + "\' does not exist. You should use one of the following: " + ImageColorCfg.COLORMAPS + "\n\'grayscale\' has been chosen by default.")

            colormap = "grayscale";
        }

        /// stretch
        if (stretch) {
            stretch = stretch.toLowerCase();
        }

        /// reversed 
        if (reversed === undefined) {
            reversed = false;
        }

        this.colormap = colormap;
        this.stretch = stretch;
        this.reversed = reversed;
    }

    // @api
    HpxImageSurvey.prototype.setCuts = function(lowCut, highCut) {
        this.minCut = lowCut;
        this.maxCut = highCut;
    };

    // @api
    ImageColorCfg.prototype.getAlpha = function() {
        return this.opacity;
    };

    ImageColorCfg.COLORMAPS = [];
    
    return ImageColorCfg;
 })();
 
 