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
import { ImageLayer } from "./ImageLayer.js";
import { Utils } from "./Utils.js";

export let ImageFITS = (function () {

    function ImageFITS(url, name, view, options, successCallback = undefined, errorCallback = undefined) {
        this.view = view;
        this.wasm = view.wasm;

        // Name of the layer
        this.layer = null;
        this.added = false;
        this.subtype = "fits";
        // Set it to a default value
        this.url = url.toString();

        this.id = url.toString();
        this.name = name;

        this.imgFormat = "fits";
        this.properties = {
            formats: ["fits"]
        }
        // callbacks
        this.successCallback = successCallback;
        this.errorCallback = errorCallback;
        // initialize the color meta data here
        this.colorCfg = new ColorCfg(options);

        let self = this;

        updateMetadata(self);
        ImageLayer.update(self);

        this.query = Promise.resolve(self);
    }

    ImageFITS.prototype.isReady = function() {
        return this.added;
    }

    // @api
    ImageFITS.prototype.setOpacity = function (opacity) {
        let self = this;
        updateMetadata(self, () => {
            self.colorCfg.setOpacity(opacity);
        });
    };

    // @api
    ImageFITS.prototype.setBlendingConfig = function (additive = false) {
        updateMetadata(this, () => {
            this.colorCfg.setBlendingConfig(additive);
        });
    };

    // @api
    ImageFITS.prototype.setColormap = function (colormap, options) {
        updateMetadata(this, () => {
            this.colorCfg.setColormap(colormap, options);
        });
    }

    // @api
    ImageFITS.prototype.setCuts = function (lowCut, highCut) {
        updateMetadata(this, () => {
            this.colorCfg.setCuts(lowCut, highCut);
        });
    };

    // @api
    ImageFITS.prototype.setGamma = function (gamma) {
        updateMetadata(this, () => {
            this.colorCfg.setGamma(gamma);
        });
    };

    // @api
    ImageFITS.prototype.setSaturation = function (saturation) {
        updateMetadata(this, () => {
            this.colorCfg.setSaturation(saturation);
        });
    };

    ImageFITS.prototype.setBrightness = function (brightness) {
        updateMetadata(this, () => {
            this.colorCfg.setBrightness(brightness);
        });
    };

    ImageFITS.prototype.setContrast = function (contrast) {
        updateMetadata(this, () => {
            this.colorCfg.setContrast(contrast);
        });
    };

    ImageFITS.prototype.metadata = function () {
        return {
            ...this.colorCfg.get(),
            longitudeReversed: false,
            imgFormat: this.imgFormat
        };
    }

    // Private method for updating the view with the new meta
    var updateMetadata = function (self, callback = undefined) {
        if (callback) {
            callback();
        }

        // Tell the view its meta have changed
        try {
            if (self.added) {
                const metadata = self.metadata();
                self.wasm.setImageMetadata(self.layer, metadata);

                ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(self.view.aladinDiv, { layer: self });
            }
        } catch (e) {
            // Display the error message
            console.error(e);
        }
    }

    ImageFITS.prototype.add = function (layer) {
        this.layer = layer;

        let self = this;
        const promise = self.wasm.addImageFITS({
            layer: self.layer,
            url: self.url,
            meta: self.metadata()
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
                    self.view,
                    null,
                    null,
                    null
                );

                // Set the layer corresponding to the onein the backend
                image.layer = imageParams.layer;
                image.added = true;
                // deep copy of the color object of self
                image.colorCfg = Utils.deepCopy(self.colorCfg);

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
            console.error(e)
            if (self.errorCallback) {
                self.errorCallback()
            }

            // This error result from a promise
            // If I throw it, it will not be catched because
            // it is run async
            self.view.removeImageLayer(layer)
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
        updateMetadata(this, () => {
            this.colorCfg = colorCfg;
        });
    };

    // @api
    ImageFITS.prototype.getColorCfg = function () {
        return this.colorCfg;
    };

    // @api
    ImageFITS.prototype.getOpacity = function () {
        return this.colorCfg.getOpacity();
    };

    ImageFITS.prototype.getAlpha = ImageFITS.prototype.getOpacity;

    // @api
    ImageFITS.prototype.readPixel = function (x, y) {
        return this.wasm.readPixel(x, y, this.layer);
    };

    return ImageFITS;
})();

