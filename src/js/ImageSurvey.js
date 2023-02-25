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
 * File ImageSurvey
 * 
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 * 
 *****************************************************************************/
import { Utils } from "./Utils.js";
import { ALEvent } from "./events/ALEvent.js";
import { CooFrameEnum } from "./CooFrameEnum.js"
import { ColorCfg } from "./ColorCfg.js";
import { ImageLayer } from "./ImageLayer.js";
import { HiPSProperties } from "./HiPSProperties.js";

let PropertyParser = {};
// Utilitary functions for parsing the properties and giving default values
/// Mandatory tileSize property
PropertyParser.tileSize = function(options, properties = {}) {
    let tileSize = (options && options.tileSize) || (properties.hips_tile_width && (+properties.hips_tile_width)) || 512;

    // Check if the tile width size is a power of 2
    if (tileSize & (tileSize - 1) !== 0) {
        tileSize = 512;
    }

    return tileSize;
}

/// Mandatory frame property
PropertyParser.frame = function(options, properties = {}) {
    let frame = (options && options.cooFrame) || (properties.hips_body && "ICRSd") || properties.hips_frame;

    if (frame == "ICRS" || frame == "ICRSd" || frame == "equatorial" || frame == "j2000") {
        frame = "ICRSJ2000";
    } else if (frame == "galactic") {
        frame = "GAL";
    } else if (frame === undefined) {
        frame = "ICRSJ2000";
        console.warn('No cooframe given. Coordinate systems supported: "ICRS", "ICRSd", "j2000" or "galactic". ICRS is chosen by default');
    } else {
        frame = "ICRSd";
        console.warn('Invalid cooframe given: ' + cooFrame + '. Coordinate systems supported: "ICRS", "ICRSd", "j2000" or "galactic". ICRS is chosen by default');
    }

    return frame;
}

/// Mandatory maxOrder property
PropertyParser.maxOrder = function(options, properties = {}) {
    let maxOrder = (options && options.maxOrder) || (properties.hips_order && (+properties.hips_order));
    return maxOrder;
}

/// Mandatory minOrder property
PropertyParser.minOrder = function(options, properties = {}) {
    const minOrder = (options && options.minOrder) || (properties.hips_order_min && (+properties.hips_order_min)) || 0;
    return minOrder;
}

PropertyParser.formats = function(options, properties = {}) {
    let formats = properties.hips_tile_format || "jpeg";

    formats = formats.split(' ')
        .map((fmt) => fmt.toLowerCase());

    return formats;
}

PropertyParser.initialFov = function(options, properties = {}) {
    let initialFov = properties.hips_initial_fov && +properties.hips_initial_fov;

    if (initialFov && initialFov < 0.00002777777) {
        initialFov = 360;
    }

    return initialFov;
}

PropertyParser.skyFraction = function(options, properties = {}) {
    const skyFraction = (properties.moc_sky_fraction && (+properties.moc_sky_fraction)) || 0.0;
    return skyFraction;
}

PropertyParser.cutouts = function(options, properties = {}) {
    let cuts = properties.hips_pixel_cut && properties.hips_pixel_cut.split(" ");

    const minCutout = cuts && parseFloat(cuts[0]);
    const maxCutout = cuts && parseFloat(cuts[1]);

    return [minCutout, maxCutout]
}

PropertyParser.bitpix = function(options, properties = {}) {
    const bitpix = properties.hips_pixel_bitpix && (+properties.hips_pixel_bitpix);
    return bitpix;
}

PropertyParser.dataproductSubtype = function(options, properties = {}) {
    let dataproductSubtype = properties.dataproduct_subtype || "color";
    dataproductSubtype = dataproductSubtype.split(" ")
        .map((subtype) => subtype.toLowerCase());
    return dataproductSubtype;
}

PropertyParser.hipsBody = function(options, properties = {}) {
    return properties.hips_body !== undefined;
}

