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
import { HiPSDefinition} from "./HiPSDefinition.js";
import { ALEvent } from "./events/ALEvent.js";
import { CooFrameEnum } from "./CooFrameEnum.js"
import { MocServer } from "./MocServer.js";
import { ColorCfg } from "./ColorCfg.js";

export async function fetchSurveyProperties(rootURLOrId) {
    if (!rootURLOrId) {
        throw 'An hosting survey URL or an ID (i.e. DSS2/red) must be given';
    }

    let isUrl = false;
    if (((Utils.isHttpContext() || Utils.isHttpsContext()) && rootURLOrId.includes("http")) || rootURLOrId.includes("file://")) {
        isUrl = true;
    }

    let metadata = {};
    // If an HiPS id has been given
    if (!isUrl) {
        // Use the MOCServer to retrieve the
        // properties
        const id = rootURLOrId;
        const params = {
            get: "record",
            fmt: "json",
            ID: "*" + id + "*",
        };

        metadata = await Utils.loadFromMirrors(MocServer.MIRRORS_HTTPS, {
            data: params,
        }).then(response => response.json());

        // We get the property here
        // 1. Ensure there is exactly one survey matching
        if (!metadata || metadata.length == 0) {
            throw 'No surveys matching have been found for the id: ' + id;
        } else {
            if (metadata.length > 1) {
                let ids = [];
                let surveyFound = false;
                for (let i = 0; i < metadata.length; i++) {
                    ids.push(metadata[i].ID)
                    if (metadata[i].ID === id) {
                        metadata = metadata[i];
                        surveyFound = true;
                        break;
                    }
                }

                if (!surveyFound) {
                    metadata = metadata[0];
                    console.warn(ids + ' surveys are matching. Please use one from this list. The chosen one is: ' + metadata);
                }
            } else {
                // Exactly one matching
                metadata = metadata[0];
            }
        }
    } else {
        // Fetch the properties of the survey
        let rootURL = rootURLOrId;
        // Use the url for retrieving the HiPS properties
        // remove final slash
        if (rootURL.slice(-1) === '/') {
            rootURL = rootURL.substr(0, rootURL.length-1);
        }

        // make URL absolute
        rootURL = Utils.getAbsoluteURL(rootURL);

        // fix for HTTPS support --> will work for all HiPS served by CDS
        rootURL = Utils.fixURLForHTTPS(rootURL)

        const url = rootURL + '/properties';
        metadata = await fetch(url)
            .then((response) => response.text());
        // We get the property here
        metadata = HiPSDefinition.parseHiPSProperties(metadata);

        // 1. Ensure there is exactly one survey matching
        if (!metadata) {
            throw 'No surveys matching at this url: ' + rootURL;
        }
        // Set the service url if not found
        metadata.hips_service_url = rootURLOrId;
    }

    return metadata;
}

