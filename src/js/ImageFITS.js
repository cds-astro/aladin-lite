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
 * File ImageFITS
 *
 * Authors: Matthieu Baumann [CDS]
 *
 *****************************************************************************/
import { ALEvent } from "./events/ALEvent.js";
import { ColorCfg } from "./ColorCfg.js";
import { HiPSCache } from "./DefaultHiPSCache";
import { Aladin } from "./Aladin.js";

/**
 * @typedef {Object} ImageOptions
 *
 * @property {string} [name] - A human-readable name for the FITS image
 * @property {Function} [successCallback] - A callback executed when the FITS has been loaded
 * @property {Function} [errorCallback] - A callback executed when the FITS could not be loaded
 * @property {number} [opacity=1.0] - Opacity of the survey or image (value between 0 and 1).
 * @property {string} [colormap="native"] - The colormap configuration for the survey or image.
 * @property {string} [stretch="linear"] - The stretch configuration for the survey or image.
 * @property {boolean} [reversed=false] - If true, the colormap is reversed; otherwise, it is not reversed.
 * @property {number} [minCut] - The minimum cut value for the color configuration. If not given, 0.0 for JPEG/PNG surveys, the value of the property file for FITS surveys
 * @property {number} [maxCut] - The maximum cut value for the color configuration. If not given, 1.0 for JPEG/PNG surveys, the value of the property file for FITS surveys
 * @property {boolean} [additive=false] - If true, additive blending is applied; otherwise, it is not applied.
 * @property {number} [gamma=1.0] - The gamma correction value for the color configuration.
 * @property {number} [saturation=0.0] - The saturation value for the color configuration.
 * @property {number} [brightness=0.0] - The brightness value for the color configuration.
 * @property {number} [contrast=0.0] - The contrast value for the color configuration.
 * @property {Object} [wcs] - an object describing the WCS of the image. In case of a fits image
 * this property will be ignored as the WCS taken will be the one present in the fits file.
 * @property {number} [imgFormat='fits'] - The image format of the image to load.
 * 
 * @example
 * 
 *  aladin.setOverlayImageLayer(A.image(
 *       "https://nova.astrometry.net/image/25038473?filename=M61.jpg",
 *       {
 *           name: "M61",
 *           imgFormat: 'jpeg',
 *           wcs: {
 *               NAXIS: 0, // Minimal header
 *               CTYPE1: 'RA---TAN', // TAN (gnomic) projection + SIP distortions
 *               CTYPE2: 'DEC--TAN', // TAN (gnomic) projection + SIP distortions
 *               EQUINOX: 2000.0, // Equatorial coordinates definition (yr)
 *               LONPOLE: 180.0, // no comment
 *               LATPOLE: 0.0, // no comment
 *               CRVAL1: 185.445488837, // RA of reference point
 *               CRVAL2: 4.47896032431, // DEC of reference point
 *               CRPIX1: 588.995094299, // X reference pixel
 *               CRPIX2: 308.307905197, // Y reference pixel
 *               CUNIT1: 'deg', // X pixel scale units
 *               CUNIT2: 'deg', // Y pixel scale units
 *               CD1_1: -0.000223666022989, // Transformation matrix
 *               CD1_2: 0.000296578064584, // no comment
 *               CD2_1: -0.000296427555509, // no comment
 *               CD2_2: -0.000223774308964, // no comment
 *               NAXIS1: 1080, // Image width, in pixels.
 *               NAXIS2: 705 // Image height, in pixels.
 *           },
 *           successCallback: (ra, dec, fov, image) => {
 *               aladin.gotoRaDec(ra, dec);
 *               aladin.setFoV(fov * 5)
 *           }
 *       },
 *   ));
 */

