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
    let HpxImageSurvey = function(rootURLOrId, view, options) {
        this.backend = view;
        this.orderIdx = null;
        this.ready = false;
        this.layer = null;
        this.properties = null;
        this.meta = {
            color: {
                grayscale2Color: {
                    color: (options && options.color) || [1.0, 1.0, 1.0],
                    k: (options && options.strength) || 1.0,
                    param: {
                        h: (options && options.tf) || "Asinh",
                        minValue: (options && options.mincut) || 0,
                        maxValue: (options && options.maxcut) || 1
                    }
                }
            },
            blendCfg: {
                srcColorFactor: 'SrcAlpha',
                dstColorFactor: 'OneMinusSrcAlpha',
                func: 'FuncAdd' 
            },
            opacity: (options && options.opacity) || 1.0,
        };

        (async () => {
            const metadata = await fetchSurveyProperties(rootURLOrId);

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
            const color = this.meta.color;
            const strength = this.meta.color.grayscale2Color.strength;

            if (options && options.colormap) {
                renderCfg = {
                    grayscale2Colormap: {
                        colormap: colormap,
                        reversed: reversed,
                        param: param
                    }
                };
            } else {
                // no options have been given or without any colormap or single color referenced
                if (tileFormat === "FITS") {
                    renderCfg = color;
                } else {
                    console.log("eeee");
                    this.meta.color = "color";
                }
            }

            const opacity = this.meta.opacity;
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

            // The survey created is associated to no layer when it is created
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
            /*this.meta = {
                color: renderCfg,
                blendCfg: blendingCfg,
                opacity: opacity,
            };*/

            if (this.orderIdx < this.backend.imageSurveysIdx.get(this.layer)) {
                // discard that
                return;
            }

            const addedToTheView = this.layer !== undefined;
            if (addedToTheView) {
                // If the layer has been set then it is linked to the aladin lite view
                // Update the layer
                this.backend.setOverlayImageSurvey(this, null, this.layer);

                this.ready = true;
            }
        })();
    };

    // @api
    HpxImageSurvey.prototype.setAlpha = function(alpha) {
        alpha = +alpha; // coerce to number
        this.meta.opacity = Math.max(0, Math.min(alpha, 1));

        // Tell the view its meta have changed
        if( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }
    };

    // @api
    HpxImageSurvey.prototype.setColor = function(color, options) {
        let param = (this.meta.color.grayscale2Color && this.meta.color.grayscale2Color.param) || (this.meta.color.grayscale2Colormap && this.meta.color.grayscale2Colormap.param);
        param.h = (options && options.tf) || param.h;
        param.minValue = (options && options.mincut) || param.minValue;
        param.maxValue = (options && options.maxcut) || param.maxValue;

        if ( Array.isArray(color) ) {
            console.log("setcolor", color);

            const strength = (this.meta.color.grayscale2Colormap && this.meta.color.grayscale2Colormap.k) || (options && options.strength) || false;
            this.meta.color = {
                grayscale2Color: {
                    color: color,
                    k: strength,
                    param: param,
                }
            };

            console.log("setcolor", this.meta.color, this.meta);

        } else if (typeof color === "String") {
            const reversed = (this.meta.color.grayscale2Colormap && this.meta.color.grayscale2Colormap.reversed) || (options && options.reversed) || false;
            this.meta.color = {
                grayscale2Colormap: {
                    colormap: color,
                    reversed: reversed,
                    param: param
                }
            };
        }

        console.log("setColor", this)
        // Tell the view its meta have changed
        if ( this.ready ) {
            this.backend.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }
    };

    HpxImageSurvey.prototype.setCuts = function(cuts) {
        let param = (this.meta.color.grayscale2Color && this.meta.color.grayscale2Color.param) || (this.meta.color.grayscale2Colormap && this.meta.color.grayscale2Colormap.param);
        param.minValue = cuts[0];
        param.maxValue = cuts[1];

        if ( this.meta.color.grayscale2Color ) {
            this.meta.color.grayscale2Color.param = param;
        } else {
            this.meta.color.grayscale2Colormap.param = param;
        }

        console.log("meeetaeee", this.meta, this.layer);
        // Tell the view its meta have changed
        if ( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }
    };
    
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
                minCutout: 10.0,
                maxCutout: 10000.0,
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
                minCutout: -34,
                maxCutout: 7000
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
                bitpix: -32,
                minCutout: -10,
                maxCutout: 800,
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
        /*
        // Seems to be not hosted on saada anymore
        {
            properties: {
                id: "P/XMM/EPIC",
                name: "XMM-Newton stacked EPIC images (no phot. normalization)",
                url: "https://alasky.u-strasbg.fr/cgi/JSONProxy?url=https://saada.u-strasbg.fr/xmmallsky",
                maxOrder: 7,
                frame: "ICRSJ2000",
                longitudeReversed: false,
                tileSize: 512,
                format: "PNG",
            },
            meta: {
                color: "color",
                opacity: 1.0,
            }
        },*/
        {
            properties: {
                id: "P/XMM/PN/color",
                name: "XMM PN colored",
                url: "https://alasky.u-strasbg.fr/cgi/JSONProxy?url=https://saada.unistra.fr/PNColor",
                maxOrder: 7,
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
        /*
        The page is down
        {
            properties: {
                id: "P/GLIMPSE360",
                name: "GLIMPSE360",
                url: "https://alasky.u-strasbg.fr/cgi/JSONProxy?url=http://www.spitzer.caltech.edu/glimpse360/aladin/data",
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
        },*/
    ];

    HpxImageSurvey.getAvailableSurveys = function(view) {
        const surveys = HpxImageSurvey.SURVEYS
            .map(obj => {
                let survey = Object.assign(new HpxImageSurvey(""), obj);
                survey.backend = view;
                return survey;
            });
        console.log(surveys)
        return surveys;
    };

    return HpxImageSurvey;
})();

