/******************************************************************************
 * Aladin HTML5 project
 * 
 * File Sesame.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Sesame = (function() {
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
            success: function(data) {
                if (data.Target && data.Target.Resolver && data.Target.Resolver) {
                    callbackFunctionSuccess(data);
                }
                else {
                    callbackFunctionError(data);
                }
            },
            error: callbackFunctionError
            });
    };
    
    return Sesame;
})();

