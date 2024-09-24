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
 * File Image
 *
 * Authors: Matthieu Baumann [CDS]
 *
 *****************************************************************************/
import { ColorCfg } from "./ColorCfg.js";
import { Aladin } from "./Aladin.js";
import { Utils } from "./Utils";
import { AVM } from "./libs/avm.js";
import { HiPS } from "./HiPS.js";

/**
 * @typedef {Object} WCS
 * 
 * {@link https://ui.adsabs.harvard.edu/abs/2002A%26A...395.1077C/abstract|FITS (Paper II)}, Calabretta, M. R., and Greisen, E. W., Astronomy & Astrophysics, 395, 1077-1122, 2002
 * 
 * @property {number} [NAXIS]
 * @property {string} CTYPE1 
 * @property {string} [CTYPE2]
 * @property {number} [LONPOLE]
 * @property {number} [LATPOLE]
 * @property {number} [CRVAL1]
 * @property {number} [CRVAL2]
 * @property {number} [CRPIX1]
 * @property {number} [CRPIX2]
 * @property {string} [CUNIT1] - e.g. 'deg'
 * @property {string} [CUNIT2] - e.g. 'deg'
 * @property {number} [CD1_1]
 * @property {number} [CD1_2]
 * @property {number} [CD2_1]
 * @property {number} [CD2_2]
 * @property {number} [PC1_1]
 * @property {number} [PC1_2]
 * @property {number} [PC2_1]
 * @property {number} [PC2_2]
 * @property {number} [CDELT1]
 * @property {number} [CDELT2]
 * @property {number} [NAXIS1]
 * @property {number} [NAXIS2]
 */

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
 * @property {number} [minCut=0.0] - The minimum cut value for the color configuration. If not given, 0.0 is chosen
 * @property {number} [maxCut=1.0] - The maximum cut value for the color configuration. If not given, 1.0 is chosen
 * @property {boolean} [additive=false] - If true, additive blending is applied; otherwise, it is not applied.
 * @property {number} [gamma=1.0] - The gamma correction value for the color configuration.
 * @property {number} [saturation=0.0] - The saturation value for the color configuration.
 * @property {number} [brightness=0.0] - The brightness value for the color configuration.
 * @property {number} [contrast=0.0] - The contrast value for the color configuration.
 * @property {WCS} [wcs] - an object describing the WCS of the image. In case of a fits image
 * this property will be ignored as the WCS taken will be the one present in the fits file.
 * @property {string} [imgFormat] - Optional image format. Giving it will prevent the auto extension determination algorithm to be triggered. Possible values are 'jpeg', 'png' or 'fits'. tiff files are not supported. You can convert your tiff files to jpg ones by using the fantastic image magick suite.
 * 
 * @example
 * 
 *  aladin.setOverlayImageLayer(A.image(
 *       "https://nova.astrometry.net/image/25038473?filename=M61.jpg",
 *       {
 *           name: "M61",
 *           wcs: {
                NAXIS: 0, // Minimal header
                CTYPE1: 'RA---TAN', // TAN (gnomic) projection
                CTYPE2: 'DEC--TAN', // TAN (gnomic) projection
                EQUINOX: 2000.0, // Equatorial coordinates definition (yr)
                LONPOLE: 180.0, // no comment
                LATPOLE: 0.0, // no comment
                CRVAL1: 185.445488837, // RA of reference point
                CRVAL2: 4.47896032431, // DEC of reference point
                CRPIX1: 588.995094299, // X reference pixel
                CRPIX2: 308.307905197, // Y reference pixel
                CUNIT1: 'deg', // X pixel scale units
                CUNIT2: 'deg', // Y pixel scale units
                CD1_1: -0.000223666022989, // Transformation matrix
                CD1_2: -0.000296578064584, // no comment
                CD2_1: -0.000296427555509, // no comment
                CD2_2: 0.000223774308964, // no comment
                NAXIS1: 1080, // Image width, in pixels.
                NAXIS2: 705 // Image height, in pixels.
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
    let Image = function(url, options) {
        // Name of the layer
        this.layer = null;
        this.added = false;
        // Set it to a default value
        this.url = url;
        this.id = url;
        this.name = (options && options.name) || this.url;
        this.imgFormat = options && options.imgFormat;
        //this.formats = [this.imgFormat];
        // callbacks
        this.successCallback = options && options.successCallback;
        this.errorCallback = options && options.errorCallback;

        this.longitudeReversed = false;

        this.colorCfg = new ColorCfg(options);
        this.options = options || {};

        let self = this;

        this.query = Promise.resolve(self);
    }

    Image.prototype = {
        /* Precondition: view is already attached */
        _saveInCache: HiPS.prototype._saveInCache,

        // @api
        getCuts: HiPS.prototype.getCuts,

            // @api
        setOpacity: HiPS.prototype.setOpacity,


        // @api
        setOptions: HiPS.prototype.setOptions,
        // @api
        setBlendingConfig: HiPS.prototype.setBlendingConfig,

        // @api
        setColormap: HiPS.prototype.setColormap,

        // @api
        setCuts: HiPS.prototype.setCuts,

        // @api
        setGamma: HiPS.prototype.setGamma,

        // @api
        setSaturation: HiPS.prototype.setSaturation,

        setBrightness: HiPS.prototype.setBrightness,

        setContrast: HiPS.prototype.setContrast,

        // @api
        toggle: HiPS.prototype.toggle,
        // @oldapi
        setAlpha: HiPS.prototype.setOpacity,
    
        setColorCfg: HiPS.prototype.setColorCfg,
    
        // @api
        getColorCfg: HiPS.prototype.getColorCfg,
    
        // @api
        getOpacity: HiPS.prototype.getOpacity,
        getAlpha: HiPS.prototype.getOpacity,
    
        // @api
        readPixel: HiPS.prototype.readPixel,

        // Private method for updating the view with the new meta
        _updateMetadata: HiPS.prototype._updateMetadata,

        setView: function (view) {
            this.view = view;
            this._saveInCache();
        },

        _addFITS: function(layer) {
            let self = this;

            return Utils.fetch({
                url: this.url,
                dataType: 'readableStream',
                success: (stream) => {
                    return self.view.wasm.addImageFITS(
                        stream,
                        {
                            ...self.colorCfg.get(),
                            longitudeReversed: this.longitudeReversed,
                            imgFormat: 'fits',
                        },
                        layer
                    )
                },
                error: (e) => {
                    // try as cors 
                    const url = Aladin.JSONP_PROXY + '?url=' + self.url;

                    return Utils.fetch({
                        url: url,
                        dataType: 'readableStream',
                        success: (stream) => {
                            return self.view.wasm.addImageFITS(
                                stream,
                                {
                                    ...self.colorCfg.get(),
                                    longitudeReversed: this.longitudeReversed,
                                    imgFormat: 'fits',
                                },
                                layer
                            )
                        },
                    });
                }
            })
            .then((imageParams) => {
                self.imgFormat = 'fits';

                return Promise.resolve(imageParams);
            })
        },

        _addJPGOrPNG: function(layer) {
            let self = this;
            let img = document.createElement('img');

            return new Promise((resolve, reject) => {
                img.src = this.url;
                img.crossOrigin = "Anonymous";
                img.onload = () => {
                    const img2Blob = () => {
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
                    };

                    if (!self.options.wcs) {
                        /* look for avm tags if no wcs is given */
                        let avm = new AVM(img);

                        avm.load((obj) => {
                            // obj contains the following information:
                            // obj.id (string) = The ID provided for the image
                            // obj.img (object) = The image object
                            // obj.xmp (string) = The raw XMP header
                            // obj.wcsdata (Boolean) = If WCS have been loaded
                            // obj.tags (object) = An array containing all the loaded tags e.g. obj.tags['Headline']
                            // obj.wcs (object) = The wcs parsed from the image
                            if (obj.wcsdata) {
                                if (img.width !== obj.wcs.NAXIS1) {
                                    obj.wcs.NAXIS1 = img.width;
                                }

                                if (img.height !== obj.wcs.NAXIS2) {
                                    obj.wcs.NAXIS2 = img.height;
                                }

                                self.options.wcs = obj.wcs;

                                img2Blob()
                            } else {
                                // no tags found
                                reject('No WCS have been found in the image')
                                return;
                            }
                        })
                    } else {
                        img2Blob()
                    }
                }

                let proxyUsed = false;
                img.onerror = (e) => {
                    // use proxy
                    if (proxyUsed) {
                        console.error(e);

                        reject('Error parsing image located at: ' + self.url)
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
                            longitudeReversed: this.longitudeReversed,
                            imgFormat: 'jpeg',
                        },
                        layer
                    )
            })
            .then((imageParams) => {
                self.imgFormat = 'jpeg';
                return Promise.resolve(imageParams);
            })
            .finally(() => {
                img.remove();
            });
        },

        add: function (layer) {
            this.layer = layer;

            let self = this;
            let promise;

            if (this.imgFormat === 'fits') {
                promise = this._addFITS(layer)
                    .catch(e => {
                        console.error(`Image located at ${this.url} could not be parsed as fits file. Is the imgFormat specified correct?`)
                        return Promise.reject(e)
                    })
            } else if (this.imgFormat === 'jpeg' || this.imgFormat === 'png') {
                promise = this._addJPGOrPNG(layer)
                    .catch(e => {
                        console.error(`Image located at ${this.url} could not be parsed as a ${this.imgFormat} file. Is the imgFormat specified correct?`);
                        return Promise.reject(e)
                    })
            } else {
                // imgformat not defined we will try first supposing it is a fits file and then use the jpg heuristic
                promise = self._addFITS(layer)
                    .catch(e => {
                        return self._addJPGOrPNG(layer)
                            .catch(e => {
                                console.error(`Image located at ${self.url} could not be parsed as jpg/png/tif image file. Aborting...`)
                                return Promise.reject(e);
                            })
                    })
            }

            promise = promise.then((imageParams) => {
                self.formats = [self.imgFormat];

                // There is at least one entry in imageParams
                self.added = true;
                self.setView(self.view);

                // Set the automatic computed cuts
                let [minCut, maxCut] = self.getCuts();
                minCut = minCut || imageParams.min_cut;
                maxCut = maxCut || imageParams.max_cut;
                self.setCuts(
                    minCut,
                    maxCut
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
        },

        // FITS images does not mean to be used for storing planetary data
        isPlanetaryBody: function () {
            return false;
        },

        // @api
        focusOn: function () {
            // ensure the fits have been parsed
            if (this.added) {
                this.view.aladin.gotoRaDec(this.ra, this.dec);
                this.view.aladin.setFoV(this.fov);
            }
        },
    };

    return Image;
})();
