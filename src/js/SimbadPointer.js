// Copyright 2018 - UDS/CNRS
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
 * File SimbadPointer.js
 *
 * The SIMBAD pointer will query Simbad for a given position and radius and
 * return information on the object with 
 *  
 * Author: Thomas Boch [CDS]
 * 
 *****************************************************************************/

SimbadPointer = (function() {
    
    
    SimbadPointer = {};

    SimbadPointer.MIRRORS = ['https://alasky.u-strasbg.fr/cgi/simbad-flat/simbad-quick.py', 'https://alaskybis.u-strasbg.fr/cgi/simbad-flat/simbad-quick.py']; // list of base URL for Simbad pointer service

    
    SimbadPointer.query = function(ra, dec, radiusDegrees, aladinInstance) {
        var coo = new Coo(ra, dec, 7);
        var params = {Ident: coo.format('s/'), SR: radiusDegrees}
        var successCallback = function(result) {
            aladinInstance.view.setCursor('pointer');

            var regexp = /(.*?)\/(.*?)\((.*?),(.*?)\)/g;
            var match = regexp.exec(result);
            if (match) {
                var objCoo = new Coo();
                objCoo.parse(match[1]);
                var objName = match[2];
                var title = '<div class="aladin-sp-title"><a target="_blank" href="http://simbad.u-strasbg.fr/simbad/sim-id?Ident=' + encodeURIComponent(objName) + '">' + objName + '</a></div>';
                var content = '<div class="aladin-sp-content">';
                content += '<em>Type: </em>' + match[4] + '<br>';
                var magnitude = match[3];
                if (Utils.isNumber(magnitude)) {
                    content += '<em>Mag: </em>' + magnitude + '<br>';
                }
                content += '<br><a target="_blank" href="http://cdsportal.u-strasbg.fr/?target=' + encodeURIComponent(objName) + '">Query in CDS portal</a>';
                content += '</div>';
                aladinInstance.showPopup(objCoo.lon, objCoo.lat, title, content);
            }
            else {
                aladinInstance.hidePopup();
            }
        };
        var failureCallback = function() {
            aladinInstance.view.setCursor('pointer');
            aladinInstance.hidePopup();
        };
        Utils.loadFromMirrors(SimbadPointer.MIRRORS, {data: params, onSuccess: successCallback, onFailure: failureCallback, timeout: 5});

    };

    return SimbadPointer;
})();
    
