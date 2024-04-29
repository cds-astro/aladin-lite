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
import { Utils } from "./Utils";
import { ImageSurvey } from "./ImageSurvey.js";


export let ImageFITS = (function () {

    function ImageFITS(url, name, options, successCallback = undefined, errorCallback = undefined) {
        // Name of the layer
        this.layer = null;
        this.added = false;
        this.subtype = "fits";
        // Set it to a default value
        this.url = url;
        this.id = url;
        this.name = name || this.url;

        this.imgFormat = "fits";
        this.formats = ["fits"]
        // callbacks
        this.successCallback = successCallback;
        this.errorCallback = errorCallback;
        // initialize the color meta data here
        // set a asinh stretch by default if there is none
        /*if (options) {
            options.stretch = options.stretch || "asinh";
        }*/
        this.colorCfg = new ColorCfg(options);

        let self = this;

        this.query = Promise.resolve(self);
        this._saveInCache();
    }

    ImageFITS.prototype._saveInCache = function() {
        let self = this;

        let colorOpt = Object.fromEntries(Object.entries(this.colorCfg));
        let fitsOpt = {
            id: self.id,
            name: self.name,
            url: self.url,
            imgFormat: self.imgFormat,
            ...colorOpt
        }

        ImageSurvey.cache[self.id] = self;

        //console.log('new CACHE', ImageSurvey.cache, self.id, surveyOpt, ImageSurvey.cache[self.id], ImageSurvey.cache["CSIRO/P/RACS/mid/I"])

        // Tell that the HiPS List has been updated
        if (this.view) {
            ALEvent.HIPS_LIST_UPDATED.dispatchedTo(this.view.aladin.aladinDiv);
        }
    }
    
    // A cache storing directly the images to not query for the properties each time
    //ImageFITS.cache = {};

    ImageFITS.prototype.setView = function(view) {
        this.view = view;
    }

    // @api
    ImageFITS.prototype.setOpacity = function (opacity) {
        let self = this;
        this._updateMetadata(() => {
            self.colorCfg.setOpacity(opacity);
        });
    };

    // @api
    ImageFITS.prototype.setBlendingConfig = function (additive = false) {
        this._updateMetadata(() => {
            this.colorCfg.setBlendingConfig(additive);
        });
    };

    // @api
    ImageFITS.prototype.setColormap = function (colormap, options) {
        this._updateMetadata(() => {
            this.colorCfg.setColormap(colormap, options);
        });
    }

    // @api
    ImageFITS.prototype.setCuts = function (lowCut, highCut) {
        this._updateMetadata(() => {
            this.colorCfg.setCuts(lowCut, highCut);
        });
    };

    // @api
    ImageFITS.prototype.setGamma = function (gamma) {
        this._updateMetadata(() => {
            this.colorCfg.setGamma(gamma);
        });
    };

    // @api
    ImageFITS.prototype.setSaturation = function (saturation) {
        this._updateMetadata(() => {
            this.colorCfg.setSaturation(saturation);
        });
    };

    ImageFITS.prototype.setBrightness = function (brightness) {
        this._updateMetadata(() => {
            this.colorCfg.setBrightness(brightness);
        });
    };

    ImageFITS.prototype.setContrast = function (contrast) {
        this._updateMetadata(() => {
            this.colorCfg.setContrast(contrast);
        });
    };

    // Private method for updating the view with the new meta
    ImageFITS.prototype._updateMetadata = function (callback) {
        if (callback) {
            callback();
        }

        // Tell the view its meta have changed
        try {
            if (this.added) {
                this.view.wasm.setImageMetadata(this.layer, {
                    ...this.colorCfg.get(),
                    longitudeReversed: false,
                    imgFormat: this.imgFormat
                });
                ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.view.aladinDiv, { layer: this });
            }

            // save it in the JS HiPS cache
            this._saveInCache()
        } catch (e) {
            // Display the error message
            console.error(e);
        }
    }

    ImageFITS.prototype.add = function (layer) {
        this.layer = layer;

        let self = this;

        const promise = self.view.wasm.addImageFITS({
            layer: self.layer,
            url: self.url,
            meta: {
                ...this.colorCfg.get(),
                longitudeReversed: false,
                imgFormat: this.imgFormat
            }
        }).then((imagesParams) => {
            // There is at least one entry in imageParams
            self.added = true;

            self.children = [];

            let hduIdx = 0;
            imagesParams.forEach((imageParams) => {
                // This fits has HDU extensions
                let image = new ImageFITS(
                    imageParams.url,
                    self.name + "_ext_" + hduIdx.toString(),
                    null,
                    null,
                    null
                );

                // Set the layer corresponding to the onein the backend
                image.layer = imageParams.layer;
                image.added = true;
                image.setView(self.view);
                // deep copy of the color object of self
                image.colorCfg = Utils.deepCopy(self.colorCfg);
                // Set the automatic computed cuts
                image.setCuts(imageParams.automatic_min_cut, imageParams.automatic_max_cut);

                image.ra = imageParams.centered_fov.ra;
                image.dec = imageParams.centered_fov.dec;
                image.fov = imageParams.centered_fov.fov;

                if (!self.ra) { self.ra = image.ra; }
                if (!self.dec) { self.dec = image.dec; }
                if (!self.fov) { self.fov = image.fov; }

                self.children.push(image)

                hduIdx += 1;
            });

            // Call the success callback on the first HDU image parsed
            if (self.successCallback) {
                self.successCallback(
                    self.children[0].ra,
                    self.children[0].dec,
                    self.children[0].fov,
                    self.children[0]
                );
            }

            return self;
        }).catch((e) => {
            if (self.errorCallback) {
                self.errorCallback()
            }

            // This error result from a promise
            // If I throw it, it will not be catched because
            // it is run async
            self.view.removeImageLayer(layer)

            return Promise.reject(e);
        });

        return promise;
    };

    // @api
    ImageFITS.prototype.toggle = function () {
        if (this.colorCfg.getOpacity() != 0.0) {
            this.colorCfg.setOpacity(0.0);
        } else {
            this.colorCfg.setOpacity(this.prevOpacity);
        }
    };

    // FITS images does not mean to be used for storing planetary data
    ImageFITS.prototype.isPlanetaryBody = function() {
        return false;
    }

    // @api
    ImageFITS.prototype.focusOn = function () {
        // ensure the fits have been parsed
        if (this.added) {
            this.view.aladin.gotoRaDec(this.ra, this.dec);
            this.view.aladin.setFoV(this.fov);
        }
    };

    // @oldapi
    ImageFITS.prototype.setAlpha = ImageFITS.prototype.setOpacity;

    ImageFITS.prototype.setColorCfg = function (colorCfg) {
        this._updateMetadata(() => {
            this.colorCfg = colorCfg;
        });
    };

    // @api
    ImageFITS.prototype.getColorCfg = function () {
        return this.colorCfg;
    };

    // @api
    ImageFITS.prototype.getCuts = function () {
        return this.colorCfg.getCuts();
    };

    // @api
    ImageFITS.prototype.getOpacity = function () {
        return this.colorCfg.getOpacity();
    };

    ImageFITS.prototype.getAlpha = ImageFITS.prototype.getOpacity;

    // @api
    ImageFITS.prototype.readPixel = function (x, y) {
        return this.view.wasm.readPixel(x, y, this.layer);
    };

    return ImageFITS;
})();

