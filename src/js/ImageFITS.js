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
 
 export let ImageFITS = (function() {
     /** Constructor
      * cooFrame and maxOrder can be set to null
      * They will be determined by reading the properties file
      *  
      */
     function ImageFITS(url, view, options) {
        // A reference to the view
        this.backend = view;
        // Name of the layer
        this.layer = null;
        this.added = false;
        // Set it to a default value
        this.url = url;
        // initialize the color meta data here
        this.colorCfg = new ColorCfg(options);
        updateMetadata(this);
 
        let self = this;
        this.queryPromise = (async () => {
            // Set the cuts at this point, if the user gave one, keep it
            // otherwise, set it to default values found in the HiPS properties
            // For FITS hipses
            let minCut = self.colorCfg.minCut || 0.0;
            let maxCut = self.colorCfg.maxCut || 1.0;
            self.setCuts(minCut, maxCut);
         })();
     };
 
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
             imgFormat: 'fits'
         };
     }
 
     // Private method for updating the backend with the new meta
     var updateMetadata = function(self, callback = undefined) {        
         if (callback) {
             callback();
         }
 
         // Tell the view its meta have changed
         try {
             if( self.added ) {
                 const metadata = self.metadata();
                 self.backend.aladin.webglAPI.setImageMetadata(self.layer, metadata);
                 // once the meta have been well parsed, we can set the meta 
                 ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(self.backend.aladinDiv, {survey: self});
             }
         } catch(e) {
             // Display the error message
             console.error(e);
         }
     }
 
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
         return this.backend.aladin.webglAPI.readPixel(x, y, this.layer);
     };
 
     return ImageFITS;
 })();
 
 