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
 * File HiPS
 *
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 *
 *****************************************************************************/
import { ALEvent } from "./events/ALEvent.js";
import { ColorCfg } from "./ColorCfg.js";
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
        "icrs";
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

PropertyParser.acceptedFormats = function (properties) {
    let acceptedFormats = (properties && properties.hips_tile_format) || "jpeg";

    acceptedFormats = acceptedFormats.split(" ").map((fmt) => fmt.toLowerCase());

    return acceptedFormats;
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
 * @property {CooFrame} [cooFrame] - Coordinate frame of the survey tiles. If not given, the one from the parsed properties file will be retrieved.
 * @property {number} [maxOrder] - The maximum HEALPix order of the HiPS, i.e the HEALPix order of the most refined tile images of the HiPS.
 * @property {number} [numBitsPerPixel] - Useful if you want to display the FITS tiles of a HiPS. It specifies the number of bits per pixel. Possible values are:
 * -64: double, -32: float, 8: unsigned byte, 16: short, 32: integer 32 bits, 64: integer 64 bits
 * @property {number} [tileSize] - The width of the HEALPix tile images. Mostly 512 pixels but can be 256, 128, 64, 32
 * @property {number} [minOrder] - If not given, retrieved from the properties of the survey.
 * @property {boolean} [longitudeReversed] - Deprecated The longitudeReversed property is now deprecated since version 3.6.1. This property has been removed since version 3.7.0 and replaced with {@link Aladin#reverseLongitude} set directly on the {@link Aladin} view object and not at the HiPS level.
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
     * @param {string|FileList|HiPSLocalFiles} url - Can be:
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
        // Unique identifier for a survey
        this.id = id;

        this.options = options;
        this.name = (options && options.name) || id;
        this.startUrl = options.startUrl;

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

        this.maxOrder = options.maxOrder;
        this.minOrder = options.minOrder || 0;
        this.cooFrame = CooFrameEnum.fromString(options.cooFrame, null);
        this.tileSize = options.tileSize;
        this.skyFraction = options.skyFraction;
        this.imgFormat = options.imgFormat;
        this.acceptedFormats = options.formats;
        this.defaultFitsMinCut = options.defaultFitsMinCut;
        this.defaultFitsMaxCut = options.defaultFitsMaxCut;
        this.numBitsPerPixel = options.numBitsPerPixel;
        this.creatorDid = options.creatorDid;
        this.errorCallback = options.errorCallback;
        this.successCallback = options.successCallback;

        this.colorCfg = new ColorCfg(options);

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
                    .catch((_) => reject("HiPS " + self.id + " error: " + self.localFiles["properties"] + " does not point towards a local HiPS."))

                resolve(self);
            });
        } else {
            let isIncompleteOptions = true;

            let isID = Utils.isUrl(this.url) === undefined;
    
            if (this.imgFormat === "fits") {
                // a fits is given
                isIncompleteOptions = !(
                    this.maxOrder &&
                    (!isID && this.url) &&
                    this.imgFormat &&
                    this.tileSize &&
                    this.cooFrame &&
                    this.numBitsPerPixel
                );
            } else {
                isIncompleteOptions = !(
                    this.maxOrder &&
                    (!isID && this.url) &&
                    this.imgFormat &&
                    this.tileSize &&
                    this.cooFrame
                );
            }
    
            this.query = new Promise(async (resolve, reject) => {
                if (isIncompleteOptions) {
                    // ID typed url
                    if (self.startUrl && isID) {
                        // First download the properties from the start url
                        await HiPSProperties.fetchFromUrl(self.startUrl)
                            .then((p) => {
                                self._parseProperties(p);
                            })
                            .catch((_) => reject("HiPS " + self.id + " error: starting url " + self.startUrl + " given does not points to a HiPS location"))
    
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
                                    .catch((_) => reject("HiPS " + self.id + " error: CDS ID " + id + " is not found"));
                            },
                            1000
                        );
                    } else if (!self.startUrl && isID) {
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
                            .catch(() => {
                                // If no ID has been found then it may actually be a path
                                // url pointing to a local HiPS
                                return HiPSProperties.fetchFromUrl(id)
                                    .then((p) => {
                                        self._parseProperties(p);
                                    })
                                    .catch((_) => reject("HiPS " + self.id + " error: " + id + " does not refer to a found CDS ID nor a local path pointing towards a HiPS"))
                            })
                    } else {
                        await HiPSProperties.fetchFromUrl(self.url)
                            .then((p) => {
                                self._parseProperties(p);
                            })
                            .catch((_) => reject("HiPS " + self.id + " error: HiPS not found at url " + self.url))
                    }
                } else {
                    self._parseProperties({
                        hips_order: self.maxOrder,
                        hips_service_url: self.url,
                        hips_tile_width: self.tileSize,
                        hips_frame: self.cooFrame.label
                    })
                }
    
                if (self.updateHiPSCache) {
                    self._saveInCache();
                    self.updateHiPSCache = false;
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

        // Cube depth
        self.cubeDepth = properties && properties.hips_cube_depth && +properties.hips_cube_depth;
        self.cubeFirstFrame = properties && properties.hips_cube_firstframe && +properties.hips_cube_firstframe;

        // Max order
        const maxOrder = PropertyParser.maxOrder(properties)
        if (maxOrder !== undefined) {
            self.maxOrder = maxOrder;
        }

        // Tile size
        self.tileSize =
            PropertyParser.tileSize(properties) || self.tileSize;

        // Tile formats
        self.acceptedFormats =
            PropertyParser.acceptedFormats(properties) || self.acceptedFormats;

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
        self.cooFrame = CooFrameEnum.fromString(cooFrame, self.cooFrame || CooFrameEnum.ICRS);

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
        self.numBitsPerPixel =
            PropertyParser.bitpix(properties) || self.numBitsPerPixel;

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

        self.creatorDid = self.creatorDid || self.id || self.url;

        // check the imgFormat with respect to the formats accepted image format
        const chooseTileFormat = (acceptedFormats) => {
            if (acceptedFormats.indexOf("webp") >= 0) {
                return "webp";
            } else if (acceptedFormats.indexOf("png") >= 0) {
                return "png";
            } else if (acceptedFormats.indexOf("jpeg") >= 0) {
                return "jpeg";
            } else if (acceptedFormats.indexOf("fits") >= 0) {
                return "fits";
            } else {
                throw (
                    "Unsupported format(s) found in the properties: " +
                    acceptedFormats
                );
            }
        };

        // Set an image format with respect to the ones available for that HiPS if:
        // * the format is unknown
        // * the format is known but is not available for that HiPS
        if (!self.imgFormat || !self.acceptedFormats.includes(self.imgFormat)) {
            // Switch automatically to a available format
            let imgFormat = chooseTileFormat(self.acceptedFormats);
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
        return this.acceptedFormats;
    };

    /**
     * Sets the opacity factor when rendering the HiPS
     *
     * @memberof HiPS
     *
     * @param {number} opacity - Opacity of the survey to set. Between 0 and 1
     */
    HiPS.prototype.setOpacity = function (opacity) {
        this.setOptions({opacity})
    };

    /**
     * Sets the blending mode when rendering the HiPS
     *
     * @memberof HiPS
     *
     * @param {boolean} [additive=false] - When rendering this survey on top of the already rendered ones, the final color of the screen is computed like:
     * <br />
     * <br />opacity * this_survey_color + (1 - opacity) * already_rendered_color for the default mode
     * <br />opacity * this_survey_color + already_rendered_color for the additive mode
     * <br />
     * <br />
     * Additive mode allows you to do linear survey color combination i.e. let's define 3 surveys named s1, s2, s3. Each could be associated to one color channel, i.e. s1 with red, s2 with green and s3 with the blue color channel.
     * If the additive blending mode is enabled, then the final pixel color of your screen will be: rgb = [s1_opacity * s1_color; s2_opacity * s2_color; s3_opacity * s3_color]
     */
    HiPS.prototype.setBlendingConfig = function (additive = false) {
        this.setOptions({additive});
    };

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
        colormap = colormap || this.options.colormap;

        this.setOptions({colormap, ...options})
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
     */
    HiPS.prototype.setCuts = function (minCut, maxCut) {
        this.setOptions({minCut, maxCut})
    };

    /**
     * Returns the low and high cuts under the form of a 2 element array
     *
     * @memberof HiPS
     *
     * @returns {number[]} The low and high cut values for the HiPS.
     */
    HiPS.prototype.getCuts = function () {
        return this.colorCfg.getCuts();
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
    HiPS.prototype.setGamma = function (gamma) {
        this.setOptions({gamma})
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
    HiPS.prototype.setSaturation = function (saturation) {
        this.setOptions({saturation})
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
    HiPS.prototype.setBrightness = function (brightness) {
        this.setOptions({brightness})
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
    HiPS.prototype.setContrast = function (contrast) {
        this.setOptions({contrast})
    };

    HiPS.prototype.setSliceNumber = function(slice) {
        this.slice = slice;

        if (this.added) {
            this.view.wasm.setSliceNumber(this.layer, slice);
        }
    }

    // Private method for updating the backend with the new meta
    HiPS.prototype._updateMetadata = function () {
        try {
            if (this.added) {
                this.view.wasm.setImageMetadata(this.layer, {
                    ...this.colorCfg.get(),
                    imgFormat: this.imgFormat,
                });
                // once the meta have been well parsed, we can set the meta
                ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.view.aladinDiv, {
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
    * @param {boolean} [options.additive=false] - If true, additive blending is applied; otherwise, it is not applied.
    * @param {number} [options.gamma=1.0] - The gamma correction value for the color configuration.
    * @param {number} [options.saturation=0.0] - The saturation value for the color configuration.
    * @param {number} [options.brightness=0.0] - The brightness value for the color configuration.
    * @param {number} [options.contrast=0.0] - The contrast value for the color configuration.
     */
    HiPS.prototype.setOptions = function(options) {
        this.colorCfg.setOptions(options);

        /// Set image format
        if (options.imgFormat) {
            let imgFormat = options.imgFormat.toLowerCase();

            if (imgFormat === "jpg") {
                imgFormat = "jpeg";
            }

            if (!["fits", "png", "jpeg", "webp"].includes(imgFormat)) {
                console.warn('Formats must lie in ["fits", "png", "jpg", "webp"]. imgFormat option property ignored');
            } else {
                // Passed the check, we erase the image format with the new one
                // We do nothing if the imgFormat is the same
                
                // Check the properties to see if the given format is available among the list
                // If the properties have not been retrieved yet, it will be tested afterwards
                const availableFormats = this.acceptedFormats;
                // user wants a fits but the metadata tells this format is not available
                if (!availableFormats || (availableFormats && availableFormats.indexOf(imgFormat) >= 0)) {
                    this.imgFormat = imgFormat;

                    let [minCut, maxCut] = this.getCuts();
                    if (minCut === undefined && maxCut === undefined && imgFormat === "fits") {
                        // sets the default cuts parsed from the properties
                        this.setCuts(this.defaultFitsMinCut, this.defaultFitsMaxCut)
                    }
                } else {
                    console.warn(this.id + " does not provide " + imgFormat + " tiles")
                }
            }
        }

        this.options = {
            ...this.options,
            ...options,
            minCut: this.colorCfg.minCut,
            maxCut: this.colorCfg.maxCut
        };

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

    // @api
    HiPS.prototype.getColorCfg = function () {
        return this.colorCfg;
    };
    
    /**
     * Get the opacity of the HiPS layer
     * 
     * @memberof HiPS
     * 
     * @returns {number} The opacity of the layer
     */
    HiPS.prototype.getOpacity = function () {
        return this.colorCfg.getOpacity();
    };

    HiPS.prototype.getAlpha = HiPS.prototype.getOpacity;

    /**
     * Read a specific screen pixel value
     * 
     * @todo This has not yet been implemented
     * @memberof HiPS
     * @param {number} x - x axis in screen pixels to probe
     * @param {number} y - y axis in screen pixels to probe
     * @returns {number} the value of that pixel
     */
    HiPS.prototype.readPixel = function (x, y) {
        return this.view.wasm.readPixel(x, y, this.layer);
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

        let self = this;
        let hipsCache = this.view.aladin.hipsCache;

        if (hipsCache.contains(self.id)) {
            hipsCache.append(self.id, this.options)
        }
    };

    HiPS.prototype._add2View = function (layer) {
        this.layer = layer;
        let self = this;

        const config = {
            layer,
            properties: {
                creatorDid: self.creatorDid,
                url: self.url,
                maxOrder: self.maxOrder,
                cooFrame: self.cooFrame.system,
                tileSize: self.tileSize,
                formats: self.acceptedFormats,
                bitpix: self.numBitsPerPixel,
                skyFraction: self.skyFraction,
                minOrder: self.minOrder,
                hipsInitialFov: self.initialFov,
                hipsInitialRa: self.initialRa,
                hipsInitialDec: self.initialDec,
                hipsCubeDepth: self.cubeDepth,
                isPlanetaryBody: self.isPlanetaryBody(),
                hipsBody: self.hipsBody,
            },
            meta: {
                ...this.colorCfg.get(),
                imgFormat: this.imgFormat,
            }
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

    HiPS.DEFAULT_SURVEY_ID = "P/DSS2/color";

    return HiPS;
})();
