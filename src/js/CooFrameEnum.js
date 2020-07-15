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
 
CooFrameEnum = (function() {

    var systems = {J2000: 'J2000', GAL: 'Galactic'};
    return {
        SYSTEMS: systems,

        J2000: {label: "J2000", system: systems.J2000},
        J2000d: {label: "J2000d", system: systems.J2000},
        GAL:  {label: "Galactic", system: systems.GAL}
    };
 
})();



CooFrameEnum.fromString = function(str, defaultValue) {
    if (! str) {
        return defaultValue ? defaultValue : null;
    }
    
    str = str.toLowerCase().replace(/^\s+|\s+$/g, ''); // convert to lowercase and trim
    
    if (str.indexOf('j2000d')==0 || str.indexOf('icrsd')==0) {
        return CooFrameEnum.J2000d;
    }
    else if (str.indexOf('j2000')==0 || str.indexOf('icrs')==0) {
        return CooFrameEnum.J2000;
    }
    else if (str.indexOf('gal')==0) {
        return CooFrameEnum.GAL;
    }
    else {
        return defaultValue ? defaultValue : null;
    }
};

