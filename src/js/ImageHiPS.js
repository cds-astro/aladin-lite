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
 * File ImageHiPS
 *
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 *
 *****************************************************************************/
import { Utils } from "./Utils";
import { ALEvent } from "./events/ALEvent.js";
import { ColorCfg } from "./ColorCfg.js";
import { HiPSProperties } from "./HiPSProperties.js";
import { HiPSCache } from "./DefaultHiPSCache.js";

let PropertyParser = {};
// Utilitary functions for parsing the properties and giving default values
/// Mandatory tileSize property
PropertyParser.tileSize = function (properties) {
    let tileSize =
        (properties &&
            properties.hips_tile_width &&
            +properties.hips_tile_width) ||
        512;

    // Check if the tile width size is a power of 2
    if (tileSize & (tileSize - 1 !== 0)) {
        tileSize = 512;
    }

    return tileSize;
};

/// Mandatory frame property
PropertyParser.cooFrame = function (properties) {
    let cooFrame =
        (properties && properties.hips_body && "ICRSd") ||
        (properties && properties.hips_frame) ||
        "ICRS";
    return cooFrame;
};

/// Mandatory maxOrder property
PropertyParser.maxOrder = function (properties) {
    let maxOrder =
        properties && properties.hips_order && +properties.hips_order;
    return maxOrder;
};

/// Mandatory minOrder property
PropertyParser.minOrder = function (properties) {
    const minOrder =
        (properties &&
            properties.hips_order_min &&
            +properties.hips_order_min) ||
        0;
    return minOrder;
};

PropertyParser.formats = function (properties) {
    let formats = (properties && properties.hips_tile_format) || "jpeg";

    formats = formats.split(" ").map((fmt) => fmt.toLowerCase());

    return formats;
};

PropertyParser.initialFov = function (properties) {
    let initialFov =
        properties &&
        properties.hips_initial_fov &&
        +properties.hips_initial_fov;

    if (initialFov && initialFov < 0.00002777777) {
        initialFov = 360;
    }

    return initialFov;
};

PropertyParser.skyFraction = function (properties) {
    const skyFraction =
        (properties &&
            properties.moc_sky_fraction &&
            +properties.moc_sky_fraction) ||
        0.0;
    return skyFraction;
};

PropertyParser.cutouts = function (properties) {
    let cuts =
        properties &&
        properties.hips_pixel_cut &&
        properties.hips_pixel_cut.split(" ");

    const minCutout = cuts && parseFloat(cuts[0]);
    const maxCutout = cuts && parseFloat(cuts[1]);

    return [minCutout, maxCutout];
};

PropertyParser.bitpix = function (properties) {
    const bitpix =
        properties &&
        properties.hips_pixel_bitpix &&
        +properties.hips_pixel_bitpix;
    return bitpix;
};

PropertyParser.isPlanetaryBody = function (properties) {
    return properties && properties.hips_body !== undefined;
};

/**
 * @typedef {Object} ImageHiPSOptions
 *
 * @property {string} [name] - The name of the survey to be displayed in the UI
 * @property {Function} [successCallback] - A callback executed when the HiPS has been loaded
 * @property {Function} [errorCallback] - A callback executed when the HiPS could not be loaded
 * @property {string} [imgFormat] - Formats accepted 'webp', 'png', 'jpeg' or 'fits'. Will raise an error if the HiPS does not contain tiles in this format
 * @property {CooFrame} [cooFrame="J2000"] - Coordinate frame of the survey tiles
 * @property {number} [maxOrder] - The maximum HEALPix order of the HiPS, i.e the HEALPix order of the most refined tile images of the HiPS.
 * @property {number} [numBitsPerPixel] - Useful if you want to display the FITS tiles of a HiPS. It specifies the number of bits per pixel. Possible values are:
 * -64: double, -32: float, 8: unsigned byte, 16: short, 32: integer 32 bits, 64: integer 64 bits
 * @property {number} [tileSize] - The width of the HEALPix tile images. Mostly 512 pixels but can be 256, 128, 64, 32
 * @property {number} [minOrder] - If not given, retrieved from the properties of the survey.
 * @property {boolean} [longitudeReversed=false] - Set it to True for planetary survey visualization 
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
 */
