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
 * File PlanetaryFeaturesNameResolver.js
 *
 * Author: Thomas Boch[CDS]
 *
 *****************************************************************************/

import { Utils } from "./Utils.js";

import $ from 'jquery';

export let PlanetaryFeaturesNameResolver = (function() {
    let PlanetaryFeaturesNameResolver = {};

    PlanetaryFeaturesNameResolver.cache = {};

    PlanetaryFeaturesNameResolver.URL = 'https://alasky.cds.unistra.fr/planetary-features/resolve';

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


    PlanetaryFeaturesNameResolver.resolve = function(featureName, body, callbackFunctionSuccess, callbackFunctionError) {
        const url = PlanetaryFeaturesNameResolver.URL;

        $.ajax({
            url: url ,
            data: {"identifier": featureName, 'body': body, 'threshold': 0.7, 'format': 'csv'},
            method: 'GET',
            dataType: 'text',
            success: function(result) {
                const lines = result.split('\n');
                const fields = csvToArray(lines[0])[0];
                if (lines.length>1 && lines[1].length>0) {
                    const values = csvToArray(lines[1])[0];
                    const lonFieldIdx = fields.findIndex((element) => element. includes('longitude'));
                    const latFieldIdx = fields.findIndex((element) => element. includes('latitude'));
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

                    callbackFunctionSuccess({lon: lon, lat: lat});
                }
                else {
                    callbackFunctionError(data);
                }
            },
            error: callbackFunctionError
            });
    };

    return PlanetaryFeaturesNameResolver;
})();

