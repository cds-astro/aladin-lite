// Copyright 2023 - UDS/CNRS
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
 * File PlanetaryFeaturesPointer.js
 *
 * The Planetary feature pointer will query a dedicated service for a given position and radius and
 * return information on the nearest planetary feature
 *
 * Author: Thomas Boch [CDS]
 *
 *****************************************************************************/
import { Utils }          from "./Utils.js";
import { AladinUtils } from "./AladinUtils.js";

export let PlanetaryFeaturesPointer = (function() {
    const PlanetaryFeaturesPointer = {};

    PlanetaryFeaturesPointer.MIRRORS = ['https://alasky.cds.unistra.fr/planetary-features/cs']; // list of base URL for planetary features pointer service

    // dict of radius (in meters) for some planetary bodies
    PlanetaryFeaturesPointer.PLANETS_RADIUS = {
        'mercury':      2439400,
        'venus':        6051000,
        'earth':        6378137,
        'mars':         3396190,
        'moon':         1738100,
        'ceres':         473000,
        'titan':        2575000,
        'titania':       788400,
        'dione':         561400,
        'enceladus':     252100,
        'iapetus':       734500,
        'mimas':         198200,
        'rhea':          763800,
        'tethys':        533000,
        'callisto':     2410300,
        'ariel':         578900,
        'charon':        606000,
        'triton':       1353000,
        'pluto':        1188300
    }

    // list of planetary bodies with west increasing longitudes
    PlanetaryFeaturesPointer.HAS_WEST_INCREASING_LONGITUDES = ['amalthea', 'callisto', 'deimos', 'dione', 'enceladus', 'epimetheus', 'eros', 'europa', 'ganymede', 'gaspra', 'hyperion', 'iapetus', 'io', 'janus', 'mathilde', 'mercury', 'mimas', 'phobos', 'phoebe', 'proteus', 'rhea', 'tethys', 'thebe', 'titan'];


    PlanetaryFeaturesPointer.query = function(ra, dec, radiusDegrees, body, aladinInstance) {
        let lon = ra;
        const lat = dec;
        if (PlanetaryFeaturesPointer.HAS_WEST_INCREASING_LONGITUDES.includes(body)) {
            lon = 360 - lon;
        }
        const params = {"lon": lon, "lat": lat, "SR": radiusDegrees, "format": "csv", "body": body};

        function csvToArray(text) {
            let p = '', row = [''], ret = [row], i = 0, r = 0, s = !0, l;
            for (l of text) {
                if ('"' === l) {
                    if (s && l === p) row[i] += l;
                    s = !s;
                } else if (',' === l && s) l = row[++i] = '';
                else if ('\n' === l && s) {
                    if ('\r' === p) row[i] = row[i].slice(0, -1);
                    row = ret[++r] = [l = '']; i = 0;
                } else row[i] += l;
                p = l;
            }
            return ret;
        };

        Utils.loadFromMirrors(PlanetaryFeaturesPointer.MIRRORS, {contentType: "text/plain", data: params})
            .then((response) => response.text())
            .then((result) => {
                aladinInstance.view.setCursor('pointer');

                const lines = result.split('\n');
                const fields = csvToArray(lines[0])[0];

                if (lines.length>1 && lines[1].length>0) {
                    const values = csvToArray(lines[1])[0];


                    const lonFieldIdx = fields.findIndex((element) => element.includes('longitude'));
                    const latFieldIdx = fields.findIndex((element) => element.includes('latitude'));

                    const featureFieldIdx = fields.findIndex((element) => element.includes('feature_name'));
                    const featureName = values[featureFieldIdx];
                    const featureId = values[fields.indexOf('feature_id')]
                    const title = '<div class="aladin-sp-title"><a target="_blank" href="https://planetarynames.wr.usgs.gov/Feature/' + featureId + '">' + featureName + '</a></div>';
                    const featureType = values[fields.indexOf('feature_type')]
                    let content = '<div class="aladin-sp-content">' + ' ' + '</div>';
                    content += '<em>Type: </em>' + featureType + '<br><br>';
                    content += '<a target="_blank" href="https://planetarynames.wr.usgs.gov/Feature/' + featureId + '">More information</a>';

                    let lon = parseFloat(values[lonFieldIdx]);
                    const lat = parseFloat(values[latFieldIdx]);

                    let eastIncreasing = true;
                    const coordinateSystemIdx = fields.indexOf('coordinate_system');
                if (coordinateSystemIdx>0 && values[coordinateSystemIdx].includes("+West")) {
                        eastIncreasing = false;
                    }

                    if (! eastIncreasing) {
                        lon = 360 - lon;
                    }

                    let radiusDeg = undefined;
                    try {
                        const diameter = parseFloat(values[fields.indexOf('diameter')]);
                        if (body in PlanetaryFeaturesPointer.PLANETS_RADIUS) {
                            const parallelLength = 2 * Math.PI * PlanetaryFeaturesPointer.PLANETS_RADIUS[body] * Math.cos(lat * Math.PI /  8180.0);
                            const radiusRadians = 2 * Math.PI * (1000 * diameter / 2) / parallelLength;
                            radiusDeg = 180 * radiusRadians / Math.PI;
                        }
                    } catch(e) {
                        console.error(e);
                    }
                    aladinInstance.showPopup(lon, lat, title, content, radiusDeg);
                }
                else {
                    let no_match_title =  '<div class="aladin-sp-title">Ohoh</div>';
                    let formattedRadiusString = AladinUtils.degreesToString(radiusDegrees);
                    let no_match_content = '<div class="aladin-sp-content">No match was found on <a href="https://planetarynames.wr.usgs.gov">planetarynames.wr.usgs.gov</a> in ' + formattedRadiusString + ' around this point.';
                    no_match_content += '</div>';
                    aladinInstance.showPopup(ra, dec, no_match_title, no_match_content);
                }
            })
            .catch((e) => {
                
                aladinInstance.view.setCursor('pointer');
                aladinInstance.hidePopup();
            })
    };

    return PlanetaryFeaturesPointer;
})();

