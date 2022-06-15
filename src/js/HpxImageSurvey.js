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
 * File HpxImageSurvey
 * 
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 * 
 *****************************************************************************/
import { Utils } from "./Utils.js";
import { HiPSDefinition} from "./HiPSDefinition.js";
import { convertToHsl } from "daisyui/src/colors/functions";
import { ALEvent } from "./events/ALEvent.js";

export async function fetchSurveyProperties(rootURLOrId) {
    if (!rootURLOrId) {
        throw 'An hosting survey URL or an ID (i.e. DSS2/red) must be given';
    }

    let isUrl = false;
    if (rootURLOrId.includes("http")) {
        isUrl = true;
    }

    const request = async (url) => {
        const response = await fetch(url);
        const json = await response.json();

        return json;
    };

    let metadata = {};
    // If an HiPS id has been given
    if (!isUrl) {
        // Use the MOCServer to retrieve the
        // properties
        const id = rootURLOrId;
        const MOCServerUrl = 'https://alasky.unistra.fr/MocServer/query?ID=*' + encodeURIComponent(id) + '*&get=record&fmt=json';

        metadata = await request(MOCServerUrl);

        // We get the property here

        // 1. Ensure there is exactly one survey matching
        if (!metadata) {
            throw 'no surveys matching';
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
                    throw ids + ' surveys are matching. Please use one from this list.';
                }
            } else if (metadata.length === 0) {
                throw 'no surveys matching';
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

        // fast fix for HTTPS support --> will work for all HiPS served by CDS
        if (Utils.isHttpsContext() && ( /cds.unistra.fr/i.test(rootURL) || /unistra.fr/i.test(rootURL)  ) ) {
            rootURL = rootURL.replace('http://', 'https://');
        }

        const url = rootURL + '/properties';
        metadata = await fetch(url)
            .then((response) => response.text());
        // We get the property here
        metadata = HiPSDefinition.parseHiPSProperties(metadata);

        // 1. Ensure there is exactly one survey matching
        if (!metadata) {
            throw 'no surveys matching';
        }
        // Set the service url if not found
        metadata.hips_service_url = rootURLOrId;
    }

    return metadata;
}

