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
            properties: {
                id: "P/2MASS/color",
                name: "2MASS colored",
                url: "http://alasky.u-strasbg.fr/2MASS/Color",
                maxOrder: 9,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/DSS2/color",
                name: "DSS colored",
                url: "http://alasky.u-strasbg.fr/DSS/DSSColor",
                maxOrder: 9,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/DSS2/red",
                name: "DSS2 Red (F+R)",
                url: "http://alasky.u-strasbg.fr/DSS/DSS2Merged",
                maxOrder: 9,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "FITS",
                bitpix: 16,
            },
            meta: {
                color: {
                    grayscale2Color: {
                        color: [1.0, 0.0, 0.0],
                        k: 1.0,
                        param: {
                            h: "Asinh",
                            minValue: 10.0,
                            maxValue: 10000.0,
                        }
                    }
                },
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/PanSTARRS/DR1/g",
                name: "PanSTARRS DR1 g",
                url: "http://alasky.u-strasbg.fr/Pan-STARRS/DR1/g",
                maxOrder: 11,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "FITS",
                bitpix: -32,
            },
            meta: {
                color: {
                    grayscale2Colormap: {
                        colormap: "redTemperature",
                        reversed: false,
                        param: {
                            h: "Asinh",
                            minValue: -34,
                            maxValue: 7000,
                        }
                    }
                },
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/PanSTARRS/DR1/color-z-zg-g",
                name: "PanSTARRS DR1 color",
                url: "http://alasky.u-strasbg.fr/Pan-STARRS/DR1/color-z-zg-g",
                maxOrder: 11,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/DECaPS/DR1/color",
                name: "DECaPS DR1 color",
                url: "http://alasky.u-strasbg.fr/DECaPS/DR1/color",
                maxOrder: 11,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "PNG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/Fermi/color",
                name: "Fermi color",
                url: "http://alasky.u-strasbg.fr/Fermi/Color",
                maxOrder: 3,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/Finkbeiner",
                name: "Halpha",
                url: "http://alasky.u-strasbg.fr/FinkbeinerHalpha",
                maxOrder: 3,
                frame: "GAL",
                longitudeReversed: false,
                tileSize: 128,
                format: "FITS",
                bitpix: -32
            },
            meta: {
                color: {
                    grayscale2Colormap: {
                        colormap: "redTemperature",
                        reversed: false,
                        param: {
                            h: "Asinh",
                            minValue: -10,
                            maxValue: 800,
                        }
                    }
                },
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/GALEXGR6/AIS/color",
                name: "GALEX Allsky Imaging Survey colored",
                url: "http://alasky.unistra.fr/GALEX/GR6-03-2014/AIS-Color",
                maxOrder: 8,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/IRIS/color",
                name: "IRIS colored",
                url: "http://alasky.u-strasbg.fr/IRISColor",
                maxOrder: 3,
                frame: "GAL",
                longitudeReversed: false,
                tileSize: 256,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/Mellinger/color",
                name: "Mellinger colored",
                url: "http://alasky.u-strasbg.fr/MellingerRGB",
                maxOrder: 4,
                frame: "GAL",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/SDSS9/color",
                name: "SDSS9 colored",
                url: "http://alasky.u-strasbg.fr/SDSS/DR9/color",
                maxOrder: 10,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/SPITZER/color",
                name: "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
                url: "http://alasky.u-strasbg.fr/SpitzerI1I2I4color",
                maxOrder: 9,
                frame: "GAL",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/VTSS/Ha",
                name: "VTSS-Ha",
                url: "http://alasky.u-strasbg.fr/VTSS/Ha",
                maxOrder: 3,
                frame: "GAL",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/XMM/EPIC",
                name: "XMM-Newton stacked EPIC images (no phot. normalization)",
                url: "http://saada.u-strasbg.fr/xmmallsky",
                maxOrder: 7,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/XMM/PN/color",
                name: "XMM PN colored",
                url: "http://saada.unistra.fr/PNColor",
                maxOrder: 7,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/allWISE/color",
                name: "AllWISE color",
                url: "http://alasky.u-strasbg.fr/AllWISE/RGB-W4-W2-W1/",
                maxOrder: 8,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
        {
            properties: {
                id: "P/GLIMPSE360",
                name: "GLIMPSE360",
                url: "http://www.spitzer.caltech.edu/glimpse360/aladin/data",
                maxOrder: 9,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "JPG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },
    ];

    HpxImageSurvey.getAvailableSurveys = function() {
    	return HpxImageSurvey.SURVEYS;
    };

    return HpxImageSurvey;
})();