export let Image = (function () {
    /**
     * The object describing a FITS image
     *
     * @class
     * @constructs Image
     *
     * @param {string} url - Mandatory unique identifier for the layer. Can be an arbitrary name
     * @param {ImageOptions} [options] - The option for the survey
     *
     */
    function Image(url, options) {
        // Name of the layer
        this.layer = null;
        this.added = false;
        // Set it to a default value
        this.url = url;
        this.id = url;
        this.name = (options && options.name) || this.url;
        this.imgFormat = (options && options.imgFormat) || "fits";
        this.formats = [this.imgFormat];
        // callbacks
        this.successCallback = options && options.successCallback;
        this.errorCallback = options && options.errorCallback;
        // initialize the color meta data here
        // set a asinh stretch by default if there is none
        /*if (options) {
            options.stretch = options.stretch || "asinh";
        }*/
        this.colorCfg = new ColorCfg(options);
        this.options = options;

        let self = this;
        this.query = Promise.resolve(self);
    }

    Image.prototype._saveInCache = function () {
        if (HiPSCache.contains(self.id)) {
            HiPSCache.append(this.id, this);
        }
    };

    // A cache storing directly the images to not query for the properties each time
    //Image.cache = {};

    Image.prototype.setView = function (view) {
        this.view = view;

        this._saveInCache();
    };

    // @api
    Image.prototype.setOpacity = function (opacity) {
        let self = this;
        this._updateMetadata(() => {
            self.colorCfg.setOpacity(opacity);
        });
    };

    // @api
    Image.prototype.setBlendingConfig = function (additive = false) {
        this._updateMetadata(() => {
            this.colorCfg.setBlendingConfig(additive);
        });
    };

    // @api
    Image.prototype.setColormap = function (colormap, options) {
        this._updateMetadata(() => {
            this.colorCfg.setColormap(colormap, options);
        });
    };

    // @api
    Image.prototype.setCuts = function (lowCut, highCut) {
        this._updateMetadata(() => {
            this.colorCfg.setCuts(lowCut, highCut);
        });
    };

    // @api
    Image.prototype.setGamma = function (gamma) {
        this._updateMetadata(() => {
            this.colorCfg.setGamma(gamma);
        });
    };

    // @api
    Image.prototype.setSaturation = function (saturation) {
        this._updateMetadata(() => {
            this.colorCfg.setSaturation(saturation);
        });
    };

    Image.prototype.setBrightness = function (brightness) {
        this._updateMetadata(() => {
            this.colorCfg.setBrightness(brightness);
        });
    };

    Image.prototype.setContrast = function (contrast) {
        this._updateMetadata(() => {
            this.colorCfg.setContrast(contrast);
        });
    };

    // Private method for updating the view with the new meta
    Image.prototype._updateMetadata = function (callback) {
        if (callback) {
            callback();
        }

        // Tell the view its meta have changed
        try {
            if (this.added) {
                this.view.wasm.setImageMetadata(this.layer, {
                    ...this.colorCfg.get(),
                    longitudeReversed: false,
                    imgFormat: this.imgFormat,
                });
                ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.view.aladinDiv, {
                    layer: this,
                });
            }

            // save it in the JS HiPS cache
            this._saveInCache();
        } catch (e) {
            // Display the error message
            console.error(e);
        }
    };

    Image.prototype.add = function (layer) {
        this.layer = layer;

        let self = this;

        let promise;
        if (this.imgFormat === 'fits') {
            let id = this.url;
            promise = fetch(this.url)
                .then((resp) => resp.body)
                .then((readableStream) => {
                    return self.view.wasm
                        .addImageFITS(
                            id,
                            readableStream,
                            {
                                ...self.colorCfg.get(),
                                longitudeReversed: false,
                                imgFormat: self.imgFormat,
                            },
                            layer
                        )
                })
        } else if (this.imgFormat === 'jpg' || this.imgFormat === 'jpeg') {
            let img = document.createElement('img');

            promise =
                new Promise((resolve, reject) => {
                    img.src = this.url;
                    img.crossOrigin = "Anonymous";
                    img.onload = () => {
                        var canvas = document.createElement("canvas");
        
                        canvas.width = img.width;
                        canvas.height = img.height;
                    
                        // Copy the image contents to the canvas
                        var ctx = canvas.getContext("2d");
                        ctx.drawImage(img, 0, 0, img.width, img.height);
        
                        const imageData = ctx.getImageData(0, 0, img.width, img.height);

                        const blob = new Blob([imageData.data]);
                        const stream = blob.stream(1024);

                        resolve(stream)
                    }
        
                    let proxyUsed = false;
                    img.onerror = () => {
                        // use proxy
                        if (proxyUsed) {
                            reject('Error parsing img ' + self.url)
                            return;
                        }

                        proxyUsed = true;
                        img.src = Aladin.JSONP_PROXY + '?url=' + self.url;                   
                    }
                })
                .then((readableStream) => {
                    let wcs = self.options && self.options.wcs;
                    wcs.NAXIS1 = wcs.NAXIS1 || img.width;
                    wcs.NAXIS2 = wcs.NAXIS2 || img.height;

                    return self.view.wasm
                        .addImageWithWCS(
                            readableStream,
                            wcs,
                            {
                                ...self.colorCfg.get(),
                                longitudeReversed: false,
                                imgFormat: self.imgFormat,
                            },
                            layer
                        )
                })
                .finally(() => {
                    img.remove();
                })
        } else {
            console.warn(`Image format: ${this.imgFormat} not supported`);
            promise = Promise.reject();
        };

        promise = promise.then((imageParams) => {
            // There is at least one entry in imageParams
            self.added = true;
            self.setView(self.view);

            // Set the automatic computed cuts
            self.setCuts(
                imageParams.min_cut,
                imageParams.max_cut
            );

            self.ra = imageParams.centered_fov.ra;
            self.dec = imageParams.centered_fov.dec;
            self.fov = imageParams.centered_fov.fov;

            // Call the success callback on the first HDU image parsed
            if (self.successCallback) {
                self.successCallback(
                    self.ra,
                    self.dec,
                    self.fov,
                    self
                );
            }

            return self;
        })
        .catch((e) => {
            // This error result from a promise
            // If I throw it, it will not be catched because
            // it is run async
            self.view.removeImageLayer(layer);

            return Promise.reject(e);
        });

        return promise;
    };

    // @api
    Image.prototype.toggle = function () {
        if (this.colorCfg.getOpacity() != 0.0) {
            this.colorCfg.setOpacity(0.0);
        } else {
            this.colorCfg.setOpacity(this.prevOpacity);
        }
    };

    // FITS images does not mean to be used for storing planetary data
    Image.prototype.isPlanetaryBody = function () {
        return false;
    };

    // @api
    Image.prototype.focusOn = function () {
        // ensure the fits have been parsed
        if (this.added) {
            this.view.aladin.gotoRaDec(this.ra, this.dec);
            this.view.aladin.setFoV(this.fov);
        }
    };

    // @oldapi
    Image.prototype.setAlpha = Image.prototype.setOpacity;

    Image.prototype.setColorCfg = function (colorCfg) {
        this._updateMetadata(() => {
            this.colorCfg = colorCfg;
        });
    };

    // @api
    Image.prototype.getColorCfg = function () {
        return this.colorCfg;
    };

    // @api
    Image.prototype.getCuts = function () {
        return this.colorCfg.getCuts();
    };

    // @api
    Image.prototype.getOpacity = function () {
        return this.colorCfg.getOpacity();
    };

    Image.prototype.getAlpha = Image.prototype.getOpacity;

    // @api
    Image.prototype.readPixel = function (x, y) {
        return this.view.wasm.readPixel(x, y, this.layer);
    };

    return Image;
})();