export let ImageSurvey = (function () {
    /** Constructor
     * cooFrame and maxOrder can be set to null
     * They will be determined by reading the properties file
     *  
     */
    function ImageSurvey(id, name, url, view, options) {
        // A reference to the view
        this.view = view;
        this.added = false;
        this.id = id;
        this.name = name;

        // initialize the color meta data here
        this.colorCfg = new ColorCfg(options);

        this.properties = {};

        let self = this;
        self.query = (async () => {
            let maxOrder, frame, tileSize, formats, minCutout, maxCutout, bitpix, skyFraction, minOrder, initialFov, initialRa, initialDec, hipsBody, dataproductSubtype;

            try {
                const properties = await HiPSProperties.fetch(url || id);
                // Give a better name if we have the HiPS metadata
                self.name = self.name || properties.obs_title;
                // Set it to a default value
                if (!properties.hips_service_url) {
                    throw 'no valid service URL for retrieving the tiles'
                }
                url = Utils.fixURLForHTTPS(properties.hips_service_url);

                // Request all the properties to see which mirror is the fastest
                HiPSProperties.getFasterMirrorUrl(properties)
                    .then((url) => {
                        self.setUrl(url);
                    });

                // Max order
                maxOrder = PropertyParser.maxOrder(options, properties);

                // Tile size
                tileSize = PropertyParser.tileSize(options, properties);

                // Tile formats
                formats = PropertyParser.formats(options, properties);

                // min order
                minOrder = PropertyParser.minOrder(options, properties);

                // Frame
                frame = PropertyParser.frame(options, properties);

                // sky fraction
                skyFraction = PropertyParser.skyFraction(options, properties);

                // Initial fov/ra/dec
                initialFov = PropertyParser.initialFov(options, properties);
                initialRa = +properties.hips_initial_ra;
                initialDec = +properties.hips_initial_dec;

                // Cutouts
                [minCutout, maxCutout] = PropertyParser.cutouts(options, properties);

                // Bitpix
                bitpix = PropertyParser.bitpix(options, properties);

                // Dataproduct subtype
                dataproductSubtype = PropertyParser.dataproductSubtype(options, properties);

                // HiPS body
                hipsBody = PropertyParser.hipsBody(options, properties);

                // TODO move that code out of here
                if (properties.hips_body !== undefined) {
                    if (self.view.options.showFrame) {
                        self.view.aladin.setFrame('J2000d');
                        let frameChoiceElt = document.querySelectorAll('.aladin-location > .aladin-frameChoice')[0];
                        frameChoiceElt.innerHTML = '<option value="' + CooFrameEnum.J2000d.label + '" selected="selected">J2000d</option>';
                    }
                } else {
                    if (self.view.options.showFrame) {
                        const cooFrame = CooFrameEnum.fromString(self.view.options.cooFrame, CooFrameEnum.J2000);
                        let frameChoiceElt = document.querySelectorAll('.aladin-location > .aladin-frameChoice')[0];
                        frameChoiceElt.innerHTML = '<option value="' + CooFrameEnum.J2000.label + '" '
                            + (cooFrame == CooFrameEnum.J2000 ? 'selected="selected"' : '') + '>J2000</option><option value="' + CooFrameEnum.J2000d.label + '" '
                            + (cooFrame == CooFrameEnum.J2000d ? 'selected="selected"' : '') + '>J2000d</option><option value="' + CooFrameEnum.GAL.label + '" '
                            + (cooFrame == CooFrameEnum.GAL ? 'selected="selected"' : '') + '>GAL</option>';
                    }
                }
            } catch (e) {
                console.error("Could not fetch properties for the survey ", self.id, " with the error:\n", e)
                if (!options.maxOrder) {
                    throw "The max order is mandatory for a HiPS."
                }

                if (!options.tileSize) {
                    console.warn("The tile size has not been given, 512 is chosen by default");
                }

                url = Utils.fixURLForHTTPS(url);

                // Max order
                maxOrder = PropertyParser.maxOrder(options);

                // Tile size
                tileSize = PropertyParser.tileSize(options);

                // Tile formats
                formats = PropertyParser.formats(options);

                // min order
                minOrder = PropertyParser.minOrder(options);

                // Frame
                frame = PropertyParser.frame(options);
            }

            self.properties = {
                url: url,
                maxOrder: maxOrder,
                frame: frame,
                tileSize: tileSize,
                formats: formats,
                minCutout: minCutout,
                maxCutout: maxCutout,
                bitpix: bitpix,
                skyFraction: skyFraction,
                minOrder: minOrder,
                hipsInitialFov: initialFov,
                hipsInitialRa: initialRa,
                hipsInitialDec: initialDec,
                dataproductSubtype: dataproductSubtype,
                hipsBody: hipsBody
            };

            // Use the property to define and check some user given infos
            // Longitude reversed
            let longitudeReversed = false;
            if (options && options.longitudeReversed === true) {
                longitudeReversed = true;
            }

            if (self.properties.hipsBody) {
                longitudeReversed = true;
            }

            self.longitudeReversed = longitudeReversed;

            // Image format
            let imgFormat = options && options.imgFormat;
            if (imgFormat) {
                // transform to lower case
                imgFormat = imgFormat.toLowerCase();
                // convert JPG -> JPEG
                if (imgFormat === "jpg") {
                    imgFormat = "jpeg";
                }

                // user wants a fits but the properties tells this format is not available
                if (imgFormat === "fits" && formats.indexOf('fits') < 0) {
                    throw self.name + " does not provide fits tiles";
                }

                if (imgFormat === "png" && formats.indexOf('png') < 0) {
                    throw self.name + " does not provide png tiles";
                }

                if (imgFormat === "jpeg" && formats.indexOf('jpeg') < 0) {
                    throw self.name + " does not provide jpeg tiles";
                }
            } else {
                // user wants nothing then we choose one from the properties
                if (formats.indexOf('png') >= 0) {
                    imgFormat = "png";
                } else if (formats.indexOf('jpeg') >= 0) {
                    imgFormat = "jpeg";
                } else if (formats.indexOf('fits') >= 0) {
                    imgFormat = "fits";
                } else {
                    throw "Unsupported format(s) found in the properties: " + formats;
                }
            }

            self.imgFormat = imgFormat;

            // Color cuts
            //const lowCut = self.colorCfg.minCut || self.properties.minCutout || 0.0;
            //const highCut = self.colorCfg.maxCut || self.properties.maxCutout || 1.0;
            //self.setCuts(lowCut, highCut);

            ImageLayer.update(self);

            return self;
        })();
    };

    ImageSurvey.prototype.setUrl = function (url) {
        if (this.properties.url !== url) {
            console.info("Change url of ", this.id, " from ", this.properties.url, " to ", url)

            // If added to the backend, then we need to tell it the url has changed
            if (this.added) {
                this.view.aladin.webglAPI.setHiPSUrl(this.properties.url, url);
            }

            this.properties.url = url;
        }
    }

    // @api
    // TODO: include imgFormat inside the ImageSurvey's meta attribute
    ImageSurvey.prototype.setImageFormat = function (format) {
        let self = this;
        self.query
            .then(() => {
                updateMetadata(self, () => {
                    let imgFormat = format.toLowerCase();
        
                    if (imgFormat !== "fits" && imgFormat !== "png" && imgFormat !== "jpg" && imgFormat !== "jpeg") {
                        throw 'Formats must lie in ["fits", "png", "jpg"]';
                    }
        
                    if (imgFormat === "jpg") {
                        imgFormat = "jpeg";
                    }
        
                    // Passed the check, we erase the image format with the new one
                    // We do nothing if the imgFormat is the same
                    if (self.imgFormat === imgFormat) {
                        return;
                    }
        
                    // Check the properties to see if the given format is available among the list
                    // If the properties have not been retrieved yet, it will be tested afterwards
                    const availableFormats = self.properties.formats;
                    // user wants a fits but the metadata tells this format is not available
                    if (imgFormat === "fits" && availableFormats.indexOf('fits') < 0) {
                        throw self.id + " does not provide fits tiles";
                    }
        
                    if (imgFormat === "png" && availableFormats.indexOf('png') < 0) {
                        throw self.id + " does not provide png tiles";
                    }
        
                    if (imgFormat === "jpeg" && availableFormats.indexOf('jpeg') < 0) {
                        throw self.id + " does not provide jpeg tiles";
                    }
        
                    // Check if it is a fits
                    self.imgFormat = imgFormat;
    
    
                    console.log("change image format", self.imgFormat)
    
                });
            })
    };

    // @api
    ImageSurvey.prototype.setOpacity = function (opacity) {
        let self = this;
        updateMetadata(self, () => {
            self.colorCfg.setOpacity(opacity);
        });
    };

    // @api
    ImageSurvey.prototype.setBlendingConfig = function (additive = false) {
        updateMetadata(this, () => {
            this.colorCfg.setBlendingConfig(additive);
        });
    };

    // @api
    ImageSurvey.prototype.setColormap = function (colormap, options) {
        updateMetadata(this, () => {
            this.colorCfg.setColormap(colormap, options);
        });
    }

    // @api
    ImageSurvey.prototype.setCuts = function (lowCut, highCut) {
        updateMetadata(this, () => {
            this.colorCfg.setCuts(lowCut, highCut);
        });
    };

    // @api
    ImageSurvey.prototype.setGamma = function (gamma) {
        updateMetadata(this, () => {
            this.colorCfg.setGamma(gamma);
        });
    };

    // @api
    ImageSurvey.prototype.setSaturation = function (saturation) {
        updateMetadata(this, () => {
            this.colorCfg.setSaturation(saturation);
        });
    };

    ImageSurvey.prototype.setBrightness = function (brightness) {
        updateMetadata(this, () => {
            this.colorCfg.setBrightness(brightness);
        });
    };

    ImageSurvey.prototype.setContrast = function (contrast) {
        updateMetadata(this, () => {
            this.colorCfg.setContrast(contrast);
        });
    };

    ImageSurvey.prototype.metadata = function () {
        return {
            ...this.colorCfg.get(),
            longitudeReversed: this.longitudeReversed,
            imgFormat: this.imgFormat
        };
    }

    // Private method for updating the backend with the new meta
    var updateMetadata = function (self, callback = undefined) {
        if (callback) {
            callback();
        }

        // Tell the view its meta have changed
        try {
            if (self.added) {
                const metadata = self.metadata();
                self.view.aladin.webglAPI.setImageMetadata(self.layer, metadata);
                // once the meta have been well parsed, we can set the meta 
                ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(self.view.aladinDiv, { layer: self });
            }
        } catch (e) {
            // Display the error message
            console.error(e);
        }
    }

    ImageSurvey.prototype.add = function (layer) {
        this.layer = layer;
        /*console.log({
            layer: this.layer,
            properties: this.properties,
            meta: this.metadata(),
        })*/
        this.view.aladin.webglAPI.addImageSurvey({
            layer: this.layer,
            properties: this.properties,
            meta: this.metadata(),
        });

        this.added = true;
    }

    // @api
    ImageSurvey.prototype.toggle = function () {
        if (this.colorCfg.getOpacity() != 0.0) {
            this.colorCfg.setOpacity(0.0);
        } else {
            this.colorCfg.setOpacity(this.prevOpacity);
        }
    };

    // @oldapi
    ImageSurvey.prototype.setAlpha = ImageSurvey.prototype.setOpacity;

    ImageSurvey.prototype.setColorCfg = function (colorCfg) {
        updateMetadata(this, () => {
            this.colorCfg = colorCfg;
        });
    };

    // @api
    ImageSurvey.prototype.getColorCfg = function () {
        return this.colorCfg;
    };

    // @api
    ImageSurvey.prototype.getOpacity = function () {
        return this.colorCfg.getOpacity();
    };

    ImageSurvey.prototype.getAlpha = ImageSurvey.prototype.getOpacity;

    // @api
    ImageSurvey.prototype.readPixel = function (x, y) {
        return this.view.aladin.webglAPI.readPixel(x, y, this.layer);
    };

    ImageSurvey.DEFAULT_SURVEY_ID = "P/DSS2/color";

    return ImageSurvey;
})();

