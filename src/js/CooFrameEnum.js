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
 * File CooFrameEnum
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

export let CooFrameEnum = (function() {

    // Corresponds to the Rust CooSystem enum possibilities.
    var systems = {ICRS: 'ICRS', GAL: 'GAL'};
    return {
        SYSTEMS: systems,

        ICRS: {label: "ICRS", system: systems.ICRS, explain: "International Celestial Reference System"},
        ICRSd: {label: "ICRSd", system: systems.ICRS, explain: "International Celestial Reference System in decimals"},
        GAL:  {label: "GAL", system: systems.GAL, explain: "Galactical"},

        fromString: function(str, defaultValue) {
            if (! str) {
                return defaultValue ? defaultValue : null;
            }
            
            str = str.toLowerCase().replace(/^\s+|\s+$/g, ''); // convert to lowercase and trim
            
            if (str.indexOf('j2000d')==0 || str.indexOf('icrsd')==0) {
                return CooFrameEnum.ICRSd;
            }
            else if (str.indexOf('j2000')==0 || str.indexOf('icrs')==0 || str.indexOf('equatorial')==0) {
                return CooFrameEnum.ICRS;
            }
            else if (str.indexOf('gal')==0) {
                return CooFrameEnum.GAL;
            }
            else {
                return defaultValue ? defaultValue : null;
            }
        }
    };
    
})();





