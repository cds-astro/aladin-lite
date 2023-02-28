/******************************************************************************
 * Aladin Lite project
 * 
 * File MOC
 *
 * This class represents a MOC (Multi Order Coverage map) layer
 * 
 * Author: Thomas Boch[CDS], Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

import { Aladin }   from "./Aladin.js";
import { Utils }   from "./Utils.js";
import { Color } from "./Color";
import { ALEvent } from "./events/ALEvent.js";

export let MOC = (function() {
    let MOC = function(options) {
        //this.order = undefined;

        this.uuid = Utils.uuidv4();
        this.type = 'moc';

        // TODO homogenize options parsing for all kind of overlay (footprints, catalog, MOC)
        options = options || {};
        this.name = options.name || "MOC";

        this.color = options.color || Color.getNextColor();
        this.color = Color.standardizeColor(this.color);
        
        this.opacity = options.opacity || 1;

        this.opacity = Math.max(0, Math.min(1, this.opacity)); // 0 <= this.opacity <= 1
        this.lineWidth = options["lineWidth"] || 1;
        this.adaptativeDisplay = options['adaptativeDisplay'] !== false;

        //this.proxyCalled = false; // this is a flag to check whether we already tried to load the MOC through the proxy

        this.isShowing = true;
        this.ready = false;
        this.skyFrac = undefined;
    }

    /**
     *  Return a value between 0 and 1 denoting the fraction of the sky
     *  covered by the MOC
     */
    MOC.prototype.skyFraction = function() {
        return this.skyFrac;
    };

    /**
     * set MOC data by parsing a MOC serialized in JSON
     * (as defined in IVOA MOC document, section 3.1.1)
     */
    MOC.prototype.dataFromJSON = function(jsonMOC) {
        this.dataJSON = jsonMOC;
    };

    /**
     * set MOC data by parsing a URL pointing to a FITS MOC file
     */
    MOC.prototype.dataFromFITSURL = function(mocURL, successCallback) {
        this.dataURL = mocURL;
        this.promiseFetchData = fetch(this.dataURL)
            .then((resp) => resp.arrayBuffer());
        this.successCallback = successCallback;
    };

    MOC.prototype.setView = function(view) {
        let self = this;

        this.view = view;
        this.mocParams = new Aladin.wasmLibs.core.MOC(this.uuid, this.opacity, this.lineWidth, this.isShowing, this.color, this.adaptativeDisplay);

        if (this.dataURL) {
            this.promiseFetchData
                .then((arrayBuffer) => {
                    // Add the fetched moc to the rust backend
                    self.view.wasm.addFITSMoc(self.mocParams, new Uint8Array(arrayBuffer));
                    self.ready = true;

                    if (self.successCallback) {
                        self.successCallback(self)
                    }

                    // Cache the sky fraction
                    self.skyFrac = self.view.wasm.mocSkyFraction(this.mocParams);

                    // Add it to the view
                    self.view.mocs.push(self);
                    self.view.allOverlayLayers.push(self);

                    // Tell the MOC has been fully loaded and can be sent as an event
                    ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.dispatchedTo(self.view.aladinDiv, {layer: self});

                    self.view.requestRedraw();
                })
        } else if (this.dataFromJSON) {
            self.view.wasm.addJSONMoc(self.mocParams, self.dataJSON);
            self.ready = true;

            // Cache the sky fraction
            self.skyFrac = self.view.wasm.mocSkyFraction(self.mocParams);

            // Add it to the view
            self.view.mocs.push(self);
            self.view.allOverlayLayers.push(self);

            // Tell the MOC has been fully loaded and can be sent as an event
            ALEvent.GRAPHIC_OVERLAY_LAYER_ADDED.dispatchedTo(self.view.aladinDiv, {layer: self});

            self.view.requestRedraw();
        }
    };

    MOC.prototype.reportChange = function() {
        if (this.view) {
            // update the new moc params to the backend
            this.mocParams = new Aladin.wasmLibs.core.MOC(this.uuid, this.opacity, this.lineWidth, this.isShowing, this.color, this.adaptativeDisplay);
            this.view.wasm.setMocParams(this.mocParams);
            this.view.requestRedraw();
        }
    };

    MOC.prototype.delete = function() {
        if (this.view) {
            // update the new moc params to the backend
            this.view.wasm.removeMoc(this.mocParams);
            this.view.requestRedraw();
        }
    };

    MOC.prototype.show = function() {
        if (this.isShowing) {
            return;
        }
        this.isShowing = true;
        this.reportChange();
    };

    MOC.prototype.hide = function() {
        if (! this.isShowing) {
            return;
        }
        this.isShowing = false;
        this.reportChange();
    };

    // Tests whether a given (ra, dec) point on the sky is within the current MOC object
    //
    // returns true if point is contained, false otherwise
    MOC.prototype.contains = function(ra, dec) {
        if (!this.ready) {
            throw this.name + " is not yet ready, either because it has not been downloaded yet or because it has not been added to the aladin instance."
        }

        // update the new moc params to the backend
        return this.view.wasm.mocContains(this.mocParams, ra, dec);
    };

    return MOC;

})();

    