export let HpxImageSurvey = (function() {
    /** Constructor
     * cooFrame and maxOrder can be set to null
     * They will be determined by reading the properties file
     *  
     */
    function HpxImageSurvey(rootURLOrId, view, options, callback) {
        // A reference to the view
        this.backend = view;
        // A number used to ensure the correct layer ordering in the aladin lite view
        this.orderIdx = null;
        // Set to true once its metadata has been received
        this.ready = false;
        // Name of the layer
        this.layer = null;
        this.added = false;
        // Default options
        this.options = {
            ...{
                longitudeReversed: false,
                reversed: false,
                stretch: "Linear",
                opacity: 1.0,
            },
            ...options
        };
        
        if (this.options.imgFormat) {
            // Img format preprocessing
            // transform to upper case
            this.options.imgFormat = this.options.imgFormat.toUpperCase();

            // convert JPG -> JPEG
            if (this.options.imgFormat === "JPG") {
                this.options.imgFormat = "JPEG";
            }
        }

        if (this.options.imgFormat === 'FITS') {
            //tileFormat = "FITS";
            this.fits = true;
        } else if (this.options.imgFormat === "PNG") {
            //tileFormat = "PNG";
            this.fits = false;
        } else {
            // jpeg default case
            //tileFormat = "JPG";
            this.fits = false;
        }

        this.updateMeta();

        (async () => {
            const metadata = await fetchSurveyProperties(rootURLOrId);

            // HiPS url
            let id = metadata.creator_did;
            let name = metadata.obs_title;
            let url = metadata.hips_service_url;
            if (!url) {
                throw 'no valid service URL for retrieving the tiles'
            }

            if (url.startsWith('http://alasky')) {
                // From alasky one can directly use the https access
                url = url.replace('http', 'https');
                url = url.replace('https://alasky.cds.unistra.fr/', 'https://alasky.cds.unistra.fr/');
            }

            // HiPS order
            const order = (+metadata.hips_order);

            // HiPS tile format
            const tileFormats = metadata.hips_tile_format.split(' ').map((fmt) => fmt.toUpperCase());
            if (this.options.imgFormat) {
                // user wants a fits but the metadata tells this format is not available
                if (this.options.imgFormat === "FITS" && tileFormats.indexOf('FITS') < 0) {
                    throw name + " does not provide fits tiles";
                }
                
                if (this.options.imgFormat === "PNG" && tileFormats.indexOf('PNG') < 0) {
                    throw name + " does not provide png tiles";
                }
                
                if (this.options.imgFormat === "JPEG" && tileFormats.indexOf('JPEG') < 0) {
                    throw name + " does not provide jpeg tiles";
                }
            } else {
                // user wants nothing then we choose one from the metadata
                if (tileFormats.indexOf('PNG') >= 0) {
                    this.options.imgFormat = "PNG";
                    this.fits = false;
                } else if (tileFormats.indexOf('JPEG') >= 0) {
                    this.options.imgFormat = "JPEG";
                    this.fits = false;
                } else if (tileFormats.indexOf('FITS') >= 0) {
                    this.options.imgFormat = "FITS";
                    this.fits = true;
                } else {
                    throw "Unsupported format(s) found in the metadata: " + tileFormats;
                }
            }

            // HiPS tile size
            const tileSize = +metadata.hips_tile_width;

            // HiPS coverage sky fraction
            const skyFraction = +metadata.moc_sky_fraction || 1.0;

            // HiPS frame
            this.options.cooFrame = this.options.cooFrame || metadata.hips_frame;
            let frame = null;
            if (this.options.cooFrame == "ICRS" || this.options.cooFrame == "ICRSd" || this.options.cooFrame == "equatorial") {
                frame = "ICRSJ2000";
            } else if (this.options.cooFrame == "galactic") {
                frame = "GAL";
            } else {
                throw 'Coordinate systems supported: "ICRS", "ICRSd" or "galactic"';
            }

            // HiPS grayscale
            this.colored = false;
            if (metadata.dataproduct_subtype && (metadata.dataproduct_subtype === "color" || metadata.dataproduct_subtype[0] === "color") ) {
                this.colored = true;
            }

            if (!this.colored) {
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
                const bitpix = +metadata.hips_pixel_bitpix;

                this.properties = {
                    id: id,
                    name: name,
                    url: url,
                    maxOrder: order,
                    frame: frame,
                    tileSize: tileSize,
                    formats: tileFormats,
                    minCutout: propertiesDefaultMinCut,
                    maxCutout: propertiesDefaultMaxCut,
                    bitpix: bitpix,
                    skyFraction: skyFraction
                };
            } else {
                this.properties = {
                    id: id,
                    name: name,
                    url: url,
                    maxOrder: order,
                    frame: frame,
                    tileSize: tileSize,
                    formats: tileFormats,
                    skyFraction: skyFraction
                };
            }

            if (!this.colored) {
                // For grayscale JPG/PNG hipses
                if (!this.fits) {
                    // Erase the cuts with the default one for image tiles
                    this.options.minCut = this.options.minCut || 0.0;
                    this.options.maxCut = this.options.maxCut || 1.0;
                // For FITS hipses
                } else {
                    this.options.minCut = this.options.minCut || this.properties.minCutout;
                    this.options.maxCut = this.options.maxCut || this.properties.maxCutout;
                }
            }

            this.updateMeta();
            this.ready = true;

            // Discard further processing if the layer has been associated to another hips
            // before the request has been resolved
            if (this.orderIdx < this.backend.imageSurveysIdx.get(this.layer)) {
                return;
            }

            if (callback) {
                callback(this);
            }

            // If the layer has been set then it is linked to the aladin lite view
            // Update the layer
            if (this.added) {
                this.backend.setOverlayImageSurvey(this, this.layer);
            }
        })();
    };

    HpxImageSurvey.prototype.updateMeta = function() {
        let blend = {
            srcColorFactor: 'SrcAlpha',
            dstColorFactor: 'OneMinusSrcAlpha',
            func: 'FuncAdd' 
        };

        const additiveBlending = this.options.additive;
        if (additiveBlending) {
            blend = {
                srcColorFactor: 'SrcAlpha',
                dstColorFactor: 'One',
                func: 'FuncAdd' 
            }
        }
        // reset the whole meta object
        this.meta = {};
        // populate him with the updated fields
        this.updateColor();
        this.meta.blendCfg = blend;
        this.meta.opacity = this.options.opacity;
    }

    HpxImageSurvey.prototype.updateColor = function() {
        if (this.colored) {
            this.meta.color = "color";
        } else {
            if (this.options.color) {
                this.meta.color = {
                    grayscale: {
                        stretch: this.options.stretch,
                        minCut: this.options.minCut,
                        maxCut: this.options.maxCut,
                        color: {
                            color: this.options.color,
                        }
                    }
                };
            } else {
                // If not defined we set the colormap to grayscale
                if (!this.options.colormap) {
                    this.options.colormap = "grayscale";
                }

                if (this.options.colormap === "native") {
                    this.options.colormap = "grayscale";
                }

                this.meta.color = {
                    grayscale: {
                        stretch: this.options.stretch,
                        minCut: this.options.minCut,
                        maxCut: this.options.maxCut,
                        color: {
                            colormap: {
                                reversed: this.options.reversed,
                                name: this.options.colormap,
                            }
                        }
                    }
                };
            }
        }
    };

    // @api
    HpxImageSurvey.prototype.setOpacity = function(opacity) {
        opacity = +opacity; // coerce to number
        this.options.opacity = Math.max(0, Math.min(opacity, 1));

        this.updateMeta();

        // Tell the view its meta have changed
        if( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }

        if (this.added) {
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    };

    // @api
    HpxImageSurvey.prototype.getOpacity = function() {
        return this.options.opacity;
    };

    // @api
    HpxImageSurvey.prototype.setBlendingConfig = function(additive = false) {
        this.options.additive = additive;

        this.updateMeta();

        // Tell the view its meta have changed
        if( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }

        if (this.added) {
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    };

    // @api
    HpxImageSurvey.prototype.setColor = function(color, options) {
        this.options = {...this.options, ...options};
        // Erase the colormap given first
        if (this.options.colormap) {
            delete this.options.colormap;
        }
        this.options.color = color;

        this.updateColor();

        // Tell the view its meta have changed
        if ( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }

        if (this.added) {
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    };

    // @api
    HpxImageSurvey.prototype.setColormap = function(colormap, options) {
        this.options = {...this.options, ...options};
        // Erase the color given first
        if (this.options.color) {
            delete this.options.color;
        }
        this.options.colormap = colormap;

        this.updateColor();
        // Tell the view its meta have changed
        if ( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }

        if (this.added) {
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    }

    // @api
    HpxImageSurvey.prototype.setCuts = function(cuts) {
        this.options.minCut = cuts[0];
        this.options.maxCut = cuts[1];

        this.updateColor();

        // Tell the view its meta have changed
        if ( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }

        if (this.added) {
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    };

    // @api
    HpxImageSurvey.prototype.changeImageFormat = function(format) {
        let imgFormat = format.toUpperCase();
        if (imgFormat !== "FITS" && imgFormat !== "PNG" && imgFormat !== "JPG" && imgFormat !== "JPEG") {
            throw 'Formats must lie in ["fits", "png", "jpg"]';
        }

        if (imgFormat === "JPG") {
            imgFormat = "JPEG";
        }

        // Check the properties to see if the given format is available among the list
        // If the properties have not been retrieved yet, it will be tested afterwards
        if (this.properties) {
            const availableFormats = this.properties.formats;
            const idSurvey = this.properties.id;
            // user wants a fits but the metadata tells this format is not available
            if (imgFormat === "FITS" && availableFormats.indexOf('FITS') < 0) {
                throw idSurvey + " does not provide fits tiles";
            }
            
            if (imgFormat === "PNG" && availableFormats.indexOf('PNG') < 0) {
                throw idSurvey + " does not provide png tiles";
            }
            
            if (imgFormat === "JPEG" && availableFormats.indexOf('JPEG') < 0) {
                throw idSurvey + " does not provide jpeg tiles";
            }
        }

        // Passed the check, we erase the image format with the new one
        // We do nothing if the imgFormat is the same
        if (this.options.imgFormat === imgFormat) {
            return;
        }

        this.options.imgFormat = imgFormat;
        // Check if it is a fits
        this.fits = (this.options.imgFormat === 'FITS');

        // Tell the view its meta have changed
        if ( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyImageFormat(this.layer, imgFormat);
        }

        if (this.added) {
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    };

    // @api
    HpxImageSurvey.prototype.getMeta = function() {
        return this.meta;
    };
    
    // @api
    HpxImageSurvey.prototype.getAlpha = function() {
        return this.meta.opacity;
    };

    // @api
    HpxImageSurvey.prototype.readPixel = function(x, y) {
        return this.backend.aladin.webglAPI.readPixel(x, y, this.layer);
    };

    HpxImageSurvey.DEFAULT_SURVEY_ID = "P/DSS2/color";
    
    HpxImageSurvey.SURVEYS_OBJECTS = {};
    HpxImageSurvey.SURVEYS = [
        {
            id: "CDS/P/2MASS/color",
            name: "2MASS colored",
            url: "https://alasky.cds.unistra.fr/2MASS/Color",
            maxOrder: 9,
        },
        {
            id: "CDS/P/DSS2/color",
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
                imgFormat: "fits",
                colormap: "rainbow",
                stretch: 'Linear'
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
                stretch: 'Asinh',
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
            id: "P/GALEXGR6_7/color",
            name: "GALEX GR6/7 - Color composition",
            url: "https://alasky.cds.unistra.fr/GALEX/GALEXGR6_7_color/",
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
                stretch: 'Asinh',
                colormap: "redtemperature",
                imgFormat: "fits",
            }
        },
        {
            id: "P/SPITZER/color",
            name: "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
            url: "https://alasky.cds.unistra.fr/SpitzerI1I2I4color",
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
            id: "CDS/P/allWISE/color",
            name: "AllWISE color",
            url: "https://alasky.cds.unistra.fr/AllWISE/RGB-W4-W2-W1/",
            maxOrder: 8,
        },
        /*
        The page is down
        {
            id: "P/GLIMPSE360",
            name: "GLIMPSE360",
            url: "https://alasky.cds.unistra.fr/cgi/JSONProxy?url=http://www.spitzer.caltech.edu/glimpse360/aladin/data",
            maxOrder: 9,
        },*/
    ];

    HpxImageSurvey.getAvailableSurveys = function() {
        return HpxImageSurvey.SURVEYS;
    };

    return HpxImageSurvey;
})();

