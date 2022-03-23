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

export let HpxImageSurvey = (function() {
    /** Constructor
     * cooFrame and maxOrder can be set to null
     * They will be determined by reading the properties file
     *  
     */
    let HpxImageSurvey = function(rootURLOrId) {
        this.survey = null;
        if (!rootURLOrId) {
            throw 'An hosting survey URL or an ID (i.e. DSS2/red) must be given';
        }

        let isUrl = false;
        console.log("root url", rootURLOrId)
        if (rootURLOrId.includes("http")) {
            isUrl = true;
        }

        const request = async (url) => {
            const response = await fetch(url);
            const json = await response.json();

            return json;
        };

        // If an HiPS id has been given
        let url = null;
        if (!isUrl) {
            // Use the MOCServer to retrieve the
            // properties
            const id = rootURLOrId;
            const MOCServerUrl = 'https://alasky.unistra.fr/MocServer/query?ID=*' + encodeURIComponent(id) + '*&get=record&fmt=json';

            return (async () => {
                let metadata = await request(MOCServerUrl);

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
                // Let is build the survey object
                return HpxImageSurvey.parseSurveyProperties(metadata);
            })();
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

            console.log("ROOT URL", rootURL);
            url = rootURL + '/properties';


            return (async () => {
                console.log("properties url", url);
                let metadata = await fetch(url)
                    .then((response) => response.text());
                // We get the property here
                metadata = HiPSDefinition.parseHiPSProperties(metadata);
                console.log("metadata", metadata);

                // 1. Ensure there is exactly one survey matching
                if (!metadata) {
                    throw 'no surveys matching';
                }
                // Set the service url if not found
                metadata.hips_service_url = rootURLOrId;
                // Let is build the survey object
                return HpxImageSurvey.parseSurveyProperties(metadata);
            })();
        }
    };

    HpxImageSurvey.parseSurveyProperties = function(metadata) {
        const order = (+metadata.hips_order);
        const hipsTileFormat = metadata.hips_tile_format.split(' ');
        let cuts = [undefined, undefined];
        if (metadata.hips_pixel_cut) {
            cuts = metadata.hips_pixel_cut.split(" ");
        }

        let tileFormat;
        let color;

        if (hipsTileFormat.indexOf('fits') >= 0) {
            tileFormat = {
                FITSImage: {
                    bitpix: parseInt(metadata.hips_pixel_bitpix)
                }
            };
            color = {
                grayscale2Colormap: {
                    colormap: "rdBu",
                    reversed: false,
                    param: {
                        h: "Asinh",
                        minValue: parseFloat(cuts[0]),
                        maxValue: parseFloat(cuts[1])
                    }
                }
                /*grayscale2Color: {
                    color: [1.0, 1.0, 1.0],
                    k: 1.0,
                    param: {
                        h: "Asinh",
                        minValue: parseFloat(cuts[0]),
                        maxValue: parseFloat(cuts[1])
                    }
                }*/
                /*grayscale2Color: {
                    color: [1, 1, 1],
                    k: 1,
                    param: {
                        h: "Asinh",
                        minValue: parseFloat(cuts[0]) || 0,
                        maxValue: parseFloat(cuts[1]) || 1
                    }
                }*/
            };
        } else {
            color = "color";

            if (hipsTileFormat.indexOf('png') >= 0) {
                tileFormat = {
                    Image: {
                        format: "png"
                    }
                };
            } else {
                tileFormat = {
                    Image: {
                        format: "jpeg"
                    }
                };
            }
        }

        let tileSize = 512;
        // Verify the validity of the tile width
        if (metadata.hips_tile_width) {
            let hipsTileWidth = parseInt(metadata.hips_tile_width);
            let isPowerOfTwo = hipsTileWidth && !(hipsTileWidth & (hipsTileWidth - 1));

            if (isPowerOfTwo === true) {
                tileSize = hipsTileWidth;
            }
        }
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
        return {
            properties: {
                url: url,
                maxOrder:  parseInt(metadata.hips_order),
                frame: {
                    label: "J2000",
                    system: "J2000"
                },
                tileSize: tileSize,
                format: tileFormat,
                minCutout: parseFloat(cuts[0]),
                maxCutout: parseFloat(cuts[1]),
            },
            meta: {
                color: color,
                blendCfg: {
                    srcColorFactor: 'SrcAlpha',
                    dstColorFactor: 'OneMinusSrcAlpha',
                    func: 'FuncAdd' 
                },
                opacity: 1.0,
            }
        };
    }

    HpxImageSurvey.create = async function(idOrRootUrl, options) {
        if (!idOrRootUrl) {
            return;
        }
    
        let survey = await new HpxImageSurvey(idOrRootUrl);
        return survey;
    };

    return HpxImageSurvey;
})();

