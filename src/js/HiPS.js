// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
//


/******************************************************************************
 * Aladin Lite project
 *
 * File HiPS
 *
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 *
 *****************************************************************************/
import { ALEvent } from "./events/ALEvent.js";
import { HiPSProperties } from "./HiPSProperties.js";
import { Aladin } from "./Aladin.js"; 
import { CooFrameEnum } from "./CooFrameEnum.js";
import { Utils } from "./Utils"

let PropertyParser = {};
// Utilitary functions for parsing the properties and giving default values
/// Mandatory tileSize property
PropertyParser.tileSize = function (properties) {
    let tileSize =
        (properties &&
            properties.hips_tile_width &&
            +properties.hips_tile_width)

    // Check if the tile width size is a power of 2
    if (tileSize && ((tileSize & (tileSize - 1)) !== 0)) {
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
    let formats = properties?.hips_tile_format;

    formats = formats?.split(" ").map((fmt) => fmt.toLowerCase());

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

PropertyParser.hipsDataMinmax = function (properties) {
    let data_minmax =
        properties &&
        properties.hips_data_minmax &&
        properties.hips_data_minmax.split(" ");

    const minData = data_minmax && parseFloat(data_minmax[0]);
    const maxData = data_minmax && parseFloat(data_minmax[1]);

    return [minData, maxData];
};

PropertyParser.dataRange = function (properties) {
    let range =
        properties &&
        properties.hips_data_range &&
        properties.hips_data_range.split(" ");

    const minRange = range && parseFloat(range[0]);
    const maxRange = range && parseFloat(range[1]);

    return [minRange, maxRange];
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
 * HiPS options
 * 
 * @description Minimal user provided properties will prevent Aladin Lite from fetching the properties file describing the HiPS properties.
 * These minimal informations needed by aladin lite are:
 * <ul>
 * <li>The max HEALPix order of the survey tiles</li>
 * <li>A location url of the HiPS. If a CDS ID is given (i.e. one took from the {@link https://aladin.cds.unistra.fr/hips/list| CDS HiPS list aggregator}) e.g. CDS/P/2MASS/K) then the properties is retrieved to obtain a base url for fetching the tiles.</li>
 * <li>The image format of the HiPS tiles ('jpeg', 'png', 'fits', 'webp' are supported)</li>
 * <li>The size of one tile (typically 512x512)</li>
 * <li>The coordinate frame of the HiPS</li>
 * </ul>
 * 
 * @typedef {Object} HiPSOptions
 * @property {string} [name] - The name of the survey to be displayed in the UI
 * @property {Function} [successCallback] - A callback executed when the HiPS has been loaded
 * @property {Function} [errorCallback] - A callback executed when the HiPS could not be loaded
 * @property {string} [imgFormat] - Formats accepted 'webp', 'png', 'jpeg' or 'fits'. Will raise an error if the HiPS does not contain tiles in this format
 * @property {string} [cooFrame] - Coordinate frame of the survey tiles. If not given, the one from the parsed properties file will be retrieved. Possible values: 'ICRS', 'ICRSd', 'galactic', 'equatorial', 'j2000' 
 * @property {number} [maxOrder] - The maximum HEALPix order of the HiPS, i.e the HEALPix order of the most refined tile images of the HiPS.
 * @property {number} [bitpix] - Useful if you want to display the FITS tiles of a HiPS. It specifies the number of bits per pixel. Possible values are:
 * -64: double, -32: float, 8: unsigned byte, 16: short, 32: integer 32 bits, 64: integer 64 bits
 * @property {number} [tileSize=512] - The width of the HEALPix tile images. Mostly 512 pixels but can be 256, 128, 64, 32
 * @property {number} [minOrder] - If not given, retrieved from the properties of the survey.
 * @property {boolean} [longitudeReversed] - Deprecated The longitudeReversed property is now deprecated since version 3.6.1. This property has been removed since version 3.7.0 and replaced with {@link Aladin#reverseLongitude} set directly on the {@link Aladin} view object and not at the HiPS level.
 * @property {number} [opacity=1.0] - Opacity of the survey or image (value between 0 and 1).
 * @property {string} [colormap="native"] - The colormap configuration for the survey or image.
 * @property {string} [stretch="linear"] - The stretch configuration for the survey or image.
 * @property {boolean} [reversed=false] - If true, the colormap is reversed; otherwise, it is not reversed.
 * @property {number} [minCut] - The minimum cut value for the color configuration. If not given, 0.0 for JPEG/PNG surveys, the value of the property file for FITS surveys
 * @property {number} [maxCut] - The maximum cut value for the color configuration. If not given, 1.0 for JPEG/PNG surveys, the value of the property file for FITS surveys
 * @property {boolean} [blending=false] - If true, additive blending is applied; otherwise, it is not applied.
 * @property {number} [gamma=1.0] - The gamma correction value for the color configuration.
 * @property {number} [saturation=0.0] - The saturation value for the color configuration.
 * @property {number} [brightness=0.0] - The brightness value for the color configuration.
 * @property {number} [contrast=0.0] - The contrast value for the color configuration.
 * @property {string} [requestMode='cors'] - Determines how the request will interact with cross-origin resources.
    * <ul> 
    * <li>'cors' - allow cross-origin requests with proper CORS headers.</li>
    * <li>'no-cors' - send the request without CORS.</li>
    * <li>'same-origin' - only allow requests to the same origin.</li>
    * </ul>
 * @property {string} [requestCredentials='same-origin'] - Specifies whether to send cookies and HTTP credentials with the request.
    *  <ul>
    *  <li>'omit' - never send credentials.</li>
    *  <li>'same-origin' - send only for same-origin requests.</li>
    *  <li>'include' - always send, even for cross-origin requests.</li>
    *  </ul>
 */

/**
 * Screen pixel prober type
 * 
 * @typedef {Object} PixelProber
 * @property {number} [x] - x screen coordinate. Default is set to the view center, i.e. half the width in pixels of the aladin lite div.
 * @property {number} [y] - y screen coordinate. Default is set to the view center, i.e. half the height in pixels of the aladin lite div.
 */

/**
 * Screen line prober type
 * 
 * @typedef {Object} LineProber
 * @property {number} [x1] - x start point screen coordinate
 * @property {number} [y1] - y start point screen coordinate
 * @property {number} [x2] - x end point screen coordinate
 * @property {number} [y2] - y end point screen coordinate
 */

/**
 * Sky great circle arc prober type
 * 
 * @typedef {Object} GreatCircleArcProber
 * @property {number} [ra1] - ra first point sky coordinate (in icrs) frame
 * @property {number} [dec1] - dec first point sky coordinate (in icrs) frame
 * @property {number} [ra2] - ra end point sky coordinate (in icrs) frame
 * @property {number} [dec2] - dec end point sky coordinate (in icrs) frame
 */

/**
 * Screen rectangular prober type
 * 
 * @typedef {Object} RectProber
 * @property {number} [top] - top screen pixel coordinate
 * @property {number} [left] - left screen pixel coordinate
 * @property {number} [w] - width in screen pixel
 * @property {number} [h] - height in screen pixel
 */


/**
 * JS {@link https://developer.mozilla.org/fr/docs/Web/API/FileList| FileList} API type
 * 
 * @typedef {Object} FileList
 */

/**
 * Tiles are accessed like so: HIPSLocalFiles[norder][ipix] = {@link File};<br/>
 * The properties file is accessed with: HIPSLocalFiles["properties"]
 * @typedef {Object} HiPSLocalFiles
 * @property {File} properties - The local properties file of the HiPS
 */

export let HiPS = (function () {
    /**
     * The object describing an image survey
     *
     * @class
     * @constructs HiPS
     *
     * @param {string} id - Mandatory unique identifier for the layer. Can be an arbitrary name
     * @param {string|FileList|HiPSLocalFiles} location - Can be:
     * <ul>
     * <li>An http url towards a HiPS.</li>
     * <li>A relative path to your HiPS</li>
     * <li>A special ID pointing towards a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>A dict storing a local HiPS files. This object contains a tile file: hips[order][ipix] = File and refers to the properties file like so: hips["properties"] = File. </li>
     *     A javascript {@link FileList} pointing to the opened webkit directory is also accepted.
     * </ul>
     * @param {HiPSOptions} [options] - The option for the survey
     *
     * @description Giving a CDS ID will do a query to the MOCServer first to retrieve metadata. Then it will also check for the presence of faster HiPS nodes to choose a faster url to query to tiles from.
     */
    function HiPS(id, location, options) {
        this.added = false;
        // Required ID, synonym of the CreatorDid of a HiPS
        this.id = id;

        this.name = (options && options.name) || id;
        this.startUrl = options && options.startUrl;
        this.requestMode = options && options.requestMode || 'cors';
        this.requestCredentials = options && options.requestCredentials || 'same-origin';
        this.type = "hips"

        this.slice = 0;

        if (location instanceof FileList) {
            let localFiles = {};
            for (var file of location) {
                let path = file.webkitRelativePath;
                if (path.includes("Norder") && path.includes("Npix")) {
                    const order = +path.substring(path.indexOf("Norder") + 6).split("/")[0];
                    if (!localFiles[order]) {
                        localFiles[order] = {}
                    }

                    let tile = path.substring(path.indexOf("Npix") + 4).split(".");
                    const ipix = +tile[0];
                    const fmt = tile[1];

                    if (!localFiles[order][ipix]) {
                        localFiles[order][ipix] = {}
                    }

                    localFiles[order][ipix][fmt] = file;
                }

                if (path.includes("properties")) {
                    localFiles['properties'] = file;
                }

                if (path.includes("Moc")) {
                    localFiles['moc'] = file;
                }
            }

            this.localFiles = localFiles;
        } else if (location instanceof Object) {
            this.localFiles = location;
        }

        this.url = location;

        // Max order Required
        this.maxOrder = options && options.maxOrder;
        this.minOrder = (options && options.minOrder) || 0;
        // Frame Required
        this.cooFrame = options && CooFrameEnum.fromString(options.cooFrame, null)?.system;
        this.tileSize = options && options.tileSize || 512;
        this.skyFraction = options && options.skyFraction;
        // Image format Required
        this.imgFormat = options && options.imgFormat;
        this.formats = (options && options.formats) || (this.imgFormat && [this.imgFormat]);
        this.defaultFitsMinCut = options && options.defaultFitsMinCut;
        this.defaultFitsMaxCut = options && options.defaultFitsMaxCut;
        this.bitpix = options && options.bitpix;
        this.creatorDid = (options && options.creatorDid) || this.id || this.url;
        this.errorCallback = options && options.errorCallback;
        this.successCallback = options && options.successCallback;
        this.dataproductType = options && options.dataproductType || 'image';
        this.tileDepth = options && options.tileDepth;
        this.orderFreq = options && options.orderFreq;
        this.emMin = options && options.emMin;
        this.emMax = options && options.emMax;

        // Opacity of the survey/image
        this.opacity = (options && options.opacity) || 1.0;

        // Colormap config options
        this.colormap = (options && options.colormap) || "native";
        this.colormap = this.colormap.toLowerCase();

        this.stretch = (options && options.stretch) || "linear";
        this.stretch = this.stretch.toLowerCase();
        this.reversed = false;
        // Keep the image tile format because we want the cuts
        this.imgFormat = (options && options.imgFormat) || 'png';

        if (options && options.reversed === true) {
            this.reversed = true;
        }

        this.minCut = {
            webp: 0.0,
            jpeg: 0.0,
            png: 0.0,
            fits: undefined // wait the default value coming from the properties
        };    

        this.maxCut = {
            webp: 255.0,
            jpeg: 255.0,
            png: 255.0,
            fits: undefined // wait the default value coming from the properties
        };

        this.setCuts(options.minCut, options.maxCut);

        this.blending = options && options.blending;
        if (this.blending === undefined)  {
            this.blending = false;
        }

        // A default value for gamma correction
        this.gamma = (options && options.gamma) || 1.0;
        this.saturation = (options && options.saturation) || 0.0;
        this.brightness = (options && options.brightness) || 0.0;
        this.contrast = (options && options.contrast) || 0.0;

        let self = this;

        if (this.localFiles) {
            // Fetch the properties file
            this.query = new Promise(async (resolve, reject) => {
                // look for the properties file
                await HiPSProperties.fetchFromFile(self.localFiles["properties"])
                    .then((p) => {
                        self._parseProperties(p);
                        self.url = "local";

                        delete self.localFiles["properties"]
                    })
                    .catch((e) => reject("HiPS " + self.id + " error: " + self.localFiles["properties"] + " does not point towards a local HiPS.\nReason: " + e.stack))

                resolve(self);
            });
        } else {
            let mustRequestProperties = true;

            let isUrl = Utils.isUrl(this.url) !== undefined;
    
            if (this.imgFormat === "fits") {
                // a fits is given
                mustRequestProperties = !(
                    this.maxOrder &&
                    isUrl &&
                    this.imgFormat &&
                    this.cooFrame &&
                    // TODO: this should not be mandatory, one just parse FITS files
                    this.bitpix
                );
            } else {
                mustRequestProperties = !(
                    this.maxOrder &&
                    isUrl &&
                    this.imgFormat &&
                    this.cooFrame
                );
            }
    
            this.query = new Promise(async (resolve, reject) => {
                if (mustRequestProperties) {
                    // ID typed url
                    if (self.startUrl && !isUrl) {
                        // First download the properties from the start url
                        await HiPSProperties.fetchFromUrl(self.startUrl, self.requestMode, self.requestCredentials)
                            .then((p) => {
                                self._parseProperties(p);
                            })
                            .catch((e) => reject("HiPS " + self.id + " error: starting url " + self.startUrl + " given does not points to a HiPS location.\nReason: " + e.stack))
    
                        // the url stores a "CDS ID" we take it prioritaly
                        // if the url is null, take the id, this is for some tests
                        // to pass because some users might just give null as url param and a "CDS ID" as id param
                        let id = self.url || self.id;

                        self.url = self.startUrl;

                        setTimeout(
                            () => {
                                if (!self.added)
                                    return;

                                HiPSProperties.fetchFromID(id)
                                    .then((p) => {
                                        self._fetchFasterUrlFromProperties(p);
                                    })
                                    .catch((e) => reject("HiPS " + self.id + " error: CDS ID " + id + " is not found.\nReason: " + e.stack));
                            },
                            1000
                        );
                    } else if (!self.startUrl && !isUrl) {
                        // the url stores a "CDS ID" we take it prioritaly
                        // if the url is null, take the id, this is for some tests
                        // to pass because some users might just give null as url param and a "CDS ID" as id param
                        let id = self.url || self.id;

                        await HiPSProperties.fetchFromID(id)
                            .then((p) => {
                                self.url = p.hips_service_url;

                                self._parseProperties(p);
                                self._fetchFasterUrlFromProperties(p);
                            })
                            .catch((_) => {
                                // If no ID has been found then it may actually be a path
                                // url pointing to a local HiPS
                                return HiPSProperties.fetchFromUrl(id)
                                    .then((p) => {
                                        self._parseProperties(p);
                                    })
                                    .catch((e) => reject("HiPS " + self.id + " error: " + id + " does not refer to a found CDS ID nor a local path pointing towards a HiPS.\nReason: " + e.stack))
                            })
                    } else {
                        await HiPSProperties.fetchFromUrl(self.url, self.requestMode, self.requestCredentials)
                            .then((p) => {
                                self._parseProperties(p);
                            })
                            .catch((e) => reject("HiPS " + self.id + " error: HiPS not found at url " + self.url + "\nReason: " + e.stack))
                    }
                } else {
                    /*self._parseProperties({
                        hips_order: self.maxOrder,
                        hips_service_url: self.url,
                        hips_tile_width: self.tileSize,
                        hips_frame: self.cooFrame
                    })*/
                }
    
                if (self.updateHiPSCache) {
                    self._updateMetadata()
                }
    
                resolve(self);
            });
        }
    };

    HiPS.prototype._fetchFasterUrlFromProperties = function(properties) {
        let self = this;

        HiPSProperties.getFasterMirrorUrl(properties)
            .then((url) => {
                if (self.url !== url) {
                    console.info(
                        "Change url of ",
                        self.id,
                        " to ",
                        url
                    );

                    self.url = url;
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

    HiPS.prototype._parseProperties = function(properties) {
        let self = this;
        self.creatorDid = properties.creator_did || self.creatorDid;

        // HiPS Cube special keywords
        self.cubeDepth = properties && properties.hips_cube_depth && +properties.hips_cube_depth;
        self.cubeFirstFrame = properties && properties.hips_cube_firstframe && +properties.hips_cube_firstframe;
        self.emMin = (properties && properties.em_min && +properties.em_min) || self.emMin;
        self.emMax = (properties && properties.em_max && +properties.em_max) || self.emMax;

        if (self.emMax < self.emMin) {
            let tmp = self.emMin;
            self.emMin = self.emMax;
            self.emMax = tmp;
        }

        self.dataMinMax = PropertyParser.hipsDataMinmax(properties);

        self.dataRange = PropertyParser.dataRange(properties);

        // HiPS3D special keywords
        self.orderFreq = (properties && properties.hips_order_freq && +properties.hips_order_freq) || self.orderFreq;
        self.tileDepth = (properties && properties.hips_tile_depth && +properties.hips_tile_depth) || self.tileDepth;
        self.obsRestFreq = properties && properties.obs_restfreq && +properties.obs_restfreq;

        // Max order
        const maxOrder = PropertyParser.maxOrder(properties)
        if (maxOrder !== undefined) {
            self.maxOrder = maxOrder;
        }

        // dataproduct type
        self.dataproductType = properties && properties.dataproduct_type || self.dataproductType;

        // Tile size
        self.tileSize =
            PropertyParser.tileSize(properties) || self.tileSize;

        // Tile formats
        self.formats =
            PropertyParser.formats(properties) || self.formats;

        // Min order
        const minOrder = PropertyParser.minOrder(properties)
        if (minOrder !== undefined) {
            self.minOrder = minOrder;
        }

        // Frame
        let cooFrame =
            PropertyParser.cooFrame(properties);
        // Parse the cooframe from the properties but if it fails, take the one given by the user
        // If the user gave nothing, then take ICRS as the default one
        self.cooFrame = CooFrameEnum.fromString(cooFrame, null)?.system || self.cooFrame || CooFrameEnum.ICRS.system;

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
        self.defaultFitsMinCut = cutoutFromProperties[0] || 0.0;
        self.defaultFitsMaxCut = cutoutFromProperties[1] || 1.0;

        // Bitpix
        self.bitpix =
            PropertyParser.bitpix(properties) || self.bitpix;

        // HiPS body
        if (properties.hips_body) {
            self.hipsBody = properties.hips_body;
            // The HiPS is a planetary one, so we reverse the longitude axis globally
            self.view.aladin.reverseLongitude(true)
        }

        // Give a better name if we have the HiPS metadata
        self.name = self.name || properties.obs_title;

        self.name = self.name || self.id || self.url;
        self.name = self.name.replace(/  +/g, ' ');

        // check the imgFormat with respect to the formats accepted image format
        const chooseTileFormat = (formats) => {
            if (formats.indexOf("webp") >= 0) {
                return "webp";
            } else if (formats.indexOf("png") >= 0) {
                return "png";
            } else if (formats.indexOf("jpeg") >= 0) {
                return "jpeg";
            } else if (formats.indexOf("fits") >= 0) {
                return "fits";
            } else if (formats.indexOf("fits.fz") >= 0) {
                return "fits";
            } else {
                throw (
                    "Unsupported format(s) found in the properties: " +
                    formats
                );
            }
        };

        // Set an image format with respect to the ones available for that HiPS if:
        // * the format is unknown
        // * the format is known but is not available for that HiPS
        if (!self.imgFormat || !self.formats.includes(self.imgFormat)) {
            // Switch automatically to a available format
            let imgFormat = chooseTileFormat(self.formats);
            self.setImageFormat(imgFormat)

            console.info(self.id + " tile format chosen: " + self.imgFormat)
        }

        // Set a cuts for fits formats if no cuts has been yet given
        let [minCut, maxCut] = self.getCuts();
        if (self.imgFormat === "fits" && minCut === undefined && maxCut === undefined) {
            self.setCuts(self.defaultFitsMinCut, self.defaultFitsMaxCut);
        }
    }

    /**
     * Checks if the HiPS represents a planetary body.
     *
     * This method returns a boolean indicating whether the HiPS corresponds to a planetary body, e.g. the earth or a celestial body.
     *
     * @memberof HiPS
     *
     * @returns {boolean} Returns true if the HiPS represents a planetary body; otherwise, returns false.
     */
    HiPS.prototype.isPlanetaryBody = function () {
        return this.hipsBody !== undefined;
    };

    /**
     * Sets the image format for the HiPS.
     *
     * This method updates the image format of the HiPS, performs format validation, and triggers the update of metadata.
     *
     * @memberof HiPS
     *
     * @param {string} imgFormat - The desired image format. Should be one of ["fits", "png", "jpg", "webp"].
     *
     * @throws {string} Throws an error if the provided format is not one of the supported formats or if the format is not available for the specific HiPS.
     */
    HiPS.prototype.setImageFormat = function (imgFormat) {
        this.setOptions({imgFormat});
    };

    /**
     * Get the list of accepted tile format for that HiPS
     *
     * @memberof HiPS
     *
     * @returns {string[]} Returns the formats accepted for the survey, i.e. the formats of tiles that are availables. Could be PNG, WEBP, JPG and FITS.
     */
    HiPS.prototype.getAvailableFormats = function () {
        return this.formats;
    };

    /**
     * Sets the blending mode when rendering the HiPS
     *
     * @memberof HiPS
     *
     * @param {boolean} [blending=false] - When rendering this survey on top of the already rendered ones, the final color of the screen is computed like:
     * <br />
     * <br />opacity * this_survey_color + (1 - opacity) * already_rendered_color for the default mode
     * <br />opacity * this_survey_color + already_rendered_color for the additive mode
     * <br />
     * <br />
     * Additive mode allows you to do linear survey color combination i.e. let's define 3 surveys named s1, s2, s3. Each could be associated to one color channel, i.e. s1 with red, s2 with green and s3 with the blue color channel.
     * If the additive blending mode is enabled, then the final pixel color of your screen will be: rgb = [s1_opacity * s1_color; s2_opacity * s2_color; s3_opacity * s3_color]
     */
    HiPS.prototype.setBlendingConfig = function (blending = false) {
        this.setOptions({blending});
    };

    HiPS.prototype.isSpectralCube = function() {
        return this.tileDepth !== undefined && this.tileDepth !== null;
    }

    /**
     * Sets the colormap when rendering the HiPS.
     *
     * @memberof HiPS
     *
     * @param {string} [colormap] - The colormap label to use. See {@link https://matplotlib.org/stable/users/explain/colors/colormaps.html|here} for more info about colormaps. 
     * If null or undefined, the colormap type is not changed.
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
    HiPS.prototype.setColormap = function (colormap, options) {
        this.setOptions({colormap, ...options});
    };

    /**
     * Sets the gamma correction factor for the HiPS.
     *
     * This method updates the gamma of the HiPS.
     *
     * @memberof HiPS
     *
     * @param {number} minCut - The low cut value to set for the HiPS.
     * @param {number} maxCut - The high cut value to set for the HiPS.
     * @param {string} [imgFormat] - The image format for which one wants to set the cuts. By default, the format used is the current imageFormat
     */
    HiPS.prototype.setCuts = function (minCut, maxCut, cutFormat) {
        this.setOptions({minCut, maxCut, cutFormat})
    };

     /**
     * Sets the brightness for the HiPS.
     *
     * This method updates the brightness of the HiPS.
     *
     * @memberof HiPS
     *
     * @param {number} brightness - The brightness value to set for the HiPS. Between 0 and 1
     */
    HiPS.prototype.setBrightness = function(brightness) {
        this.setOptions({brightness})
    };

    // @api
    HiPS.prototype.getBrightness = function() {
        return this.brightness;
    };

    /**
     * Sets the contrast for the HiPS.
     *
     * This method updates the contrast of the HiPS and triggers the update of metadata.
     *
     * @memberof HiPS
     *
     * @param {number} contrast - The contrast value to set for the HiPS. Between 0 and 1
     */
    HiPS.prototype.setContrast = function(contrast) {
        this.setOptions({contrast})
    };

    // @api
    HiPS.prototype.getContrast = function() {
        return this.kContrast;
    };

    /**
     * Sets the saturation for the HiPS.
     *
     * This method updates the saturation of the HiPS.
     *
     * @memberof HiPS
     *
     * @param {number} saturation - The saturation value to set for the HiPS. Between 0 and 1
     */
    HiPS.prototype.setSaturation = function(saturation) {
        this.setOptions({saturation})
    };

    // @api
    HiPS.prototype.getSaturation = function() {
        return this.kSaturation;
    };

    /**
     * Sets the gamma correction factor for the HiPS.
     *
     * This method updates the gamma of the HiPS.
     *
     * @memberof HiPS
     *
     * @param {number} gamma - The saturation value to set for the HiPS. Between 0.1 and 10
     */
    HiPS.prototype.setGamma = function(gamma) {
        this.setOptions({gamma})
    };

    // @api
    HiPS.prototype.getGamma = function() {
        return this.kGamma;
    };

    /**
     * Sets the opacity factor when rendering the HiPS
     *
     * @memberof HiPS
     *
     * @param {number} opacity - Opacity of the survey to set. Between 0 and 1
     */
    HiPS.prototype.setOpacity = function(opacity) {
        this.setOptions({opacity})
    };

    /**
     * Get the opacity of the HiPS layer
     * 
     * @memberof HiPS
     * 
     * @returns {number} The opacity of the layer
     */
    HiPS.prototype.getOpacity = function() {
        return this.opacity;
    };

    // @api
    HiPS.prototype.getAlpha = HiPS.prototype.getOpacity;

    HiPS.prototype.getBlendingConfig = function() {
        return this.blending;
    };

    // @api
    HiPS.prototype.getColormap = function() {
        return this.colormap;
    };

    HiPS.prototype.getReversed = function() {
        return this.reversed;
    };

    /**
     * Returns the low and high cuts under the form of a 2 element array
     *
     * @memberof HiPS
     *
     * @returns {number[]} The low and high cut values for the HiPS.
     */
    HiPS.prototype.getCuts = function() {
        return [
            this.minCut[this.imgFormat],
            this.maxCut[this.imgFormat]
        ];
    };

    HiPS.prototype.setSliceNumber = function(slice) {
        this.slice = slice;

        if (this.added) {
            let meters = this.emMin + ((slice / this.cubeDepth) * (this.emMax - this.emMin));

            let freq = 299792458.0 / meters;
            this.view.wasm.setFreq(this.layer, freq);
        }
    }

    /**
     * Set the frequency to look at (for HiPS3D object only).
     *
     * @memberof HiPS
     *
     * @param {Object} [options] - frequency object
     * @param {number} [options.value] = The frequency value expressed in `options.unit`
     * @param {"Hz"|"m"|"m/s"} [options.unit="Hz"] - The unit of the frequency passed
     * @param {number} [options.restFreq] - "The rest frequency (in Hz) to use for computing the velocity in m.s-1"
     *
     * @example
     * hips3d.setFrequency({ value: 1420302592, unit: 'Hz' })
     */
    HiPS.prototype.setFrequency = function(options) {
        if (this.added) {
            const SPEED_OF_LIGHT = 299792458.0;

            const value = options && options.value;
            const unit = options && options.unit;

            let freq;
            if (unit === "m") {
                freq = SPEED_OF_LIGHT / value;
            } else if (unit === "m/s") {
                // A velocity is given in "m/s"
                const restFreq = options && options.restFreq;
                if (!restFreq) {
                    throw 'When giving a velocity, a rest frequency must be given as well for computing the frequency to query the HiPS'
                }

                freq = restFreq * (1.0 - value / SPEED_OF_LIGHT)
            } else {
                // unit is "Hz"
                freq = value;
            }

            this.view.wasm.setFreq(this.layer, freq);
        }
    }

    /**
     * Get the frequency in Hz
     *
     * @memberof HiPS
     */
    HiPS.prototype.getFrequency = function() {
        if (this.added) {
            return this.view.wasm.getFreq(this.layer);
        }
    }

    /**
     * Get the frequency window around the current observed frequency in Hz
     *
     * @memberof HiPS
     */
    HiPS.prototype.getFrequencyWindow = function() {
        if (this.added) {
            return this.view.wasm.getFreqWindow(this.layer);
        }
    }

    // Private method for updating the backend with the new meta
    HiPS.prototype._updateMetadata = function () {
        try {
            if (this.added) {
                this.view.wasm.setImageMetadata(this.layer, this._prepareMetadataForWASM());
                // once the meta have been well parsed, we can set the meta
                ALEvent.LAYER_CHANGED.dispatchedTo(this.view.aladinDiv, {
                    layer: this,
                });
            }

            // Save it in the JS HiPS cache
            this._saveInCache();
        } catch (e) {
            // Display the error message
            console.error(e);
        }
    };

    /**
    * Set color options generic method for changing colormap, opacity, ... of the HiPS
    *
    * @memberof HiPS
    *  
    * @param {Object} options
    * @param {number} [options.imgFormat] - Image format of the HiPS tiles. Possible values are "jpeg", "png", "webp" or "fits".
    * Some formats might not be handled depending on the survey simply because tiles of that format have not been generated.
    * @param {number} [options.opacity=1.0] - Opacity of the survey or image (value between 0 and 1).
    * @param {string} [options.colormap="native"] - The colormap configuration for the survey or image.
    * @param {string} [options.stretch="linear"] - The stretch configuration for the survey or image.
    * @param {boolean} [options.reversed=false] - If true, the colormap is reversed; otherwise, it is not reversed.
    * @param {number} [options.minCut] - The minimum cut value for the color configuration. If not given, 0.0 for JPEG/PNG surveys, the value of the property file for FITS surveys
    * @param {number} [options.maxCut] - The maximum cut value for the color configuration. If not given, 1.0 for JPEG/PNG surveys, the value of the property file for FITS surveys
    * @param {boolean} [options.blending=false] - If true, additive blending is applied; otherwise, it is not applied.
    * @param {number} [options.gamma=1.0] - The gamma correction value for the color configuration.
    * @param {number} [options.saturation=0.0] - The saturation value for the color configuration.
    * @param {number} [options.brightness=0.0] - The brightness value for the color configuration.
    * @param {number} [options.contrast=0.0] - The contrast value for the color configuration.
     */
    HiPS.prototype.setOptions = function(options) {
        /// imgFormat
        if (options && options.imgFormat) {
            this.imgFormat = options.imgFormat
        }

        /// colormap
        if (options && options.colormap) {
            this.colormap = options.colormap.toLowerCase()
        }

        /// stretch
        if (options && options.stretch) {
            let stretch = options.stretch;
            this.stretch = stretch.toLowerCase()
        }
        
        /// reversed
        if (options && options.reversed !== undefined) {
            this.reversed = options.reversed;
        }

        /// cuts
        let cutFormat = options.cutFormat?.toLowerCase() || this.imgFormat;

        if (cutFormat === "jpg") {
            cutFormat = "jpeg";
        }

        
        let minCut = options && options.minCut;
        if (minCut instanceof Object) {
            // Mincut is given in the form of an javascript object with all the formats
            this.minCut = {...this.minCut, ...minCut};
        } else if (minCut !== null && minCut !== undefined) {
            this.minCut[cutFormat] = minCut;
        }

        let maxCut = options && options.maxCut;
        if (maxCut instanceof Object) {
            this.maxCut = {...this.maxCut, ...maxCut};
        } else if (maxCut !== null && maxCut !== undefined) {
            this.maxCut[cutFormat] = maxCut;
        }
        
        if (options && Utils.isNumber(options.brightness)) {
            let brightness = options.brightness;

            brightness = +brightness || 0.0; // coerce to number
            this.brightness = Math.max(-1, Math.min(brightness, 1));
        }
        
        if (options && Utils.isNumber(options.saturation)) {
            let saturation = options.saturation;
            saturation = +saturation || 0.0; // coerce to number

            this.saturation = Math.max(-1, Math.min(saturation, 1));
        }

        if (options && Utils.isNumber(options.contrast)) {
            let contrast = options.contrast;

            contrast = +contrast || 0.0; // coerce to number
            this.contrast = Math.max(-1, Math.min(contrast, 1));
        }

        if (options && Utils.isNumber(options.gamma)) {
            let gamma = options.gamma;
            gamma = +gamma; // coerce to number
            this.gamma = Math.max(0.1, Math.min(gamma, 10));
        }

        if (options && Utils.isNumber(options.opacity)) {
            let opacity = options.opacity;
            opacity = +opacity; // coerce to number
            this.opacity = Math.max(0, Math.min(opacity, 1));
        }
    
        if (options && options.blending) {
            this.blending = options.blending;
        }

        /// Set image format
        if (options.imgFormat) {
            if (this.dataproductType === "spectral-cube" && this.view.spectraDisplayer && this.view.spectraDisplayer.hips === this) {
                this.view.spectraDisplayer.resetScale()
            }

            let imgFormat = options.imgFormat.toLowerCase();

            if (imgFormat === "jpg") {
                imgFormat = "jpeg";
            }

            if (!["fits", "png", "jpeg", "webp", "fits.fz"].includes(imgFormat)) {
                console.warn('Formats must lie in ["fits", "png", "jpg", "webp"]. imgFormat option property ignored');
            } else {
                // Passed the check, we erase the image format with the new one
                // We do nothing if the imgFormat is the same

                // Check the properties to see if the given format is available among the list
                // If the properties have not been retrieved yet, it will be tested afterwards
                const availableFormats = this.formats;
                // user wants a fits but the metadata tells this format is not available
                if (!availableFormats || (availableFormats && availableFormats.indexOf(imgFormat) >= 0)) {
                    this.imgFormat = imgFormat;

                    let [minCut, maxCut] = this.getCuts();
                    if (minCut === undefined && maxCut === undefined && (imgFormat === "fits" || imgFormat === "fits.fz")) {
                        // sets the default cuts parsed from the properties
                        this.setCuts(this.defaultFitsMinCut, this.defaultFitsMaxCut)
                    }
                } else {
                    console.warn(this.id + " does not provide " + imgFormat + " tiles")
                }
            }
        }

        this._updateMetadata();
    };

    /**
     * Toggle the HiPS turning its opacity to 0 back and forth
    *
    * @memberof HiPS
    */
    HiPS.prototype.toggle = function () {
        const opacity = this.getOpacity()
        if (opacity != 0.0) {
            this.prevOpacity = opacity;
            this.setOpacity(0.0);
        } else {
            this.setOpacity(this.prevOpacity);
        }
    };

    /**
     * Old method for setting the opacity use {@link HiPS#setOpacity} instead
     * 
     * @memberof HiPS
     * @deprecated
     */
    HiPS.prototype.setAlpha = HiPS.prototype.setOpacity;

    /**
     * Probe the HiPS at a screen pixel location.
     * 
     * @description
     * Returns the true pixel value for the pixel located at the given (x, y) pixel screen position.
     * This method returns the true value coming from the tiles (color or 1 channel fits). It does not take into
     * account the apply of a transfer function, a colormap, cuts etc... It only returns the true pixel value coming from the tile
     * 
     * If you want to retrieve the pixels after apply of a transfer function, colormap, etc... i.e. if you are not looking for
     * the real HiPS pixel values, then you might be more interested in {@link Aladin#readPixel} instead.
     * 
     * @memberof HiPS
     * @param {number} [x] - x screen pixel coordinate. Default is set to the view center, i.e. half the width in pixels of the aladin lite div.
     * @param {number} [y] - y screen pixel coordinate. Default is set to the view center, i.e. half the height in pixels of the aladin lite div.
     * @returns {number} - The pixel value coming directly from the tiles
     */
    HiPS.prototype.readPixel = function (x, y) {
        x = x || (this.view.width / 2);
        y = y || (this.view.height / 2);
        return this.view.wasm.probePixel(x, y, this.layer);
    };

    /**
     * Probe the HiPS true pixels
     * 
     * @description
     * Returns the true pixels composing this HiPS.
     * This method returns the true value coming from the tiles (whether it refers to colored or 1 channel fits ones). It does not take into
     * account the apply of a transfer function, a colormap, cuts etc... i.e. it returns the true pixel values coming from the tiles.
     * 
     * This method is called by {@link HiPS#readPixel} with a pixel prober on the view center.
     * 
     * If you want to retrieve the pixels you directly see on the screen, then you might be more interested in {@link Aladin#readCanvas} instead.
     * 
     * @memberof HiPS
     * @param {PixelProber|LineProber|GreatCircleArcProber} prober - A prob object. Only, `pixel`, `line` or `arc` are accepted.
     * @returns {number[]} The pixel value(s) probed.
     */
    HiPS.prototype.probePixels = function (prober) {
        if (Utils.isNumber(prober.x) && Utils.isNumber(prober.y)) {
            // pixel probing
            return this.readPixel(prober.x, prober.y);
        } else if (Utils.isNumber(prober.x1) && Utils.isNumber(prober.y1) && Utils.isNumber(prober.x2) && Utils.isNumber(prober.y2)) {
            // line probing
            return this.view.wasm.probeLineOfPixels(prober.x1, prober.y1, prober.x2, prober.y2, this.layer);
        } else if (Utils.isNumber(prober.ra1) && Utils.isNumber(prober.dec1) && Utils.isNumber(prober.ra2) && Utils.isNumber(prober.dec2)) {
            // get the vertices along the great circle arc
            let pixelsAlongArc = view.wasm.projectGreatCircleArc(prober.ra1, prober.dec1, prober.ra2, prober.dec2);

            let pixels = []
            for (var i = 0; i < pixelsAlongArc.length; i+=4) {
                pixels = pixels.concat(this.probe({
                    x1: pixelsAlongArc[i],
                    y1: pixelsAlongArc[i+1],
                    x2: pixelsAlongArc[i+2],
                    y2: pixelsAlongArc[i+3],
                }))
            }

            return pixels
        }
    };

    HiPS.prototype._setView = function (view) {
        this.view = view;
    };

    /* Precondition: view is attached */
    HiPS.prototype._saveInCache = function () {
        if (!this.view) {
            this.updateHiPSCache = true;
            return;
        }

        this.updateHiPSCache = false;

        let self = this;
        let hipsCache = this.view.aladin.hipsCache;

        if (hipsCache.contains(self.id)) {
            hipsCache.update(self.id, {
                creatorDid: self.creatorDid,
                url: self.url,
                maxOrder: self.maxOrder,
                cooFrame: self.cooFrame,
                tileSize: self.tileSize,
                formats: self.formats,
                bitpix: self.bitpix,
                skyFraction: self.skyFraction,
                minOrder: self.minOrder,
                initialFov: self.initialFov,
                initialRa: self.initialRa,
                initialDec: self.initialDec,
                emMin: self.emMin,
                emMax: self.emMax,
                // HiPS Cube
                cubeDepth: self.cubeDepth,
                // HiPS3D
                tileDepth: self.tileDepth,
                orderFreq: self.orderFreq,
                // Dataproduct type
                dataproductType: self.dataproductType, 
                isPlanetaryBody: self.isPlanetaryBody(),
                hipsBody: self.hipsBody,
                requestCredentials: self.requestCredentials,
                requestMode: self.requestMode,
                name: this.name,
                id: this.id,
                type: this.type,
                ...this._getMetadata(),
            })
        }
    };

    HiPS.prototype._removeFromView = function() {
        if (!this.view)
            return;

        if (this.added) {
            this.view.wasm.removeLayer(this.layer);
        }
    };

    HiPS.prototype._getMetadata = function() {
        return {
            imgFormat: this.imgFormat,
            blending: this.blending,
            opacity: this.opacity,
            // Tonal corrections constants
            gamma: this.gamma,
            saturation: this.saturation,
            brightness: this.brightness,
            contrast: this.contrast,

            stretch: this.stretch,
            minCut: this.minCut,
            maxCut: this.maxCut,
            reversed: this.reversed,
            colormap: this.colormap,
        };
    };

    HiPS.prototype._addToView = function (layer) {
        if (!this.view)
            return this;

        this.layer = layer;
        let self = this;

        const config = {
            layer,
            properties: {
                creatorDid: self.creatorDid,
                url: self.url,
                maxOrder: self.maxOrder,
                cooFrame: self.cooFrame,
                tileSize: self.tileSize,
                formats: self.formats,
                bitpix: self.bitpix,
                skyFraction: self.skyFraction,
                minOrder: self.minOrder,
                initialFov: self.initialFov,
                initialRa: self.initialRa,
                initialDec: self.initialDec,
                emMin: self.emMin,
                emMax: self.emMax,
                // HiPS Cube
                cubeDepth: self.cubeDepth,
                // HiPS3D
                tileDepth: self.tileDepth,
                orderFreq: self.orderFreq,
                // Dataproduct type
                dataproductType: self.dataproductType, 
                isPlanetaryBody: self.isPlanetaryBody(),
                hipsBody: self.hipsBody,
                requestCredentials: self.requestCredentials,
                requestMode: self.requestMode,
            },
            meta: this._prepareMetadataForWASM()
        };

        let localFiles;
        if (this.localFiles) {
            localFiles = new Aladin.wasmLibs.core.HiPSLocalFiles(this.localFiles["moc"]);

            let fmt;
            for (var order in this.localFiles) {
                if (order === "moc")
                    continue;

                for (var ipix in this.localFiles[order]) {
                    for (var f in this.localFiles[order][ipix]) {
                        if (f === "png") {
                            fmt = Aladin.wasmLibs.core.ImageExt.Png;
                        } else if (f === "fits") {
                            fmt = Aladin.wasmLibs.core.ImageExt.Fits;
                        } else {
                            fmt = Aladin.wasmLibs.core.ImageExt.Jpeg;
                        }

                        const tileFile = this.localFiles[order][+ipix][f];
                        localFiles.insert(+order, BigInt(+ipix), fmt, tileFile)
                    }
                }
            }
        }

        this.view.wasm.addHiPS(
            config,
            localFiles
        );

        this.added = true;

        if (this.successCallback) {
            this.successCallback(this)
        }

        return this
    };

    HiPS.prototype._prepareMetadataForWASM = function() {
        let metadata = this._getMetadata();
        let blending = {
            srcColorFactor: 'SrcAlpha',
            dstColorFactor: 'OneMinusSrcAlpha',
            func: 'FuncAdd' 
        };

        if (this.blending) {
            blending = {
                srcColorFactor: 'SrcAlpha',
                dstColorFactor: 'One',
                func: 'FuncAdd' 
            }
        }

        let minCut = this.minCut[this.imgFormat]
        if (this.imgFormat !== "fits") {
            minCut /= 255.0
        }

        let maxCut = this.maxCut[this.imgFormat]
        if (this.imgFormat !== "fits") {
            maxCut /= 255.0
        }

        metadata["minCut"] = minCut;
        metadata["maxCut"] = maxCut;
        metadata["blending"] = blending;

        return metadata;
    };

    HiPS.DEFAULT_SURVEY_ID = "P/DSS2/color";

    return HiPS;
})();