export let ImageSurvey = (function() {
    /** Constructor
     * cooFrame and maxOrder can be set to null
     * They will be determined by reading the properties file
     *  
     */
    function ImageSurvey(id, name, rootURL, view, options) {
        // A reference to the view
        this.backend = view;
        this.added = false;
        this.id = id;
        this.name = name;

        // Image format
        let imgFormat = options && options.imgFormat;
        // transform to lower case
        if (imgFormat) {
            imgFormat = imgFormat.toLowerCase();
            // convert JPG -> JPEG
            if (imgFormat === "jpg") {
                imgFormat = "jpeg";
            }
    
            if (imgFormat === 'fits') {
                //tileFormat = "FITS";
                this.fits = true;
            } else if (imgFormat === "png") {
                //tileFormat = "PNG";
                this.fits = false;
            } else {
                // jpeg default case
                //tileFormat = "JPG";
                this.fits = false;
            }
        }

        this.imgFormat = imgFormat;
        
        // Longitude reversed
        let longitudeReversed = false;
        if (options && options.longitudeReversed === true) {
            longitudeReversed = true;
        }
        this.longitudeReversed = longitudeReversed;

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
        self.query = (async () => {
            const metadata = await fetchSurveyProperties(rootURL || id);
            // HiPS url
            self.name = self.name || metadata.obs_title;
            // Set it to a default value
            self.url = metadata.hips_service_url;
            // Request all the properties to see which mirror is the fastest
            self.getFastestHiPSMirror(metadata);

            if (!self.url) {
                throw 'no valid service URL for retrieving the tiles'
            }
            self.url = Utils.fixURLForHTTPS(self.url);

            // HiPS order
            const order = (+metadata.hips_order);

            // HiPS tile format
            let formats = metadata.hips_tile_format || "jpeg";
            formats = formats.split(' ').map((fmt) => fmt.toLowerCase());
            if (self.imgFormat) {
                // user wants a fits but the metadata tells this format is not available
                if (self.imgFormat === "fits" && formats.indexOf('fits') < 0) {
                    throw self.name + " does not provide fits tiles";
                }
                
                if (self.imgFormat === "png" && formats.indexOf('png') < 0) {
                    throw self.name + " does not provide png tiles";
                }
                
                if (self.imgFormat === "jpeg" && formats.indexOf('jpeg') < 0) {
                    throw self.name + " does not provide jpeg tiles";
                }
            } else {
                // user wants nothing then we choose one from the metadata
                if (formats.indexOf('png') >= 0) {
                    self.imgFormat = "png";
                    self.fits = false;
                } else if (formats.indexOf('jpeg') >= 0) {
                    self.imgFormat = "jpeg";
                    self.fits = false;
                } else if (formats.indexOf('fits') >= 0) {
                    self.imgFormat = "fits";
                    self.fits = true;
                } else {
                    throw "Unsupported format(s) found in the metadata: " + formats;
                }
            }

            // HiPS order min
            let hipsOrderMin = metadata.hips_order_min;
            if (hipsOrderMin === undefined) {
                hipsOrderMin = 3;
            } else {
                hipsOrderMin = +hipsOrderMin;
            }

            // HiPS tile size
            let tileSize = null;
            if (metadata.hips_tile_width === undefined) {
                tileSize = 512;
            } else {
                tileSize = +metadata.hips_tile_width;
            }

            // Check if the tile width size is a power of 2
            if (tileSize & (tileSize - 1) !== 0) {
                tileSize = 512;
            }

            // HiPS coverage sky fraction
            const skyFraction = +metadata.moc_sky_fraction || 0.0;
            // HiPS planet/planetoïde
            let cooFrame = undefined;
            if (metadata.hips_body !== undefined) {
                cooFrame = "ICRSd";
                self.longitudeReversed = true;
            }

            // HiPS frame
            cooFrame = cooFrame || metadata.hips_frame;
            let frame = null;

            if (cooFrame == "ICRS" || cooFrame == "ICRSd" || cooFrame == "equatorial" || cooFrame == "j2000") {
                frame = "ICRSJ2000";
            } else if (cooFrame == "galactic") {
                frame = "GAL";
            } else if (cooFrame === undefined) {
                frame = "ICRSJ2000";
                console.warn('No cooframe given. Coordinate systems supported: "ICRS", "ICRSd", "j2000" or "galactic". ICRS is chosen by default');
            } else {
                frame = "ICRSd";
                console.warn('Invalid cooframe given: ' + cooFrame + '. Coordinate systems supported: "ICRS", "ICRSd", "j2000" or "galactic". ICRS is chosen by default');
            }

            // HiPS initial fov/ra/dec
            let initialFov = +metadata.hips_initial_fov;
            const initialRa = +metadata.hips_initial_ra;
            const initialDec = +metadata.hips_initial_dec;

            if (initialFov < 0.00002777777) {
                initialFov = 360;
            }

            // Grayscale hips, this is not mandatory that there are fits tiles associated with it unfortunately
            // For colored HiPS, no fits tiles provided

            // HiPS cutouts
            let cuts = (metadata.hips_pixel_cut && metadata.hips_pixel_cut.split(" ")) || undefined;
            let propertiesDefaultMinCut = undefined;
            let propertiesDefaultMaxCut = undefined;
            if ( cuts ) {
                propertiesDefaultMinCut = parseFloat(cuts[0]);
                propertiesDefaultMaxCut = parseFloat(cuts[1]);
            }

            // HiPS bitpix
            let bitpix = undefined;
            if (metadata.hips_pixel_bitpix) {
                bitpix = +metadata.hips_pixel_bitpix;
            }

            self.colored = false;
            if (metadata.dataproduct_subtype === 'color') {
                self.colored = true;
            }
        
            self.properties = {
                url: self.url,
                maxOrder: order,
                frame: frame,
                tileSize: tileSize,
                formats: formats,
                minCutout: propertiesDefaultMinCut,
                maxCutout: propertiesDefaultMaxCut,
                bitpix: bitpix,
                skyFraction: skyFraction,
                minOrder: hipsOrderMin,
                hipsInitialFov: initialFov,
                hipsInitialRa: initialRa,
                hipsInitialDec: initialDec,
                colored: self.colored,
            };
            self.formats = formats;

            // Set the cuts at this point, if the user gave one, keep it
            // otherwise, set it to default values found in the HiPS properties
            let minCut, maxCut;
            if (!self.fits) {
                // Erase the cuts with the default one for image tiles
                minCut = self.colorCfg.minCut || 0.0;
                maxCut = self.colorCfg.maxCut || 1.0;
            } else {
                // For FITS hipses
                minCut = self.colorCfg.minCut || self.properties.minCutout;
                maxCut = self.colorCfg.maxCut || self.properties.maxCutout;
            }
            self.setCuts(minCut, maxCut);

            if (metadata.hips_body !== undefined) {
                if (self.backend.options.showFrame) {
                    self.backend.aladin.setFrame('J2000d');
                    let frameChoiceElt = document.querySelectorAll('.aladin-location > .aladin-frameChoice')[0];
                    frameChoiceElt.innerHTML = '<option value="' + CooFrameEnum.J2000d.label + '" selected="selected">J2000d</option>';
                }
            } else {
                if (self.backend.options.showFrame) {
                    const cooFrame = CooFrameEnum.fromString(self.backend.options.cooFrame, CooFrameEnum.J2000);
                    let frameChoiceElt = document.querySelectorAll('.aladin-location > .aladin-frameChoice')[0];
                    frameChoiceElt.innerHTML = '<option value="' + CooFrameEnum.J2000.label + '" '
                    + (cooFrame == CooFrameEnum.J2000 ? 'selected="selected"' : '') + '>J2000</option><option value="' + CooFrameEnum.J2000d.label + '" '
                    + (cooFrame == CooFrameEnum.J2000d ? 'selected="selected"' : '') + '>J2000d</option><option value="' + CooFrameEnum.GAL.label + '" '
                    + (cooFrame == CooFrameEnum.GAL ? 'selected="selected"' : '') + '>GAL</option>';
                }
            }

            ////// Update SURVEYS
            let idxSelectedHiPS = 0;
            const surveyFound = ImageSurvey.SURVEYS.some(s => {
                let res = self.id.endsWith(s.id);
                if (!res) {
                    idxSelectedHiPS += 1;
                }

                return res;
            });
            // The survey has not been found among the ones cached
            if (!surveyFound) {
                throw 'Should have been found!'
            } else {
                // Update the ImageSurvey
                let surveyDef = ImageSurvey.SURVEYS[idxSelectedHiPS];
                surveyDef.options = {
                    ...self.colorCfg.get(),
                    imgFormat: self.imgFormat,
                    longitudeReversed: self.longitudeReversed,
                };
                surveyDef.maxOrder = self.properties.maxOrder;
                surveyDef.url = self.properties.url;
            }

            return self;
        });
    };

    ImageSurvey.prototype.getFastestHiPSMirror = async function(metadata) {
        let self = this;
        const pingHipsServiceUrl = (hipsServiceUrl) => {
            const controller = new AbortController()

            let startRequestTime = Date.now();
            const maxTime = 2000;
            // 5 second timeout:
            const timeoutId = setTimeout(() => controller.abort(), maxTime)
            const promise = fetch(hipsServiceUrl + '/properties', { cache: 'no-store', signal: controller.signal, mode: "cors"}).then(response => {
                const duration = Date.now() - startRequestTime;//the time needed to do the request
                // completed request before timeout fired
                clearTimeout(timeoutId)
                // Resolve with the time duration of the request
                return { duration: duration, baseUrl: hipsServiceUrl, validRequest: true };
            }).catch((e) => {
                return { duration: maxTime, baseUrl: hipsServiceUrl, validRequest: false };
            });

            return promise;
        };

        // Get all the possible hips_service_url urls
        let promises = new Array();
        promises.push(pingHipsServiceUrl(metadata.hips_service_url));

        let numHiPSServiceURL = 1;
        while (metadata.hasOwnProperty("hips_service_url_" + numHiPSServiceURL.toString())) {
            const key = "hips_service_url_" + numHiPSServiceURL.toString();

            let curUrl = metadata[key];
            promises.push(pingHipsServiceUrl(curUrl))
            numHiPSServiceURL += 1;
        }

        let url = await Promise.all(promises).then((responses) => {
            // filter the ones that failed to not choose them
            // it may be a cors issue at this point
            let validResponses = responses.filter((resp) => { return resp.validRequest === true; });

            const getRandomIntInclusive = function(min, max) {
                min = Math.ceil(min);
                max = Math.floor(max);
                return Math.floor(Math.random() * (max - min + 1)) + min;
            };

            validResponses.sort((r1, r2) => {
                return r1.duration - r2.duration;
            });

            if (validResponses.length >= 2) {
                const isSecondUrlOk = ((validResponses[1].duration - validResponses[0].duration) / validResponses[0].duration) < 0.20;

                if (isSecondUrlOk) {
                    return validResponses[getRandomIntInclusive(0, 1)].baseUrl;
                } else {
                    return validResponses[0].baseUrl;
                }
            } else {
                return validResponses[0].baseUrl;
            }
        });


        let castToHTTPSUrl = ((url) => {
            if (Utils.isHttpsContext()) {
                const switchToHttps = Utils.HTTPS_WHITELIST.some(element => {
                    return url.includes(element);
                });
                if (switchToHttps) {
                    url = url.replace('http://', 'https://');
                }
            }

            return url;
        });

        url = castToHTTPSUrl(url);
        const pastUrl = castToHTTPSUrl(metadata.hips_service_url);
        // Change the backend survey url
        if (pastUrl !== url) {
            console.info("Change url of ", self.id, " from ", pastUrl, " to ", url)

            // If added to the backend, then we need to tell it the url has changed
            if (self.added) {
                self.backend.aladin.webglAPI.setHiPSUrl(pastUrl, url);
            }
        }

        self.url = url;
    }

    // @api
    // TODO: include imgFormat inside the ImageSurvey's meta attribute
    ImageSurvey.prototype.setImageFormat = function(format) {
        let self = this;

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
            if (self.properties) {
                const availableFormats = self.properties.formats;
                const idSurvey = self.properties.id;
                // user wants a fits but the metadata tells this format is not available
                if (imgFormat === "fits" && availableFormats.indexOf('fits') < 0) {
                    throw idSurvey + " does not provide fits tiles";
                }
                
                if (imgFormat === "png" && availableFormats.indexOf('png') < 0) {
                    throw idSurvey + " does not provide png tiles";
                }
                
                if (imgFormat === "jpeg" && availableFormats.indexOf('jpeg') < 0) {
                    throw idSurvey + " does not provide jpeg tiles";
                }
            }
    
            // Check if it is a fits
            self.imgFormat = imgFormat;
            self.fits = (self.imgFormat === 'fits');
        });
    };

    // @api
    ImageSurvey.prototype.setOpacity = function(opacity) {
        let self = this;
        updateMetadata(self, () => {
            self.colorCfg.setOpacity(opacity);
        });
    };

    // @api
    ImageSurvey.prototype.setBlendingConfig = function(additive = false) {
        updateMetadata(this, () => {
            this.colorCfg.setBlendingConfig(additive);
        });
    };

    // @api
    ImageSurvey.prototype.setColormap = function(colormap, options) {
        updateMetadata(this, () => {
            this.colorCfg.setColormap(colormap, options);
        });
    }

    // @api
    ImageSurvey.prototype.setCuts = function(lowCut, highCut) {
        updateMetadata(this, () => {
            this.colorCfg.setCuts(lowCut, highCut);
        });
    };

    // @api
    ImageSurvey.prototype.setGamma = function(gamma) {
        updateMetadata(this, () => {
            this.colorCfg.setGamma(gamma);
        });
    };

    // @api
    ImageSurvey.prototype.setSaturation = function(saturation) {
        updateMetadata(this, () => {
            this.colorCfg.setSaturation(saturation);
        });
    };

    ImageSurvey.prototype.setBrightness = function(brightness) {
        updateMetadata(this, () => {
            this.colorCfg.setBrightness(brightness);
        });
    };

    ImageSurvey.prototype.setContrast = function(contrast) {
        updateMetadata(this, () => {
            this.colorCfg.setContrast(contrast);
        });
    };

    ImageSurvey.prototype.metadata = function() {
        return {
            ...this.colorCfg.get(),
            longitudeReversed: this.longitudeReversed,
            imgFormat: this.imgFormat
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

    ImageSurvey.prototype.add = function(layer) {
        this.layer = layer;

        this.backend.aladin.webglAPI.addImageSurvey({
            layer: this.layer,
            properties: this.properties,
            meta: this.metadata(),
        });

        this.added = true;
    }

    // @api
    ImageSurvey.prototype.toggle = function() {
        if (this.colorCfg.getOpacity() != 0.0) {
            this.colorCfg.setOpacity(0.0);
        } else {
            this.colorCfg.setOpacity(this.prevOpacity);
        }
    };

    // @oldapi
    ImageSurvey.prototype.setAlpha = ImageSurvey.prototype.setOpacity;

    ImageSurvey.prototype.setColorCfg = function(colorCfg) {
        updateMetadata(this, () => {
            this.colorCfg = colorCfg;
        });
    };

    // @api
    ImageSurvey.prototype.getColorCfg = function() {
        return this.colorCfg;
    };
    
    // @api
    ImageSurvey.prototype.getOpacity = function() {
        return this.colorCfg.getOpacity();
    };

    ImageSurvey.prototype.getAlpha = ImageSurvey.prototype.getOpacity;

    // @api
    ImageSurvey.prototype.readPixel = function(x, y) {
        return this.backend.aladin.webglAPI.readPixel(x, y, this.layer);
    };

    /* Some constants */
    ImageSurvey.DEFAULT_SURVEY_ID = "P/DSS2/color";

    ImageSurvey.SURVEYS_OBJECTS = {};
    ImageSurvey.SURVEYS = [
        {
            id: "P/2MASS/color",
            name: "2MASS colored",
            url: "https://alasky.cds.unistra.fr/2MASS/Color",
            maxOrder: 9,
        },
        {
            id: "P/DSS2/color",
            name: "DSS colored",
            url: "https://alasky.cds.unistra.fr/DSS/DSSColor",
            maxOrder: 9,
        },
        {
            id: "P/DSS2/red",
            name: "DSS2 Red (F+R)",
            url: "https://alasky.cds.unistra.fr/DSS/DSS2Merged",
            maxOrder: 9,
            // options
            options: {
                minCut: 1000.0,
                maxCut: 10000.0,
                colormap: "magma",
                stretch: 'Linear',
                imgFormat: "fits"
            }
        },
        {
            id: "P/DM/I/350/gaiaedr3",
            name: "Density map for Gaia EDR3 (I/350/gaiaedr3)",
            url: "https://alasky.cds.unistra.fr/ancillary/GaiaEDR3/density-map",
            maxOrder: 7,
            // options
            options: {
                minCut: 0,
                maxCut: 12000,
                stretch: 'asinh',
                colormap: "rdylbu",
                imgFormat: "fits",
            }
        },
        {
            id: "P/PanSTARRS/DR1/g",
            name: "PanSTARRS DR1 g",
            url: "https://alasky.cds.unistra.fr/Pan-STARRS/DR1/g",
            maxOrder: 11,
            // options
            options: {
                minCut: -34,
                maxCut: 7000,
                stretch: 'asinh',
                colormap: "redtemperature",
                imgFormat: "fits",
            }
        },
        {
            id: "P/PanSTARRS/DR1/color-z-zg-g",
            name: "PanSTARRS DR1 color",
            url: "https://alasky.cds.unistra.fr/Pan-STARRS/DR1/color-z-zg-g",
            maxOrder: 11,    
        },
        {
            id: "P/DECaPS/DR1/color",
            name: "DECaPS DR1 color",
            url: "https://alasky.cds.unistra.fr/DECaPS/DR1/color",
            maxOrder: 11,
        },
        {
            id: "P/Fermi/color",
            name: "Fermi color",
            url: "https://alasky.cds.unistra.fr/Fermi/Color",
            maxOrder: 3,
        },
        {
            id: "P/Finkbeiner",
            name: "Halpha",
            url: "https://alasky.cds.unistra.fr/FinkbeinerHalpha",
            maxOrder: 3,
            // options
            options: {
                minCut: -10,
                maxCut: 800,
                colormap: "rdbu",
                imgFormat: "fits",
            }
        },
        {
            id: "P/GALEXGR6_7/NUV",
            name: "GALEXGR6_7 NUV",
            url: "http://alasky.cds.unistra.fr/GALEX/GALEXGR6_7_NUV/",
            maxOrder: 8,
        },
        {
            id: "P/IRIS/color",
            name: "IRIS colored",
            url: "https://alasky.cds.unistra.fr/IRISColor",
            maxOrder: 3,
        },
        {
            id: "P/Mellinger/color",
            name: "Mellinger colored",
            url: "https://alasky.cds.unistra.fr/MellingerRGB",
            maxOrder: 4,
        },
        {
            id: "P/SDSS9/color",
            name: "SDSS9 colored",
            url: "https://alasky.cds.unistra.fr/SDSS/DR9/color",
            maxOrder: 10,
        },
        {
            id: "P/SDSS9/g",
            name: "SDSS9 band-g",
            url: "https://alasky.cds.unistra.fr/SDSS/DR9/band-g",
            maxOrder: 10,
            options: {
                stretch: 'asinh',
                colormap: "redtemperature",
                imgFormat: 'fits'
            }
        },
        {
            id: "P/SPITZER/color",
            name: "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
            url: "http://alasky.cds.unistra.fr/Spitzer/SpitzerI1I2I4color/",
            maxOrder: 9,
        },
        {
            id: "P/VTSS/Ha",
            name: "VTSS-Ha",
            url: "https://alasky.cds.unistra.fr/VTSS/Ha",
            maxOrder: 3,
            options: {
                minCut: -10.0,
                maxCut: 100.0,
                colormap: "grayscale",
                imgFormat: "fits"
            }
        },
        /*
        // Seems to be not hosted on saada anymore
        {
            id: "P/XMM/EPIC",
            name: "XMM-Newton stacked EPIC images (no phot. normalization)",
            url: "https://alasky.cds.unistra.fr/cgi/JSONProxy?url=https://saada.cds.unistra.fr/xmmallsky",
            maxOrder: 7,
        },*/
        {
            id: "xcatdb/P/XMM/PN/color",
            name: "XMM PN colored",
            url: "https://alasky.cds.unistra.fr/cgi/JSONProxy?url=https://saada.unistra.fr/PNColor",
            maxOrder: 7,
        },
        {
            id: "P/allWISE/color",
            name: "AllWISE color",
            url: "https://alasky.cds.unistra.fr/AllWISE/RGB-W4-W2-W1/",
            maxOrder: 8,
        },
        /*//The page is down
        {
            id: "P/GLIMPSE360",
            name: "GLIMPSE360",
            url: "https://alasky.cds.unistra.fr/cgi/JSONProxy?url=http://www.spitzer.caltech.edu/glimpse360/aladin/data",
            maxOrder: 9,
        },*/
    ];

    ImageSurvey.getAvailableSurveys = function() {
        return ImageSurvey.SURVEYS;
    };

    return ImageSurvey;
})();

