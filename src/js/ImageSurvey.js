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
import { Utils } from "./Utils";
import { ALEvent } from "./events/ALEvent.js";
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
        frame = "ICRS";
    } else if (frame == "galactic") {
        frame = "GAL";
    } else {
        frame = "ICRS";
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

    return [minCutout, maxCutout];
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

PropertyParser.isPlanetaryBody = function(options, properties = {}) {
    return properties.hips_body !== undefined;
}

/**
 * @typedef {Object} ImageSurveyOptions
 * 
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
 * @property {number} [maxOrder] - If not given, retrieved from the properties of the survey.
 * @property {number} [minOrder] - If not given, retrieved from the properties of the survey.
 * @property {boolean} [longitudeReversed=false] - Set it to True for planetary survey visualization
 * @property {string} [imgFormat] - If not given, look into the properties to see the accepted format. The format is chosen from PNG > WEBP > JPEG > FITS (in this importance order).
 * @property {string} [cooFrame] - If not given, look into the properties. If it is a planet, then ICRS is chosen, otherwise its hips_frame key is read. If no value is found in the properties, ICRS is chosen by default. 
 */
export let ImageSurvey = (function () {
    /**
     * The object describing an image survey
     *
     * @class
     * @constructs ImageSurvey
     * 
     * @param {string} [id] - Optional, a uniq id for the survey. See {@link https://aladin.cds.unistra.fr/hips/list|here} for the list of IDs.
     *      Keep in mind that it is better to directly provide an url as it will not request our mocserver first to get final survey tiles retrieval url.
     * @param {string} [name] - The name of the survey to be displayed in the UI
     * @param {string} url - The url where the survey is located. Check the hips list {@link https://aladin.cds.unistra.fr/hips/list|here} for the valid survey urls to display.
     * @param {ImageSurveyOptions} [options] - The option for the survey
     * 
     * @description Prefer provide an url better than an id. If both are given, the url will be requested first for the survey data.
     */
    function ImageSurvey(id, name, url, view, options) {
        this.view = view;
        this.wasm = view.wasm;
        this.added = false;
        this.id = id;
        this.name = name;
        this.subtype = "survey";

        this.properties = {};
        this.colorCfg = new ColorCfg(options);

        let self = this;
        self.query = (async () => {
            let obsTitle, creatorDid, maxOrder, frame, tileSize, formats, minCutout, maxCutout, bitpix, skyFraction, minOrder, initialFov, initialRa, initialDec, hipsBody, isPlanetaryBody, dataproductSubtype;

            try {
                let properties;
                try {
                    properties = await HiPSProperties.fetchFromUrl(url)
                        .catch(async (e) => {
                            // url not valid so we try with the id
                            try {
                                return await HiPSProperties.fetchFromID(id);
                            } catch(e) {
                                throw e;
                            }
                        })
                } catch(e) {
                    throw e;
                }

                obsTitle = properties.obs_title;
                creatorDid = properties.creator_did;
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
                        self._setUrl(url);
                    })
                    .catch(e => {
                        alert(e);
                        // the survey has been added so we remove it from the stack
                        self.view.removeImageLayer(self.layer)
                        throw e;
                    })

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
                isPlanetaryBody = PropertyParser.isPlanetaryBody(options, properties);
                if (properties.hips_body) {
                    hipsBody = properties.hips_body;
                }

                // TODO move that code out of here
                if (properties.hips_body !== undefined) {
                    if (self.view.options.showFrame) {
                        self.view.aladin.setFrame('J2000d');
                    }
                } /*else {
                    if (self.view.options.showFrame) {
                        const cooFrame = CooFrameEnum.fromString(self.view.options.cooFrame, CooFrameEnum.J2000);
                        let frameChoiceElt = document.querySelectorAll('.aladin-location > .aladin-frameChoice')[0];
                        frameChoiceElt.innerHTML = '<option value="' + CooFrameEnum.J2000.label + '" '
                            + (cooFrame == CooFrameEnum.J2000 ? 'selected="selected"' : '') + '>J2000</option><option value="' + CooFrameEnum.J2000d.label + '" '
                            + (cooFrame == CooFrameEnum.J2000d ? 'selected="selected"' : '') + '>J2000d</option><option value="' + CooFrameEnum.GAL.label + '" '
                            + (cooFrame == CooFrameEnum.GAL ? 'selected="selected"' : '') + '>GAL</option>';
                    }
                }*/
            } catch (e) {
                //console.error("Could not fetch properties for the survey ", self.id, " with the error:\n", e)
                /*if (!options.maxOrder) {
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
                frame = PropertyParser.frame(options);*/

                throw e;
            }

            self.properties = {
                creatorDid: creatorDid,
                obsTitle: obsTitle,
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
                isPlanetaryBody: isPlanetaryBody,
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

                if (imgFormat === "webp" && formats.indexOf('webp') < 0) {
                    throw self.name + " does not provide webp tiles";
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
                } else if (formats.indexOf('webp') >= 0) {
                    imgFormat = "webp";
                } else if (formats.indexOf('jpeg') >= 0) {
                    imgFormat = "jpeg";
                } else if (formats.indexOf('fits') >= 0) {
                    imgFormat = "fits";
                } else {
                    throw "Unsupported format(s) found in the properties: " + formats;
                }
            }

            self.imgFormat = imgFormat;

            // Initialize the color meta data here
            if (imgFormat === "fits") {
                // Take into account the default cuts given by the property file (this is true especially for FITS HiPSes)
                const minCut = self.colorCfg.minCut || (options && options.minCut) || self.properties.minCutout || 0.0;
                const maxCut = self.colorCfg.maxCut || (options && options.maxCut) || self.properties.maxCutout || 1.0;

                this.colorCfg.setCuts(minCut, maxCut);
            } else {
                const minCut = self.colorCfg.minCut || (options && options.minCut) || 0.0;
                const maxCut = self.colorCfg.maxCut || (options && options.maxCut) || 1.0;

                this.colorCfg.setCuts(minCut, maxCut);
            }

            // Color cuts
            //const lowCut = self.colorCfg.minCut || self.properties.minCutout || 0.0;
            //const highCut = self.colorCfg.maxCut || self.properties.maxCutout || 1.0;
            //self.setCuts(lowCut, highCut);

            ImageLayer.update(self);

            return self;
        })();
    };

    ImageSurvey.prototype._setUrl = function (url) {
        if (this.properties.url !== url) {
            console.info("Change url of ", this.id, " from ", this.properties.url, " to ", url)

            // If added to the backend, then we need to tell it the url has changed
            if (this.added) {
                this.wasm.setHiPSUrl(this.properties.url, url);
            }

            this.properties.url = url;
        }
    }
    /**
     * Checks if the ImageSurvey represents a planetary body.
     *
     * This method returns a boolean indicating whether the ImageSurvey corresponds to a planetary body, e.g. the earth or a celestial body. 
     *
     * @memberof ImageSurvey
     *
     * @returns {boolean} Returns true if the ImageSurvey represents a planetary body; otherwise, returns false.
     */
    ImageSurvey.prototype.isPlanetaryBody = function() {
        return this.properties.isPlanetaryBody;
    }

    /**
     * Sets the image format for the ImageSurvey.
     *
     * This method updates the image format of the ImageSurvey, performs format validation, and triggers the update of metadata.
     *
     * @memberof ImageSurvey
     *
     * @param {string} format - The desired image format. Should be one of ["fits", "png", "jpg", "webp"].
     *
     * @throws {string} Throws an error if the provided format is not one of the supported formats or if the format is not available for the specific ImageSurvey.
     */
    ImageSurvey.prototype.setImageFormat = function (format) {
        let self = this;
        self.query
            .then(() => {
                updateMetadata(self, () => {
                    let imgFormat = format.toLowerCase();

                    if (imgFormat !== "fits" && imgFormat !== "png" && imgFormat !== "jpg" && imgFormat !== "jpeg" && imgFormat !== "webp") {
                        throw 'Formats must lie in ["fits", "png", "jpg", "webp"]';
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

                    if (imgFormat === "webp" && availableFormats.indexOf('webp') < 0) {
                        throw self.id + " does not provide webp tiles";
                    }

                    if (imgFormat === "png" && availableFormats.indexOf('png') < 0) {
                        throw self.id + " does not provide png tiles";
                    }

                    if (imgFormat === "jpeg" && availableFormats.indexOf('jpeg') < 0) {
                        throw self.id + " does not provide jpeg tiles";
                    }

                    // Switch from png/webp/jpeg to fits
                    if ((self.imgFormat === 'png' || self.imgFormat === "webp" || self.imgFormat === "jpeg") && imgFormat === 'fits') {
                        if (self.properties.minCutout && self.properties.maxCutout) {
                            self.setCuts(self.properties.minCutout, self.properties.maxCutout)
                        }
                    // Switch from fits to png/webp/jpeg
                    } else if (self.imgFormat === "fits") {
                        self.setCuts(0.0, 1.0);
                    }

                    // Check if it is a fits
                    self.imgFormat = imgFormat;
                });
            })
    };

     /**
     * Sets the opacity factor when rendering the ImageSurvey
     *
     * @memberof ImageSurvey
     *
     * @returns {string[]} Returns the formats accepted for the survey, i.e. the formats of tiles that are availables. Could be PNG, WEBP, JPG and FITS.
     */
    ImageSurvey.prototype.getAvailableFormats = function() {
        return this.properties.formats;
    }

    /**
     * Sets the opacity factor when rendering the ImageSurvey
     *
     * @memberof ImageSurvey
     *
     * @param {number} opacity - Opacity of the survey to set. Between 0 and 1
     */
    ImageSurvey.prototype.setOpacity = function (opacity) {
        let self = this;
        updateMetadata(self, () => {
            self.colorCfg.setOpacity(opacity);
        });
    };

    /**
     * Sets the blending mode when rendering the ImageSurvey
     *
     * @memberof ImageSurvey
     *
     * @param {boolean} [additive=false] - 
     * 
     * @description Two rendering modes are availables i.e. the default one and the additive one.
     * When rendering this survey on top of the already rendered ones, the final color of the screen is computed like:
     * <br>
     * <br>opacity * this_survey_color + (1 - opacity) * already_rendered_color for the default mode
     * <br>opacity * this_survey_color + already_rendered_color for the additive mode
     * <br>
     * <br>
     * Additive mode allows you to do linear survey color combination i.e. let's define 3 surveys named s1, s2, s3. Each could be associated to one color channel, i.e. s1 with red, s2 with green and s3 with the blue color channel.
     * If the additive blending mode is enabled, then the final pixel color of your screen will be: rgb = [s1_opacity * s1_color; s2_opacity * s2_color; s3_opacity * s3_color]
     */
    ImageSurvey.prototype.setBlendingConfig = function (additive = false) {
        updateMetadata(this, () => {
            this.colorCfg.setBlendingConfig(additive);
        });
    };

    /**
     * Sets the colormap when rendering the ImageSurvey.
     *
     * @memberof ImageSurvey
     *
     * @param {string} [colormap="grayscale"] - The colormap label to use. See {@link https://matplotlib.org/stable/users/explain/colors/colormaps.html|here} for more info about colormaps.
     *      Possible values are:
     * <br>"blues"
     * <br>"cividis"
     * <br>"cubehelix"
     * <br>"eosb"
     * <br>"grayscale"
     * <br>"inferno"
     * <br>"magma"
     * <br>"native"
     * <br>"parula"
     * <br>"plasma"
     * <br>"rainbow"
     * <br>"rdbu"
     * <br>"rdylbu"
     * <br>"redtemperature"
     * <br>"sinebow"
     * <br>"spectral"
     * <br>"summer"
     * <br>"viridis"
     * <br>"ylgnbu"
     * <br>"ylorbr"
     * <br>"red"
     * <br>"green"
     * <br>"blue"
     * @param {Object} [options] - Options for the colormap
     * @param {string} [options.stretch] - Stretching function of the colormap. Possible values are 'linear', 'asinh', 'log', 'sqrt', 'pow'. If no given, will not change it.
     * @param {boolean} [options.reversed=false] - Reverse the colormap axis.
    */
    ImageSurvey.prototype.setColormap = function (colormap, options) {
        updateMetadata(this, () => {
            this.colorCfg.setColormap(colormap, options);
        });
    }

    /**
     * Sets the gamma correction factor for the ImageSurvey.
     *
     * This method updates the gamma of the ImageSurvey.
     *
     * @memberof ImageSurvey
     *
     * @param {number} lowCut - The low cut value to set for the ImageSurvey.
     * @param {number} highCut - The high cut value to set for the ImageSurvey.
     */
    ImageSurvey.prototype.setCuts = function (lowCut, highCut) {
        updateMetadata(this, () => {
            this.colorCfg.setCuts(lowCut, highCut);
        });
    };

    /**
     * Sets the gamma correction factor for the ImageSurvey.
     *
     * This method updates the gamma of the ImageSurvey.
     *
     * @memberof ImageSurvey
     *
     * @param {number} gamma - The saturation value to set for the ImageSurvey. Between 0.1 and 10
     */
    ImageSurvey.prototype.setGamma = function (gamma) {
        updateMetadata(this, () => {
            this.colorCfg.setGamma(gamma);
        });
    };

    /**
     * Sets the saturation for the ImageSurvey.
     *
     * This method updates the saturation of the ImageSurvey.
     *
     * @memberof ImageSurvey
     *
     * @param {number} saturation - The saturation value to set for the ImageSurvey. Between 0 and 1
     */
    ImageSurvey.prototype.setSaturation = function (saturation) {
        updateMetadata(this, () => {
            this.colorCfg.setSaturation(saturation);
        });
    };

    /**
     * Sets the brightness for the ImageSurvey.
     *
     * This method updates the brightness of the ImageSurvey.
     *
     * @memberof ImageSurvey
     *
     * @param {number} brightness - The brightness value to set for the ImageSurvey. Between 0 and 1
     */
    ImageSurvey.prototype.setBrightness = function (brightness) {
        updateMetadata(this, () => {
            this.colorCfg.setBrightness(brightness);
        });
    };

    /**
     * Sets the contrast for the ImageSurvey.
     *
     * This method updates the contrast of the ImageSurvey and triggers the update of metadata.
     *
     * @memberof ImageSurvey
     *
     * @param {number} contrast - The contrast value to set for the ImageSurvey. Between 0 and 1
     */
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
    var updateMetadata = function (self, callback) {
        if (callback) {
            callback();
        }

        // Tell the view its meta have changed
        try {
            if (self.added) {
                const metadata = self.metadata();
                self.wasm.setImageMetadata(self.layer, metadata);
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

        this.wasm.addImageSurvey({
            layer,
            properties: this.properties,
            meta: this.metadata(),
        });

        this.added = true;

        return Promise.resolve(this);
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
        return this.wasm.readPixel(x, y, this.layer);
    };

    ImageSurvey.DEFAULT_SURVEY_ID = "P/DSS2/color";

    return ImageSurvey;
})();

