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
                metadata.forEach((prop) => {
                    ids.push(prop.ID)
                });
                throw ids + ' surveys are matching. Please use one from this list.';
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
        if (Utils.isHttpsContext() && ( /u-strasbg.fr/i.test(rootURL) || /unistra.fr/i.test(rootURL)  ) ) {
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
    let HpxImageSurvey = function(metadata, aladin, options) {
        // HiPS url
        let url = metadata.hips_service_url;
        if (!url) {
            throw 'no valid service URL for retrieving the tiles'
        }

        if (url.startsWith('http://alasky')) {
            // From alasky one can directly use the https access
            url = url.replace('http', 'https');
        } else {
            // Pass by a proxy for extern http urls
            url = 'https://alasky.u-strasbg.fr/cgi/JSONProxy?url=' + url;
        }

        // HiPS order
        const order = (+metadata.hips_order);
        // HiPS cutouts
        let cuts = (metadata.hips_pixel_cut && metadata.hips_pixel_cut.split(" ")) || undefined;
        if (cuts) {
            cuts = [parseFloat(cuts[0]), parseFloat(cuts[1])]
        }

        const minCut = (options && options.mincut) || (cuts && cuts[0]) || 0;
        const maxCut = (options && options.maxcut) || (cuts && cuts[1]) || 1;

        // HiPS tile format
        let tileFormat;
        const tileFormats = (options && options.imgFormat) || metadata.hips_tile_format.split(' ');
        if (tileFormats.indexOf('fits') >= 0) {
            tileFormat = "FITS";
        } else if (tileFormats.indexOf('png') >= 0) {
            tileFormat = "PNG";
        } else if (tileFormats.indexOf('jpeg') >= 0) {
            tileFormat = "JPG";
        } else {
            throw "Only FITS, PNG or JPG tile format supported";
        }

        // HiPS tile size
        const tileSize = +metadata.hips_tile_width;
        // HiPS bitpix
        const bitpix = +metadata.hips_pixel_bitpix;
        // HiPS frame
        let frame = (options && options.cooFrame) || metadata.hips_frame || "equatorial";
        if (frame == "equatorial") {
            frame = "ICRSJ2000";
        } else if (frame == "galactic") {
            frame = "GAL";
        } else {
            frame = "ICRSJ2000";
        }
        // HiPS longitude reversed
        const longitude_reversed = (options && options.reversedLongitude) || false;

        // HiPS render options
        let renderCfg;
        const colormap = (options && options.colormap) || 'blackwhite';
        const reversed = (options && options.reversed) || true;
        const param = {
            h: (options && options.tf) || "Asinh",
            minValue: minCut,
            maxValue: maxCut
        };
        const color = (options && options.color) || [1, 1, 1];
        const strength = (options && options.strength) || 1.0;

        if (options && options.colormap) {
            renderCfg = {
                grayscale2Colormap: {
                    colormap: colormap,
                    reversed: reversed,
                    param: param
                }
            };
        } else if (options && options.color) {
            renderCfg = {
                grayscale2Color: {
                    color: color,
                    k: strength,
                    param: param
                }
            };
        } else {
            // no options have been given or without any colormap or single color referenced
            if (tileFormat === "FITS") {
                renderCfg = {
                    grayscale2Color: {
                        color: color,
                        k: strength,
                        param: param
                    }
                };
            } else {
                renderCfg = "color";
            }            
        }

        const opacity = (options && options.opacity) || 1.0;
        const additiveBlending = (options && options.additive) || false;
        let blendingCfg;
        if (additiveBlending) {
            blendingCfg = {
                srcColorFactor: 'SrcAlpha',
                dstColorFactor: 'One',
                func: 'FuncAdd' 
            }
        } else {
            blendingCfg = {
                srcColorFactor: 'SrcAlpha',
                dstColorFactor: 'OneMinusSrcAlpha',
                func: 'FuncAdd' 
            }
        }
        this.backend = aladin.webglAPI;
        console.log("BACKEND", this.backend);
        // The survey created is associated to no layer when it is created
        this.layer = null;
        this.properties = {
            url: url,
            maxOrder: order,
            frame: frame,
            longitudeReversed: longitude_reversed,
            tileSize: tileSize,
            format: tileFormat,
            minCutout: minCut,
            maxCutout: maxCut,
            bitpix: bitpix,
        };
        this.meta = {
            color: renderCfg,
            blendCfg: blendingCfg,
            opacity: opacity,
        };
    };

    // @api
    HpxImageSurvey.prototype.setAlpha = function(alpha) {
        alpha = +alpha; // coerce to number
        this.meta.opacity = Math.max(0, Math.min(alpha, 1));

        // Tell the view its meta have changed
        this.backend.setImageSurveyMeta(this.layer, this.meta);
    };

    // @api
    /*HpxImageSurvey.prototype.setColor = function(color) {
        this.meta.color = Math.max(0, Math.min(alpha, 1));

        // Tell the view its meta have changed
        this.backend.setImageSurveyMeta(this.layer, this.meta);
    };*/

    // This method is called by the view object to
    // signal to the backend whether the view must be recomputed or not
    /*HpxImageSurvey.prototype.isMetaChanged = function() {
        const metaChanged = this.needRedraw;
        this.needRedraw = false;

        return metaChanged;
    };*/
    
    // @api
    HpxImageSurvey.prototype.getAlpha = function() {
        return this.meta.opacity;
    };

    HpxImageSurvey.DEFAULT_SURVEY_ID = "P/DSS2/color";
    
    HpxImageSurvey.SURVEYS_OBJECTS = {};
    HpxImageSurvey.SURVEYS = [
        {
            "id": "P/2MASS/color",
            "url": "http://alasky.u-strasbg.fr/2MASS/Color",
            "name": "2MASS colored",
            "maxOrder": 9,
            "frame": "equatorial",
            "format": "jpeg"
        },
        {
            "id": "P/DSS2/color",
            "url": "http://alasky.u-strasbg.fr/DSS/DSSColor",
            "name": "DSS colored",
            "maxOrder": 9,
            "frame": "equatorial",
            "format": "jpeg"
        },
        {
            "id": "P/DSS2/red",
            "url": "http://alasky.u-strasbg.fr/DSS/DSS2Merged",
            "name": "DSS2 Red (F+R)",
            "maxOrder": 9,
            "frame": "equatorial",
            "format": "jpeg fits"
        },
        {
            "id": "P/PanSTARRS/DR1/g",
            "url": "http://alasky.u-strasbg.fr/Pan-STARRS/DR1/g",
            "name": "PanSTARRS DR1 g",
            "maxOrder": 11,
            "frame": "equatorial",
            "format": "jpeg fits"
        },
        {
            "id": "P/PanSTARRS/DR1/color-z-zg-g",
            "url": "http://alasky.u-strasbg.fr/Pan-STARRS/DR1/color-z-zg-g",
            "name": "PanSTARRS DR1 color",
            "maxOrder": 11,
            "frame": "equatorial",
            "format": "jpeg"
        },
        {
            "id": "P/DECaPS/DR1/color",
            "url": "http://alasky.u-strasbg.fr/DECaPS/DR1/color",
            "name": "DECaPS DR1 color",
            "maxOrder": 11,
            "frame": "equatorial",
            "format": "jpeg png"
        },
        {
            "id": "P/Fermi/color",
            "url": "http://alasky.u-strasbg.fr/Fermi/Color",
            "name": "Fermi color",
            "maxOrder": 3,
            "frame": "equatorial",
            "format": "jpeg"
        },
        {
            "id": "P/Finkbeiner",
            "url": "http://alasky.u-strasbg.fr/FinkbeinerHalpha",
            "maxOrder": 3,
            "frame": "galactic",
            "format": "jpeg fits",
            "name": "Halpha"
        },
        {
            "id": "P/GALEXGR6/AIS/color",
            "url": "http://alasky.unistra.fr/GALEX/GR6-03-2014/AIS-Color",
            "name": "GALEX Allsky Imaging Survey colored",
            "maxOrder": 8,
            "frame": "equatorial",
            "format": "jpeg"
        },
        {
            "id": "P/IRIS/color",
            "url": "http://alasky.u-strasbg.fr/IRISColor",
            "name": "IRIS colored",
            "maxOrder": 3,
            "frame": "galactic",
            "format": "jpeg"
        },
        {
            "id": "P/Mellinger/color",
            "url": "http://alasky.u-strasbg.fr/MellingerRGB",
            "name": "Mellinger colored",
            "maxOrder": 4,
            "frame": "galactic",
            "format": "jpeg"
        },
        {
            "id": "P/SDSS9/color",
            "url": "http://alasky.u-strasbg.fr/SDSS/DR9/color",
            "name": "SDSS9 colored",
            "maxOrder": 10,
            "frame": "equatorial",
            "format": "jpeg"
        },
        {
            "id": "P/SPITZER/color",
            "url": "http://alasky.u-strasbg.fr/SpitzerI1I2I4color",
            "name": "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
            "maxOrder": 9,
            "frame": "galactic",
            "format": "jpeg"
        },
        {
            "id": "P/VTSS/Ha",
            "url": "http://alasky.u-strasbg.fr/VTSS/Ha",
            "maxOrder": 3,
            "frame": "galactic",
            "format": "png jpeg fits",
            "name": "VTSS-Ha"
        },
        {
            "id": "P/XMM/EPIC",
            "url": "http://saada.u-strasbg.fr/xmmallsky",
            "name": "XMM-Newton stacked EPIC images (no phot. normalization)",
            "maxOrder": 7,
            "frame": "equatorial",
            "format": "png fits"
        },
        {
            "id": "P/XMM/PN/color",
            "url": "http://saada.unistra.fr/PNColor",
            "name": "XMM PN colored",
            "maxOrder": 7,
            "frame": "equatorial",
            "format": "png jpeg"
        },
        {
            "id": "P/allWISE/color",
            "url": "http://alasky.u-strasbg.fr/AllWISE/RGB-W4-W2-W1/",
            "name": "AllWISE color",
            "maxOrder": 8,
            "frame": "equatorial",
            "format": "jpeg"
        },
        {
            "id": "P/GLIMPSE360",
            "url": "http://www.spitzer.caltech.edu/glimpse360/aladin/data",
            "name": "GLIMPSE360",
            "maxOrder": 9,
            "frame": "equatorial",
            "format": "jpeg"
        }
    ];

    HpxImageSurvey.getAvailableSurveys = function() {
    	return HpxImageSurvey.SURVEYS;
    };

    return HpxImageSurvey;
})();

