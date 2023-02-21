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
 import { Utils } from "./Utils.js";
 import { ALEvent } from "./events/ALEvent.js";
 import { ColorCfg } from "./ColorCfg.js";
 import { ImageSurvey } from "./ImageSurvey.js";
 import { Aladin } from "./Aladin.js";

 export let ImageFITS = (function() {
     /** Constructor
      * cooFrame and maxOrder can be set to null
      * They will be determined by reading the properties file
      *  
      */
     function ImageFITS(url, view, options, successCallback = undefined, errorCallback = undefined) {
        this.view = view;
        // Name of the layer
        this.layer = null;
        this.added = false;
        // Set it to a default value
        try {
            this.url = new URL(url);
        } catch(e) {
            // The url could be created
            url = Utils.getAbsoluteURL(url)
            this.url = new URL(url);
        }
        let init = {};
        console.log(this.url)
        // Check the protocol, for http ones, use a CORS compatible proxy
        if (this.url.protocol === "file:") {
            init = { mode: 'no-cors'};
        } else if ((Utils.isHttpContext() || Utils.isHttpsContext()) && this.url.hostname !== "localhost") {
            // http(s) protocols and not in localhost
            let url = new URL(Aladin.JSONP_PROXY);
            url.searchParams.append("url", this.url);

            this.url = url;
        }

        console.log(this.url)

        this.id = this.url.toString();
        this.name = this.id;

        this.imgFormat = "fits";
        this.formats = ["fits"];
        // callbacks
        this.successCallback = successCallback;
        this.errorCallback = errorCallback;
        // initialize the color meta data here
        this.colorCfg = new ColorCfg(options);
        updateMetadata(this);
 
        let idxSelectedHiPS = 0;
        const surveyFound = ImageSurvey.SURVEYS.some(s => {
            let res = this.id.endsWith(s.id);
            if (!res) {
                idxSelectedHiPS += 1;
            }

            return res;
        });
        const opt = {
            ...this.colorCfg.get(),
            imgFormat: this.imgFormat,
            longitudeReversed: this.longitudeReversed,
        };
        // The survey has not been found among the ones cached
        if (!surveyFound) {
            ImageSurvey.SURVEYS.push({
                id: this.id,
                name: this.name,
                options: opt,
            });
        } else {
            let surveyDef = ImageSurvey.SURVEYS[idxSelectedHiPS];
            surveyDef.options = opt;
        }

        let self = this;
        // Return a promise that take the layer name as parameter
        // and when resolved, will return the ImageFITS object
        self.query = (async () => {
            return fetch(this.url, init)
                .then((resp) => resp.arrayBuffer())
                .then((arrayBuffer) => {
                    console.log(arrayBuffer)
                    self.arrayBuffer = new Uint8Array(arrayBuffer);
                    return self;
                });
        });
    }
 
     // @api
     ImageFITS.prototype.setOpacity = function(opacity) {
         let self = this;
         updateMetadata(self, () => {
             self.colorCfg.setOpacity(opacity);
         });
     };
 
     // @api
     ImageFITS.prototype.setBlendingConfig = function(additive = false) {
         updateMetadata(this, () => {
             this.colorCfg.setBlendingConfig(additive);
         });
     };
 
     // @api
     ImageFITS.prototype.setColormap = function(colormap, options) {
         updateMetadata(this, () => {
             this.colorCfg.setColormap(colormap, options);
         });
     }
 
     // @api
     ImageFITS.prototype.setCuts = function(lowCut, highCut) {
         updateMetadata(this, () => {
             this.colorCfg.setCuts(lowCut, highCut);
         });
     };
 
     // @api
     ImageFITS.prototype.setGamma = function(gamma) {
         updateMetadata(this, () => {
             this.colorCfg.setGamma(gamma);
         });
     };
 
     // @api
     ImageFITS.prototype.setSaturation = function(saturation) {
         updateMetadata(this, () => {
             this.colorCfg.setSaturation(saturation);
         });
     };
 
     ImageFITS.prototype.setBrightness = function(brightness) {
         updateMetadata(this, () => {
             this.colorCfg.setBrightness(brightness);
         });
     };
 
    ImageFITS.prototype.setContrast = function(contrast) {
        updateMetadata(this, () => {
            this.colorCfg.setContrast(contrast);
        });
    };
 
    ImageFITS.prototype.metadata = function() {
        return {
            ...this.colorCfg.get(),
            longitudeReversed: false,
            imgFormat: this.imgFormat
        };
    }
 
     // Private method for updating the view with the new meta
     var updateMetadata = function(self, callback = undefined) {        
         if (callback) {
             callback();
         }
 
         // Tell the view its meta have changed
         try {
             if( self.added ) {
                 const metadata = self.metadata();
                 self.view.aladin.webglAPI.setImageMetadata(self.layer, metadata);
                 // once the meta have been well parsed, we can set the meta 
                 ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(self.view.aladinDiv, {survey: self});
             }
         } catch(e) {
             // Display the error message
             console.error(e);
         }
     }
 
    ImageFITS.prototype.add = function(layer) {
        this.layer = layer;

        let self = this;
        try {
            const {ra, dec, fov} = this.view.aladin.webglAPI.addImageFITS({
                    layer: self.layer,
                    url: self.url.toString(),
                    meta: self.metadata()
                }, 
                self.arrayBuffer,
            );

            this.ra = ra;
            this.dec = dec;
            this.fov = fov;

            this.added = true;

            // execute the callback if there are
            if (this.successCallback) {
                this.successCallback(self.ra, self.dec, self.fov, self);
            }
        } catch (e) {
            if (this.errorCallback) {
                this.errorCallback();
            }

            // Error propagation
            throw e;
        }
    };

     // @api
     ImageFITS.prototype.toggle = function() {
         if (this.colorCfg.getOpacity() != 0.0) {
             this.colorCfg.setOpacity(0.0);
         } else {
             this.colorCfg.setOpacity(this.prevOpacity);
         }
     };
 
     // @oldapi
     ImageFITS.prototype.setAlpha = ImageFITS.prototype.setOpacity;
 
     ImageFITS.prototype.setColorCfg = function(colorCfg) {
         updateMetadata(this, () => {
             this.colorCfg = colorCfg;
         });
     };
 
     // @api
     ImageFITS.prototype.getColorCfg = function() {
         return this.colorCfg;
     };
     
     // @api
     ImageFITS.prototype.getOpacity = function() {
         return this.colorCfg.getOpacity();
     };
 
     ImageFITS.prototype.getAlpha = ImageFITS.prototype.getOpacity;
 
     // @api
     ImageFITS.prototype.readPixel = function(x, y) {
         return this.view.aladin.webglAPI.readPixel(x, y, this.layer);
     };
 
     return ImageFITS;
 })();
 
 