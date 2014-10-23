/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Sesame.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Sesame = {};

Sesame.cache = {};

Sesame.resolve = function(objectName, callbackFunctionSuccess, callbackFunctionError) {
    //var sesameUrl = "http://cdsportal.u-strasbg.fr/services/sesame?format=json";
    var sesameUrl = "http://cds.u-strasbg.fr/cgi-bin/nph-sesame.jsonp?";
    $.ajax({
        url: sesameUrl ,
        data: {"object": objectName},
        method: 'GET',
        dataType: 'jsonp',
        success: callbackFunctionSuccess,
        error: callbackFunctionError
        });
};
