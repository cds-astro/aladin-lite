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
        //let tileFormat = null;
        let imgFormat = (options && options.imgFormat);
        if (imgFormat) {
            this.imgFormat = imgFormat.toUpperCase();

            if (this.imgFormat === 'FITS') {
                //tileFormat = "FITS";
                this.fits = true;
            } else if (this.imgFormat === "PNG") {
                //tileFormat = "PNG";
                this.fits = false;
            } else {
                //tileFormat = "JPG";
                this.fits = false;
            }
        }

        if (this.fits && (options && options.colormap)) {
            this.meta = {
                color: {
                    grayscale: {
                        tf: (options && options.tf) || "Linear",
                        minCut: options && options.minCut,
                        maxCut: options && options.maxCut,
                        color: {
                            colormap: {
                                reversed: (options && options.reversed) || false,
                                name: (options && options.colormap) || 'grayscale',
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
                        tf: (options && options.tf) || "Linear",
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
                url = url.replace('https://alasky.cds.unistra.fr/', 'https://alasky.cds.unistra.fr/');
            }

            // HiPS order
            const order = (+metadata.hips_order);
            // HiPS cutouts
            let cuts = (metadata.hips_pixel_cut && metadata.hips_pixel_cut.split(" ")) || undefined;
            if (cuts) {
                cuts = [parseFloat(cuts[0]), parseFloat(cuts[1])]
            }

            // HiPS tile format
            const tileFormats = metadata.hips_tile_format.split(' ').map((fmt) => fmt.toUpperCase());
            if (this.fits) {
                // user wants a fits file
                if (tileFormats.indexOf('FITS') < 0) {
                    throw name + " has only image tiles and not fits ones";
                }
            } else {
                if (tileFormats.indexOf('PNG') < 0 && tileFormats.indexOf('JPEG') < 0) {
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
            const longitudeReversed = (options && options.longitudeReversed) || false;
            // HiPS render options
            let minCut = cuts && cuts[0];
            let maxCut = cuts && cuts[1];

            if ( this.fits ) {
                // If the survey received is a fits one
                // update the cuts
                minCut = this.meta.color.grayscale.minCut || minCut;
                maxCut = this.meta.color.grayscale.maxCut || maxCut;

                this.meta.color.grayscale.minCut = minCut;
                this.meta.color.grayscale.maxCut = maxCut;
            }

            // the output format has not been defined by the user
            // => we give him one of the available formats
            if (!this.imgFormat) {
                if (tileFormats.indexOf('PNG') >= 0) {
                    this.fits = false;
                    this.imgFormat = "PNG";
                } else if (tileFormats.indexOf('JPEG') >= 0) {
                    this.fits = false;
                    this.imgFormat = "JPEG";
                } else if (tileFormats.indexOf('FITS') >= 0) {
                    this.fits = true;
                    this.imgFormat = "FITS";
                } else {
                    throw "Unsupported format(s) found in the metadata: " + tileFormats;
                }
            }
            console.log("moc sky fraction: ", metadata.moc_sky_fraction)
            const skyFraction = +metadata.moc_sky_fraction || 1.0;
            // The survey created is associated to no layer when it is created
            this.properties = {
                id: id,
                name: name,
                url: url,
                maxOrder: order,
                frame: frame,
                longitudeReversed: longitudeReversed,
                tileSize: tileSize,
                formats: tileFormats,
                minCutout: minCut,
                maxCutout: maxCut,
                bitpix: bitpix,
                skyFraction: skyFraction
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
        //if (!this.fits) {
        //    throw 'Can only set the color of a FITS survey but this survey contains tile images.';
        //}

        if ( !this.fits ) {
            // This has a grayscale color associated        
            const tf = (options && options.tf) || "Linear";
            const minCut = (options && options.minCut) || 0.0;
            const maxCut = (options && options.maxCut) || 1.0;

            const newColor = {
                grayscale: {
                    minCut: minCut,
                    maxCut: maxCut,
                    tf: tf,
                    color: {
                        color: color
                    }
                }
            };

            // Update the color config
            this.meta.color = newColor;
        } else {
            // This has a grayscale color associated        
            const tf = (options && options.tf) || this.meta.color.grayscale.tf;
            const minCut = (options && options.minCut) || this.meta.color.grayscale.minCut;
            const maxCut = (options && options.maxCut) || this.meta.color.grayscale.maxCut;

            const newColor = {
                grayscale: {
                    minCut: minCut,
                    maxCut: maxCut,
                    tf: tf,
                    color: {
                        color: color
                    }
                }
            };

            // Update the color config
            this.meta.color = newColor;
        }

        // Tell the view its meta have changed
        if ( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }
    };

    HpxImageSurvey.prototype.setColormap = function(colormap, reversed, options) {
        //if (!this.fits) {
        //    throw 'Can only set the color of a FITS survey but this survey contains tile images.';
        //}

        if ( !this.fits ) {
            if (colormap === "native") {
                this.meta.color = "color";
            } else {
                const tf = (options && options.tf) || "Linear";
                const minCut = (options && options.minCut) || 0.0;
                const maxCut = (options && options.maxCut) || 1.0;
                const rev = reversed || false;
    
                this.meta.color = {
                    grayscale: {
                        minCut: minCut,
                        maxCut: maxCut,
                        tf: tf,
                        color: {
                            colormap: {
                                reversed: rev,
                                name: colormap,
                            },
                        }
                    }
                };
            }
        } else {
            // This has a grayscale color associated        
            const tf = (options && options.tf) || this.meta.color.grayscale.tf || "Linear";
            const minCut = (options && options.minCut) || this.meta.color.grayscale.minCut;
            const maxCut = (options && options.maxCut) || this.meta.color.grayscale.maxCut;
            const rev = reversed || (this.meta.color.grayscale.color.colormap && this.meta.color.grayscale.color.colormap.reversed) || false;

            // Update the color config
            this.meta.color = {
                grayscale: {
                    minCut: minCut,
                    maxCut: maxCut,
                    tf: tf,
                    color: {
                        colormap: {
                            reversed: rev,
                            name: colormap,
                        },
                    }
                }
            };
        }

        // Tell the view its meta have changed
        if ( this.ready ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
        }
    }

    HpxImageSurvey.prototype.setCuts = function(cuts) {
        if (!this.fits) {
            throw 'Can only set the color of a FITS survey but this survey contains tile images.';
        }

        // Update the mincut/maxcut
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
                minCut: 10.0,
                maxCut: 10000.0,
                color: [1.0, 0.0, 0.0, 1.0],
                imgFormat: "fits",
                colormap: "rainbow",
                tf: 'Linear'
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
                minCut: -14000,
                maxCut: -9000,
                tf: 'Asinh',
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

