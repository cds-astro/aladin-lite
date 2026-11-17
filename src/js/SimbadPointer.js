// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
//


/******************************************************************************
 * Aladin Lite project
 *
 * File SimbadPointer.js
 *
 * The SIMBAD pointer will query Simbad for a given position and radius and
 * return information on the object with
 *
 * Author: Thomas Boch [CDS]
 *
 *****************************************************************************/
import { Coo }            from "./libs/astro/coo.js";
import { Utils }          from "./Utils";
import { AladinUtils } from "./AladinUtils.js";

export let SimbadPointer = (function() {
    const SimbadPointer = {};

    SimbadPointer.MIRRORS = ['https://alasky.cds.unistra.fr/cgi/simbad-flat/simbad-quick.py', 'https://alaskybis.cds.unistra.fr/cgi/simbad-flat/simbad-quick.py']; // list of base URL for Simbad pointer service
    SimbadPointer.TAP_MIRRORS = ['https://simbad.idia.ac.za/simbad/sim-tap', 'https://simbad.cfa.harvard.edu/simbad/sim-tap', 'https://simbad.cds.unistra.fr/simbad/sim-tap']; // list of base URL for Simbad TAP service


    SimbadPointer.query = function(ra, dec, radiusDegrees, aladinInstance) {
        const adqlQuery = "SELECT TOP 1 basic.main_id, basic.ra, basic.dec, basic.nbref, basic.otype, allfluxes.B FROM basic LEFT OUTER JOIN allfluxes ON basic.oid = allfluxes.oidref WHERE CONTAINS(POINT('ICRS', basic.ra, basic.dec), CIRCLE('ICRS', " + ra + ", " + dec + ", " + radiusDegrees + ")) = 1 ORDER BY nbref DESC";
        const params = {REQUEST: 'doQuery', LANG: 'ADQL', FORMAT: 'json', QUERY: adqlQuery};

        const urlsList = SimbadPointer.TAP_MIRRORS.map(url => url + '/sync');
        Utils.fetchFromMirrorUrls(urlsList, {data: params, dataType: 'json', timeout: 10000})
            .then((result) => {
                if (result.data && result.data.length==1) {
                    console.log("toto")
                    const objName = result.data[0][0];
                    const raObj   = result.data[0][1];
                    const decObj  = result.data[0][2];
                    const nbRef   = result.data[0][3];
                    const otype   = result.data[0][4];
                    const magB    = result.data[0][5];
                    const title = '<div class="aladin-sp-title"><a target="_blank" href="https://simbad.idia.ac.za/simbad/sim-id?Ident=' + encodeURIComponent(objName) + '&output.format=html">' + objName + '</a></div>';
                    let content = '<div class="aladin-sp-content">';
                    content += '<em>Type: </em>' + otype + '<br>';
                    if (Utils.isNumber(magB)) {
                        content += '<em>Mag: </em>' + magB + '<br>';
                    }
                    //content += '<br><a target="_blank" href="http://cdsportal.cds.unistra.fr/?target=' + encodeURIComponent(objName) + '">Query in CDS portal</a>';
                    content += '</div>';

                    aladinInstance.showPopup(raObj, decObj, title, content);
                }
                else {
                    let noMatchTitle = '<div class="aladin-sp-title">No object found</div>';
                    let formattedRadiusString = AladinUtils.degreesToString(radiusDegrees);
                    let noMatchContent = '<div class="aladin-sp-content">No match was found on <a href="https://simbad.idia.ac.za/simbad">Simbad</a> in ' + formattedRadiusString + ' around this point.</div>';
                    aladinInstance.showPopup(ra, dec, noMatchTitle, noMatchContent);
                }
            })
            .catch((e) => {
                aladinInstance.hidePopup();
            })
    };


    // This method is kept as to be able to compare with TAP implementation, once CDS services @ Strasbourg are up again
    SimbadPointer.queryOld = function(ra, dec, radiusDegrees, aladinInstance) {
        var coo = new Coo(ra, dec, 7);
        var params = {"Ident": coo.format('s/'), "SR": radiusDegrees};

        Utils.fetchFromMirrorUrls(SimbadPointer.MIRRORS, {contentType: "text/plain", data: params, dataType: 'text'})
            .then((result) => {

                var regexp = /(.*?)\/(.*?)\((.*?),(.*?)\)/g;
                var match = regexp.exec(result);
                if (match) {
                    var objCoo = new Coo();
                    objCoo.parse(match[1]);
                    var objName = match[2];
                    var title = '<div class="aladin-sp-title"><a target="_blank" href="https://simbad.cds.unistra.fr/simbad/sim-id?Ident=' + encodeURIComponent(objName) + '">' + objName + '</a></div>';
                    var content = '<div class="aladin-sp-content">';
                    content += '<em>Type: </em>' + match[4] + '<br>';
                    var magnitude = match[3];
                    if (Utils.isNumber(magnitude)) {
                        content += '<em>Mag: </em>' + magnitude + '<br>';
                    }
                    content += '<br><a target="_blank" href="http://cdsportal.cds.unistra.fr/?target=' + encodeURIComponent(objName) + '">Query in CDS portal</a>';
                    content += '</div>';

                    aladinInstance.showPopup(objCoo.lon, objCoo.lat, title, content);
                }
                else {
                    let noMatchTitle = '<div class="aladin-sp-title">Ohoh</div>';
                    let formattedRadiusString = AladinUtils.degreesToString(radiusDegrees);
                    let noMatchContent = '<div class="aladin-sp-content">No match was found on <a href="https://simbad.cds.unistra.fr/simbad">Simbad</a> in ' + formattedRadiusString + ' around this point.</div>';
                    aladinInstance.showPopup(coo.lon, coo.lat, noMatchTitle, noMatchContent);
                }
            })
            .catch((e) => {
                aladinInstance.hidePopup();
            })
    };

    return SimbadPointer;
})();

