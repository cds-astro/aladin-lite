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
import { ALEvent } from "./events/ALEvent.js";
import { Location } from "./Location.js";
import { CooFrameEnum } from "./CooFrameEnum.js"

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
        const MOCServerUrl = 'https://alasky.cds.unistra.fr/MocServer/query?ID=*' + encodeURIComponent(id) + '*&get=record&fmt=json';

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

        // fix for HTTPS support --> will work for all HiPS served by CDS
        rootURL = Utils.fixURLForHTTPS(rootURL)

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
    function HpxImageSurvey(id, name, rootURL, view, options) {
        // A reference to the view
        this.backend = view;
        // A number used to ensure the correct layer ordering in the aladin lite view
        this.orderIdx = null;
        // Set to true once its metadata has been received
        this.ready = false;
        // Name of the layer
        this.layer = null;
        this.added = false;
        this.id = id;
        this.name = name;
        // Default options
        this.options = options || {};
        
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

        if (this.options.longitudeReversed === undefined) {
            this.options.longitudeReversed = false;
        }


        if (this.options.opacity === undefined) {
            this.options.opacity = 1.0;
        }

        let idxSelectedHiPS = 0;
        const surveyFound = HpxImageSurvey.SURVEYS.some(s => {
            let res = this.id.endsWith(s.id);
            if (!res) {
                idxSelectedHiPS += 1;
            }

            return res;
        });
        // The survey has not been found among the ones cached
        if (!surveyFound) {
            HpxImageSurvey.SURVEYS.push({
                id: this.id,
                name: this.name,
                options: this.options
            });
        } else {
            let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
            surveyDef.options = this.options;
        }

        this.updateMeta();
        let self = this;
        (async () => {
            const metadata = await fetchSurveyProperties(rootURL || id);

            // HiPS url
            self.name = self.name || metadata.obs_title;
            let url = metadata.hips_service_url;
            if (!url) {
                throw 'no valid service URL for retrieving the tiles'
            }

            url = Utils.fixURLForHTTPS(url);

            // HiPS order
            const order = (+metadata.hips_order);

            // HiPS tile format
            const tileFormats = metadata.hips_tile_format.split(' ').map((fmt) => fmt.toUpperCase());
            if (self.options.imgFormat) {
                // user wants a fits but the metadata tells this format is not available
                if (self.options.imgFormat === "FITS" && tileFormats.indexOf('FITS') < 0) {
                    throw self.name + " does not provide fits tiles";
                }
                
                if (self.options.imgFormat === "PNG" && tileFormats.indexOf('PNG') < 0) {
                    throw self.name + " does not provide png tiles";
                }
                
                if (self.options.imgFormat === "JPEG" && tileFormats.indexOf('JPEG') < 0) {
                    throw self.name + " does not provide jpeg tiles";
                }
            } else {
                // user wants nothing then we choose one from the metadata
                if (tileFormats.indexOf('PNG') >= 0) {
                    self.options.imgFormat = "PNG";
                    self.fits = false;
                } else if (tileFormats.indexOf('JPEG') >= 0) {
                    self.options.imgFormat = "JPEG";
                    self.fits = false;
                } else if (tileFormats.indexOf('FITS') >= 0) {
                    self.options.imgFormat = "FITS";
                    self.fits = true;
                } else {
                    throw "Unsupported format(s) found in the metadata: " + tileFormats;
                }
            }

            // HiPS tile size
            let tileSize = +metadata.hips_tile_width;
            if (tileSize & (tileSize - 1) !== 0) {
                // not a power of 2
                tileSize = 512;
            }

            // HiPS coverage sky fraction
            const skyFraction = +metadata.moc_sky_fraction || 1.0;
            
            let removeAllChildNodes = function removeAllChildNodes(parent) {
                while (parent.firstChild) {
                    parent.removeChild(parent.firstChild);
                }
            };
            // HiPS planet/planetoïde
            if (metadata.hips_body !== undefined) {
                self.options.cooFrame = "ICRSd";
                self.options.longitudeReversed = true;
            }

            // HiPS frame
            self.options.cooFrame = self.options.cooFrame || metadata.hips_frame;
            let frame = null;
            if (self.options.cooFrame == "ICRS" || self.options.cooFrame == "ICRSd" || self.options.cooFrame == "equatorial" || self.options.cooFrame == "j2000") {
                frame = "ICRSJ2000";
            } else if (self.options.cooFrame == "galactic") {
                frame = "GAL";
            } else if (self.options.cooFrame === undefined) {
                frame = "ICRSJ2000";
                console.warn('No cooframe given. Coordinate systems supported: "ICRS", "ICRSd", "j2000" or "galactic". ICRS is chosen by default');
            } else {
                frame = "ICRSd";
                console.warn('Invalid cooframe given: ' + self.options.cooFrame + '. Coordinate systems supported: "ICRS", "ICRSd", "j2000" or "galactic". ICRS is chosen by default');
            }

            // HiPS grayscale
            self.colored = false;
            if (metadata.dataproduct_subtype && (metadata.dataproduct_subtype.includes("color") || metadata.dataproduct_subtype[0].includes("color") )) {
                self.colored = true;
            }

            if (!self.colored) {
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

                self.properties = {
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
                self.properties = {
                    url: url,
                    maxOrder: order,
                    frame: frame,
                    tileSize: tileSize,
                    formats: tileFormats,
                    skyFraction: skyFraction
                };
            }

            if (!self.colored) {
                self.options.stretch = self.options.stretch || "Linear";

                // For grayscale JPG/PNG hipses
                if (!self.fits) {
                    // Erase the cuts with the default one for image tiles
                    self.options.minCut = self.options.minCut || 0.0;
                    self.options.maxCut = self.options.maxCut || 255.0;
                // For FITS hipses
                } else {
                    self.options.minCut = self.options.minCut || self.properties.minCutout;
                    self.options.maxCut = self.options.maxCut || self.properties.maxCutout;
                }
            }
            // Discard further processing if the layer has been associated to another hips
            // before the request has been resolved
            if (self.orderIdx < self.backend.imageSurveysIdx.get(self.layer)) {
                return;
            }

            if (metadata.hips_body !== undefined) {
                if (self.backend.options.showFrame) {
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

            self.updateMeta();
            self.ready = true;

            ////// Update SURVEYS
            let idxSelectedHiPS = 0;
            const surveyFound = HpxImageSurvey.SURVEYS.some(s => {
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
                // Update the HpxImageSurvey
                let surveyDef = HpxImageSurvey.SURVEYS[idxSelectedHiPS];
                surveyDef.options = self.options;
                surveyDef.maxOrder = self.properties.maxOrder;
                surveyDef.url = self.properties.url;
            }
            /////

            // If the layer has been set then it is linked to the aladin lite view
            // so we add it
            if (self.added) {
                self.backend.commitSurveysToBackend(self, self.layer);
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
        this.meta.longitudeReversed = this.options.longitudeReversed;
    }

    HpxImageSurvey.prototype.updateColor = function() {
        if (this.colored) {
            this.meta.color = "color";
        } else {
            let minCut = this.options.minCut;
            let maxCut = this.options.maxCut;

            if (this.options.imgFormat !== "FITS") {
                minCut /= 255.0;
                maxCut /= 255.0;
            }

            if (this.options.color) {
                this.meta.color = {
                    grayscale: {
                        stretch: this.options.stretch,
                        minCut: minCut,
                        maxCut: maxCut,
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

                let reversed = this.options.reversed;
                if (this.options.reversed === undefined) {
                    reversed = false;
                }

                this.meta.color = {
                    grayscale: {
                        stretch: this.options.stretch,
                        minCut: minCut,
                        maxCut: maxCut,
                        color: {
                            colormap: {
                                reversed: reversed,
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
        if( this.ready && this.added ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    };

    HpxImageSurvey.prototype.setAlpha = HpxImageSurvey.prototype.setOpacity;

    // @api
    HpxImageSurvey.prototype.getOpacity = function() {
        return this.options.opacity;
    };

    // @api
    HpxImageSurvey.prototype.setBlendingConfig = function(additive = false) {
        this.options.additive = additive;

        this.updateMeta();

        // Tell the view its meta have changed
        if( this.ready && this.added ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
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
        if( this.ready && this.added ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
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
        if( this.ready && this.added ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    }

    // @api
    HpxImageSurvey.prototype.setCuts = function(cuts) {
        this.options.minCut = cuts[0];
        this.options.maxCut = cuts[1];

        this.updateColor();

        // Tell the view its meta have changed
        if( this.ready && this.added ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    };

    HpxImageSurvey.prototype.setOptions = function(options) {
        this.options = options;
        this.updateMeta();

        // Tell the view its meta have changed
        if( this.ready && this.added ) {
            this.backend.aladin.webglAPI.setImageSurveyMeta(this.layer, this.meta);
            ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
        }
    };

    // @api
    HpxImageSurvey.prototype.changeImageFormat = function(format) {
        const prevImageFmt = this.options.imgFormat;
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
        try {
            if ( this.ready && this.added ) {
                this.backend.aladin.webglAPI.setImageSurveyImageFormat(this.layer, imgFormat);
                ALEvent.HIPS_LAYER_CHANGED.dispatchedTo(this.backend.aladinDiv, {survey: this});
            }
        } catch(e) {
            console.error(e);

            this.options.imgFormat = prevImageFmt;
            this.fits = (this.options.imgFormat === 'FITS');
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
                colormap: "magma",
                stretch: 'Linear'
            }
        },
        /*{
            id: "P/MeerKAT/Galactic-Centre-1284MHz-StokesI",
            name: "MeerKAT Galactic Centre 1284MHz StokesI",
            url: "https://alasky.cds.unistra.fr/MeerKAT/CDS_P_MeerKAT_Galactic-Centre-1284MHz-StokesI",
            maxOrder: 9,
            // options
            options: {
                minCut: -4e-4,
                maxCut: 0.01,
                imgFormat: "fits",
                colormap: "rainbow",
                stretch: 'Linear'
            }
        },*/
        {
            id: "P/DM/I/350/gaiaedr3",
            name: "Density map for Gaia EDR3 (I/350/gaiaedr3)",
            url: "https://alasky.cds.unistra.fr/ancillary/GaiaEDR3/density-map",
            maxOrder: 7,
            // options
            options: {
                minCut: 0,
                maxCut: 12000,
                stretch: 'Asinh',
                colormap: "rdyibu",
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