export let ImageHiPS = (function () {
    /**
     * The object describing an image survey
     *
     * @class
     * @constructs ImageHiPS
     *
     * @param {string} id - Mandatory unique identifier for the layer. Can be an arbitrary name
     * @param {string} url - Can be an url to the survey or a "CDS" ID pointing towards a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}
     * @param {ImageHiPSOptions} [options] - The option for the survey
     *
     * @description Giving a CDS ID will do a query to the MOCServer first to retrieve metadata. Then it will also check for the presence of faster HiPS nodes to choose a faster url to query to tiles from.
     */
    function ImageHiPS(id, url, options) {
        this.added = false;
        // Unique identifier for a survey
        this.id = id;
        this.name = (options && options.name) || undefined;
        this.url = url;
        this.maxOrder = options.maxOrder;
        this.minOrder = options.minOrder || 0;
        this.cooFrame = options.cooFrame;
        this.tileSize = options.tileSize;
        this.skyFraction = options.skyFraction;
        this.longitudeReversed =
            options.longitudeReversed === undefined
                ? false
                : options.longitudeReversed;
        this.imgFormat = options.imgFormat;
        this.numBitsPerPixel = options.numBitsPerPixel;
        this.creatorDid = options.creatorDid;
        this.errorCallback = options.errorCallback;
        this.successCallback = options.successCallback;

        this.colorCfg = new ColorCfg(options);
    }

    ImageHiPS.prototype.setView = function (view) {
        let self = this;

        // do not allow to call setView multiple times otherwise
        // the querying to the properties and the search to the best
        // HiPS node will be done again for the same imageHiPS
        if (self.view) {
            return;
        }

        self.view = view;

        let isMOCServerToBeQueried = true;
        if (this.imgFormat === "fits") {
            // a fits is given
            isMOCServerToBeQueried = !(
                this.maxOrder &&
                this.url &&
                this.imgFormat &&
                this.tileSize &&
                this.cooFrame &&
                this.numBitsPerPixel
            );
        } else {
            isMOCServerToBeQueried = !(
                this.maxOrder &&
                this.url &&
                this.imgFormat &&
                this.tileSize &&
                this.cooFrame
            );
        }

        self.query = (async () => {
            if (isMOCServerToBeQueried) {
                let isCDSId = false;

                let properties = await HiPSProperties.fetchFromUrl(self.url)
                    /*.catch((e) => {
                        // try with the proxy
                        url = Utils.handleCORSNotSameOrigin(url).href;

                        return HiPSProperties.fetchFromUrl(url);
                    })*/
                    .catch(async (e) => {
                        // url not valid so we try with the id
                        try {
                            isCDSId = true;
                            // the url stores a "CDS ID" we take it prioritaly
                            // if the url is null, take the id, this is for some tests
                            // to pass because some users might just give null as url param and a "CDS ID" as id param
                            let id = self.url || self.id;
                            return await HiPSProperties.fetchFromID(id);
                        } catch (e) {
                            throw e;
                        }
                    });

                //obsTitle = properties.obs_title;
                self.creatorDid = properties.creator_did || self.creatorDid;
                // url

                if (isCDSId) {
                    self.url = properties.hips_service_url;
                    if (!self.url) {
                        throw "no valid service URL for retrieving the tiles";
                    }

                    self.url = Utils.fixURLForHTTPS(self.url);

                    // Request all the properties to see which mirror is the fastest
                    HiPSProperties.getFasterMirrorUrl(properties, self.url)
                        .then((url) => {
                            if (self.url !== url) {
                                console.info(
                                    "Change url of ",
                                    self.id,
                                    " from ",
                                    self.url,
                                    " to ",
                                    url
                                );

                                self.url = url;

                                // save the new url to the cache
                                self._saveInCache();

                                // If added to the backend, then we need to tell it the url has changed
                                if (self.added) {
                                    self.view.wasm.setHiPSUrl(
                                        self.creatorDid,
                                        url
                                    );
                                }
                            }
                        })
                        .catch((e) => {
                            console.error(self);
                            console.error(e);
                        });
                }

                // Max order
                self.maxOrder =
                    PropertyParser.maxOrder(properties) || self.maxOrder;

                // Tile size
                self.tileSize =
                    PropertyParser.tileSize(properties) || self.tileSize;

                // Tile formats
                self.formats =
                    PropertyParser.formats(properties) || self.formats;

                // min order
                self.minOrder =
                    PropertyParser.minOrder(properties) || self.minOrder;

                // Frame
                self.cooFrame =
                    PropertyParser.cooFrame(properties) || self.cooFrame;

                // sky fraction
                self.skyFraction = PropertyParser.skyFraction(properties);

                // Initial fov/ra/dec
                self.initialFov = PropertyParser.initialFov(properties);
                self.initialRa =
                    properties &&
                    properties.hips_initial_ra &&
                    +properties.hips_initial_ra;
                self.initialDec =
                    properties &&
                    properties.hips_initial_dec &&
                    +properties.hips_initial_dec;

                // Cutouts
                const cutoutFromProperties = PropertyParser.cutouts(properties);
                self.minCut = cutoutFromProperties[0];
                self.maxCut = cutoutFromProperties[1];

                // Bitpix
                self.numBitsPerPixel =
                    PropertyParser.bitpix(properties) || self.numBitsPerPixel;

                // HiPS body
                if (properties.hips_body) {
                    self.hipsBody = properties.hips_body;
                    // Use the property to define and check some user given infos
                    // Longitude reversed
                    self.longitudeReversed = true;
                }

                // Give a better name if we have the HiPS metadata
                self.name = self.name || properties.obs_title;
            }

            self.name = self.name || self.id || self.url;
            self.name = self.name.replace(/  +/g, ' ');

            self.creatorDid = self.creatorDid || self.id || self.url;

            // Image format
            if (self.imgFormat) {
                // transform to lower case
                self.imgFormat = self.imgFormat.toLowerCase();
                // convert JPG -> JPEG
                if (self.imgFormat === "jpg") {
                    self.imgFormat = "jpeg";
                }

                // user wants a fits but the properties tells this format is not available
                if (
                    self.imgFormat === "fits" &&
                    self.formats &&
                    self.formats.indexOf("fits") < 0
                ) {
                    throw self.name + " does not provide fits tiles";
                }

                if (
                    self.imgFormat === "webp" &&
                    self.formats &&
                    self.formats.indexOf("webp") < 0
                ) {
                    throw self.name + " does not provide webp tiles";
                }

                if (
                    self.imgFormat === "png" &&
                    self.formats &&
                    self.formats.indexOf("png") < 0
                ) {
                    throw self.name + " does not provide png tiles";
                }

                if (
                    self.imgFormat === "jpeg" &&
                    self.formats &&
                    self.formats.indexOf("jpeg") < 0
                ) {
                    throw self.name + " does not provide jpeg tiles";
                }
            } else {
                // user wants nothing then we choose one from the properties
                if (self.formats.indexOf("webp") >= 0) {
                    self.imgFormat = "webp";
                } else if (self.formats.indexOf("png") >= 0) {
                    self.imgFormat = "png";
                } else if (self.formats.indexOf("jpeg") >= 0) {
                    self.imgFormat = "jpeg";
                } else if (self.formats.indexOf("fits") >= 0) {
                    self.imgFormat = "fits";
                } else {
                    throw (
                        "Unsupported format(s) found in the properties: " +
                        self.formats
                    );
                }
            }

            // Cutouts
            let minCut, maxCut;
            if (self.imgFormat === "fits") {
                // Take into account the default cuts given by the property file (this is true especially for FITS HiPSes)
                minCut = self.colorCfg.minCut || self.minCut || 0.0;
                maxCut = self.colorCfg.maxCut || self.maxCut || 1.0;
            } else {
                minCut = self.colorCfg.minCut || 0.0;
                maxCut = self.colorCfg.maxCut || 1.0;
            }

            self.colorCfg.setCuts(minCut, maxCut);

            // Coo frame
            if (
                self.cooFrame == "ICRS" ||
                self.cooFrame == "ICRSd" ||
                self.cooFrame == "equatorial" ||
                self.cooFrame == "j2000"
            ) {
                self.cooFrame = "ICRS";
            } else if (self.cooFrame == "galactic") {
                self.cooFrame = "GAL";
            } else {
                self.cooFrame = "ICRS";
                console.warn(
                    "Invalid cooframe given: " +
                        self.cooFrame +
                        '. Coordinate systems supported: "ICRS", "ICRSd", "j2000" or "galactic". ICRS is chosen by default'
                );
            }

            self.formats = self.formats || [self.imgFormat];

            self._saveInCache();

            return self;
        })()
    };

    ImageHiPS.prototype._saveInCache = function () {
        let self = this;

        let colorOpt = Object.fromEntries(Object.entries(this.colorCfg));
        let surveyOpt = {
            id: self.id,
            creatorDid: self.creatorDid,
            name: self.name,
            url: self.url,
            skyFraction: self.skyFraction,
            cooFrame: self.cooFrame,
            maxOrder: self.maxOrder,
            tileSize: self.tileSize,
            imgFormat: self.imgFormat,
            successCallback: self.successCallback,
            errorCallback: self.errorCallback,
            ...colorOpt,
        };

        if (self.numBitsPerPixel) {
            surveyOpt.numBitsPerPixel = self.numBitsPerPixel;
        }

        if (HiPSCache.contains(self.id)) {
            HiPSCache.append(self.id, {
                // Erase by the cache already put values which is considered
                // as the ground truth
                ...HiPSCache.get[self.id],
                // append new important infos from the properties queried
                ...surveyOpt,
            });
        }
    };

    /**
     * Checks if the ImageHiPS represents a planetary body.
     *
     * This method returns a boolean indicating whether the ImageHiPS corresponds to a planetary body, e.g. the earth or a celestial body.
     *
     * @memberof ImageHiPS
     *
     * @returns {boolean} Returns true if the ImageHiPS represents a planetary body; otherwise, returns false.
     */
    ImageHiPS.prototype.isPlanetaryBody = function () {
        return this.hipsBody !== undefined;
    };

    /**
     * Sets the image format for the ImageHiPS.
     *
     * This method updates the image format of the ImageHiPS, performs format validation, and triggers the update of metadata.
     *
     * @memberof ImageHiPS
     *
     * @param {string} format - The desired image format. Should be one of ["fits", "png", "jpg", "webp"].
     *
     * @throws {string} Throws an error if the provided format is not one of the supported formats or if the format is not available for the specific ImageHiPS.
     */
    ImageHiPS.prototype.setImageFormat = function (format) {
        let self = this;
        self.query.then(() => {
            self._updateMetadata(() => {
                let imgFormat = format.toLowerCase();

                if (
                    imgFormat !== "fits" &&
                    imgFormat !== "png" &&
                    imgFormat !== "jpg" &&
                    imgFormat !== "jpeg" &&
                    imgFormat !== "webp"
                ) {
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
                const availableFormats = self.formats;
                // user wants a fits but the metadata tells this format is not available
                if (
                    imgFormat === "fits" &&
                    availableFormats.indexOf("fits") < 0
                ) {
                    throw self.id + " does not provide fits tiles";
                }

                if (
                    imgFormat === "webp" &&
                    availableFormats.indexOf("webp") < 0
                ) {
                    throw self.id + " does not provide webp tiles";
                }

                if (
                    imgFormat === "png" &&
                    availableFormats.indexOf("png") < 0
                ) {
                    throw self.id + " does not provide png tiles";
                }

                if (
                    imgFormat === "jpeg" &&
                    availableFormats.indexOf("jpeg") < 0
                ) {
                    throw self.id + " does not provide jpeg tiles";
                }

                // Switch from png/webp/jpeg to fits
                if (
                    (self.imgFormat === "png" ||
                        self.imgFormat === "webp" ||
                        self.imgFormat === "jpeg") &&
                    imgFormat === "fits"
                ) {
                    if (self.minCut && self.maxCut) {
                        // reset cuts to those given from the properties
                        self.setCuts(self.minCut, self.maxCut);
                    }
                    // Switch from fits to png/webp/jpeg
                } else if (self.imgFormat === "fits") {
                    self.setCuts(0.0, 1.0);
                }

                // Check if it is a fits
                self.imgFormat = imgFormat;
            });
        });
    };

    /**
     * Sets the opacity factor when rendering the ImageHiPS
     *
     * @memberof ImageHiPS
     *
     * @returns {string[]} Returns the formats accepted for the survey, i.e. the formats of tiles that are availables. Could be PNG, WEBP, JPG and FITS.
     */
    ImageHiPS.prototype.getAvailableFormats = function () {
        return this.formats;
    };

    /**
     * Sets the opacity factor when rendering the ImageHiPS
     *
     * @memberof ImageHiPS
     *
     * @param {number} opacity - Opacity of the survey to set. Between 0 and 1
     */
    ImageHiPS.prototype.setOpacity = function (opacity) {
        this._updateMetadata(() => {
            this.colorCfg.setOpacity(opacity);
        });
    };

    /**
     * Sets the blending mode when rendering the ImageHiPS
     *
     * @memberof ImageHiPS
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
    ImageHiPS.prototype.setBlendingConfig = function (additive = false) {
        this._updateMetadata(() => {
            this.colorCfg.setBlendingConfig(additive);
        });
    };

    /**
     * Sets the colormap when rendering the ImageHiPS.
     *
     * @memberof ImageHiPS
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
    ImageHiPS.prototype.setColormap = function (colormap, options) {
        this._updateMetadata(() => {
            this.colorCfg.setColormap(colormap, options);
        });
    };

    /**
     * Sets the gamma correction factor for the ImageHiPS.
     *
     * This method updates the gamma of the ImageHiPS.
     *
     * @memberof ImageHiPS
     *
     * @param {number} lowCut - The low cut value to set for the ImageHiPS.
     * @param {number} highCut - The high cut value to set for the ImageHiPS.
     */
    ImageHiPS.prototype.setCuts = function (lowCut, highCut) {
        this._updateMetadata(() => {
            this.colorCfg.setCuts(lowCut, highCut);
        });
    };

    /**
     * Sets the gamma correction factor for the ImageHiPS.
     *
     * This method updates the gamma of the ImageHiPS.
     *
     * @memberof ImageHiPS
     *
     * @param {number} gamma - The saturation value to set for the ImageHiPS. Between 0.1 and 10
     */
    ImageHiPS.prototype.setGamma = function (gamma) {
        this._updateMetadata(() => {
            this.colorCfg.setGamma(gamma);
        });
    };

    /**
     * Sets the saturation for the ImageHiPS.
     *
     * This method updates the saturation of the ImageHiPS.
     *
     * @memberof ImageHiPS
     *
     * @param {number} saturation - The saturation value to set for the ImageHiPS. Between 0 and 1
     */
    ImageHiPS.prototype.setSaturation = function (saturation) {
        this._updateMetadata(() => {
            this.colorCfg.setSaturation(saturation);
        });
    };

    /**
     * Sets the brightness for the ImageHiPS.
     *
     * This method updates the brightness of the ImageHiPS.
     *
     * @memberof ImageHiPS
     *
     * @param {number} brightness - The brightness value to set for the ImageHiPS. Between 0 and 1
     */
    ImageHiPS.prototype.setBrightness = function (brightness) {
        this._updateMetadata(() => {
            this.colorCfg.setBrightness(brightness);
        });
    };

    /**
     * Sets the contrast for the ImageHiPS.
     *
     * This method updates the contrast of the ImageHiPS and triggers the update of metadata.
     *
     * @memberof ImageHiPS
     *
     * @param {number} contrast - The contrast value to set for the ImageHiPS. Between 0 and 1
     */
    ImageHiPS.prototype.setContrast = function (contrast) {
        this._updateMetadata(() => {
            this.colorCfg.setContrast(contrast);
        });
    };

    // Private method for updating the backend with the new meta
    ImageHiPS.prototype._updateMetadata = function (callback) {
        if (callback) {
            callback();
        }

        // Tell the view its meta have changed
        try {
            if (this.added) {
                this.view.wasm.setImageMetadata(this.layer, {
                    ...this.colorCfg.get(),
                    longitudeReversed: this.longitudeReversed,
                    imgFormat: this.imgFormat,
                });
                // once the meta have been well parsed, we can set the meta
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

    ImageHiPS.prototype.add = function (layer) {
        this.layer = layer;
        let self = this;

        this.view.wasm.addImageHiPS({
            layer,
            properties: {
                creatorDid: self.creatorDid,
                url: self.url,
                maxOrder: self.maxOrder,
                cooFrame: self.cooFrame,
                tileSize: self.tileSize,
                formats: self.formats,
                bitpix: self.numBitsPerPixel,
                skyFraction: self.skyFraction,
                minOrder: self.minOrder,
                hipsInitialFov: self.initialFov,
                hipsInitialRa: self.initialRa,
                hipsInitialDec: self.initialDec,
                isPlanetaryBody: self.isPlanetaryBody(),
                hipsBody: self.hipsBody,
            },
            meta: {
                ...this.colorCfg.get(),
                longitudeReversed: this.longitudeReversed,
                imgFormat: this.imgFormat,
            },
        });

        return Promise.resolve(this)
            .then((hips) => {
                if (hips.successCallback) {
                    hips.successCallback(hips)
                }

                return hips
            });
    };

    // @api
    ImageHiPS.prototype.toggle = function () {
        if (this.colorCfg.getOpacity() != 0.0) {
            this.colorCfg.setOpacity(0.0);
        } else {
            this.colorCfg.setOpacity(this.prevOpacity);
        }
    };

    // @oldapi
    ImageHiPS.prototype.setAlpha = ImageHiPS.prototype.setOpacity;

    ImageHiPS.prototype.setColorCfg = function (colorCfg) {
        this._updateMetadata(() => {
            this.colorCfg = colorCfg;
        });
    };

    // @api
    ImageHiPS.prototype.getColorCfg = function () {
        return this.colorCfg;
    };

    // @api
    ImageHiPS.prototype.getOpacity = function () {
        return this.colorCfg.getOpacity();
    };

    ImageHiPS.prototype.getAlpha = ImageHiPS.prototype.getOpacity;

    // @api
    ImageHiPS.prototype.readPixel = function (x, y) {
        return this.view.wasm.readPixel(x, y, this.layer);
    };

    ImageHiPS.DEFAULT_SURVEY_ID = "CDS/P/DSS2/color";

    return ImageHiPS;
})();
