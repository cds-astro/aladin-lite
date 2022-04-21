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
        // A reference to the view
        this.backend = view;
        // A number used to ensure the correct layer ordering in the aladin lite view
        this.orderIdx = null;
        // Set to true once its metadata has been received
        this.ready = false;
        // Name of the layer
        this.layer = null;

        let blend = null;
        const additiveBlending = (options && options.additive) || false;
        if (additiveBlending) {
            blend = {
                srcColorFactor: 'SrcAlpha',
                dstColorFactor: 'One',
                func: 'FuncAdd' 
            }
        } else {
            blend = {
                srcColorFactor: 'SrcAlpha',
                dstColorFactor: 'OneMinusSrcAlpha',
                func: 'FuncAdd' 
            }
        }

        // HiPS tile format
        let tileFormat = null;
        const tileFormats = (options && options.imgFormat) || "jpeg";
        if (tileFormats && tileFormats.indexOf('fits') >= 0) {
            tileFormat = "FITS";
            this.fits = true;
        } else if (tileFormats && tileFormats.indexOf('png') >= 0) {
            tileFormat = "PNG";
            this.fits = false;
        } else {
            tileFormat = "JPG";
            this.fits = false;
        }

        if (this.fits && (options && options.colormap)) {
            this.meta = {
                color: {
                    grayscale: {
                        tf: (options && options.tf) || "Asinh",
                        minCut: options && options.minCut,
                        maxCut: options && options.maxCut,
                        color: {
                            colormap: {
                                reversed: (options && options.reversed) || false,
                                colormap: (options && options.colormap) || 'blackwhite',
                            }
                        }
                    }
                },
                blendCfg: blend,
                opacity: (options && options.opacity) || 1.0,
            };
        } else if(this.fits) {
            const color = (options && options.color) || [1.0, 1.0, 1.0, 1.0];
            this.meta = {
                color: {
                    grayscale: {
                        tf: (options && options.tf) || "Asinh",
                        minCut: options && options.minCut,
                        maxCut: options && options.maxCut,
                        color: {
                            color: color,
                        }
                    }
                },
                blendCfg: blend,
                opacity: (options && options.opacity) || 1.0,
            };
        } else {
            this.meta = {
                color: "color",
                blendCfg: blend,
                opacity: (options && options.opacity) || 1.0,
            };
        }

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

            // HiPS tile format
            let tileFormat;
            const tileFormats = metadata.hips_tile_format.split(' ');
            if (this.fits) {
                // user wants a fits file
                if (tileFormats.indexOf('fits') >= 0) {
                    tileFormat = "FITS";
                } else {
                    throw name + " has only image tiles and not fits ones";
                }
            } else {
                if (tileFormats.indexOf('png') >= 0) {
                    tileFormat = "PNG";
                } else if (tileFormats.indexOf('jpeg') >= 0) {
                    tileFormat = "JPG";
                } else {
                    throw name + " has only fits tiles and not image ones";
                }
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
            let minCut = cuts && cuts[0];
            let maxCut = cuts && cuts[1];

            if (this.fits) {
                // If the survey received is a fits one
                // update the cuts
                console.log("grayscale", this.meta.color, cuts)
                this.meta.color.grayscale.minCut = this.meta.color.grayscale.minCut || minCut;
                this.meta.color.grayscale.maxCut = this.meta.color.grayscale.maxCut || maxCut;
            }

            // The survey created is associated to no layer when it is created
            this.properties = {
                id: id,
                name: name,
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
        if (!this.fits) {
            throw 'Can only set the color of a FITS survey but this survey contains tile images.';
        }

        // This has a grayscale color associated        
        const tf = (options && options.tf) || this.meta.color.grayscale.tf;
        const minCut = (options && options.minCut) || this.meta.color.grayscale.minCut;
        const maxCut = (options && options.maxCut) || this.meta.color.grayscale.maxCut;

        let newColor = null;
        if ( Array.isArray(color) ) {
            newColor = {
                grayscale: {
                    minCut: minCut,
                    maxCut: maxCut,
                    tf: tf,
                    color: {
                        color: color
                    }
                }
            };
        } else if (typeof color === "string") {
            const reversed = (options && options.reversed) || (this.meta.color.grayscale.color.colormap && this.meta.color.grayscale.color.colormap.reversed) || false;

            newColor = {
                grayscale: {
                    minCut: minCut,
                    maxCut: maxCut,
                    tf: tf,
                    color: {
                        colormap: {
                            reversed: reversed,
                            colormap: color,
                        },
                    }
                }
            };
        } else {
            throw "The color given: " + color + " is not recognized";
        }

        this.meta.color = newColor;

        // Tell the view its meta have changed
        if ( this.ready ) {
            this.backend.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }
    };

    HpxImageSurvey.prototype.setCuts = function(cuts) {
        if (!this.fits) {
            throw 'Can only set the color of a FITS survey but this survey contains tile images.';
        }

        this.meta.color.grayscale.minCut = cuts[0];
        this.meta.color.grayscale.maxCut = cuts[1];

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
            id: "P/2MASS/color",
            name: "2MASS colored",
            url: "https://alasky.u-strasbg.fr/2MASS/Color",
            maxOrder: 9,
        },
        {
            id: "P/DSS2/color",
            name: "DSS colored",
            url: "https://alasky.u-strasbg.fr/DSS/DSSColor",
            maxOrder: 9,
        },
        {
            id: "P/DSS2/red",
            name: "DSS2 Red (F+R)",
            url: "https://alasky.u-strasbg.fr/DSS/DSS2Merged",
            maxOrder: 9,
            // options
            options: {
                minCut: 10.0,
                maxCut: 10000.0,
                color: [1.0, 0.0, 0.0, 1.0],
                imgFormat: "fits",
            }
        },
        {
            id: "P/PanSTARRS/DR1/g",
            name: "PanSTARRS DR1 g",
            url: "https://alasky.u-strasbg.fr/Pan-STARRS/DR1/g",
            maxOrder: 11,
            // options
            options: {
                minCut: -34,
                maxCut: 7000,
                colormap: "redTemperature",
                imgFormat: "fits",
            }
        },
        {
            id: "P/PanSTARRS/DR1/color-z-zg-g",
            name: "PanSTARRS DR1 color",
            url: "https://alasky.u-strasbg.fr/Pan-STARRS/DR1/color-z-zg-g",
            maxOrder: 11,    
        },
        {
            id: "P/DECaPS/DR1/color",
            name: "DECaPS DR1 color",
            url: "https://alasky.u-strasbg.fr/DECaPS/DR1/color",
            maxOrder: 11,
        },
        {
            id: "P/Fermi/color",
            name: "Fermi color",
            url: "https://alasky.u-strasbg.fr/Fermi/Color",
            maxOrder: 3,
        },
        {
            id: "P/Finkbeiner",
            name: "Halpha",
            url: "https://alasky.u-strasbg.fr/FinkbeinerHalpha",
            maxOrder: 3,
            // options
            options: {
                minCut: -10,
                maxCut: 800,
                colormap: "rdBu",
                imgFormat: "fits",
            }
        },
        {
            id: "P/GALEXGR6/AIS/color",
            name: "GALEX Allsky Imaging Survey colored",
            url: "https://alasky.unistra.fr/GALEX/GR6-03-2014/AIS-Color",
            maxOrder: 8,
        },
        {
            id: "P/IRIS/color",
            name: "IRIS colored",
            url: "https://alasky.u-strasbg.fr/IRISColor",    
            maxOrder: 3,
        },
        {
            id: "P/Mellinger/color",
            name: "Mellinger colored",
            url: "https://alasky.u-strasbg.fr/MellingerRGB",
            maxOrder: 4,
        },
        {
            id: "P/SDSS9/color",
            name: "SDSS9 colored",
            url: "https://alasky.u-strasbg.fr/SDSS/DR9/color",
            maxOrder: 10,
        },
        {
            id: "P/SPITZER/color",
            name: "IRAC color I1,I2,I4 - (GLIMPSE, SAGE, SAGE-SMC, SINGS)",
            url: "https://alasky.u-strasbg.fr/SpitzerI1I2I4color",
            maxOrder: 9,
        },
        {
            id: "P/VTSS/Ha",
            name: "VTSS-Ha",
            url: "https://alasky.u-strasbg.fr/VTSS/Ha",
            maxOrder: 3,
            options: {
                minCut: -10.0,
                maxCut: 100.0,
                colormap: "blackwhite",
                imgFormat: "fits"
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
            id: "P/XMM/PN/color",
            name: "XMM PN colored",
            url: "https://alasky.u-strasbg.fr/cgi/JSONProxy?url=https://saada.unistra.fr/PNColor",
            maxOrder: 7,
        },
        {
            id: "P/allWISE/color",
            name: "AllWISE color",
            url: "https://alasky.u-strasbg.fr/AllWISE/RGB-W4-W2-W1/",
            maxOrder: 8,
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

    /*HpxImageSurvey.createFromProperties = function(properties, meta, view) {
        let survey = new HpxImageSurvey("", view);

        survey.meta = meta;
        survey.properties = properties;

        return survey;
    }*/

    HpxImageSurvey.getAvailableSurveys = function() {
        /*const surveys = HpxImageSurvey.SURVEYS
            .map(obj => {
                let survey = Object.assign(new HpxImageSurvey("", view), obj);
                return survey;
            });
        console.log(surveys)
        return surveys;*/
        return HpxImageSurvey.SURVEYS;
    };

    return HpxImageSurvey;
})();

