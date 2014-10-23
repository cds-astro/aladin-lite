/******************************************************************************
 * Aladin HTML5 project
 * 
 * File CooFrameEnum
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
 
CooFrameEnum = (function() {

    return {
        J2000: "J2000",
        GAL:  "Galactic"
    };
 
})();

// TODO : utiliser cette fonction partout où on reçoit une string frame en entrée
CooFrameEnum.fromString = function(str, defaultValue) {
    if (! str) {
        return defaultValue ? defaultValue : null;
    }
    
    str = str.toLowerCase().replace(/^\s+|\s+$/g, ''); // convert to lowercase and trim
    
    if (str.indexOf('j2000')==0 || str.indexOf('icrs')==0) {
        return CooFrameEnum.J2000;
    }
    else if (str.indexOf('gal')==0) {
        return CooFrameEnum.GAL;
    }
    else {
        return defaultValue ? defaultValue : null;
    }
};