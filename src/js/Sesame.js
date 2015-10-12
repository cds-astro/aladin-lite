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
 * File Sesame.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/

Sesame = (function() {
    Sesame = {};
    
    Sesame.cache = {};

    /** find RA, DEC for any target (object name or position)
     *  if successful, callback is called with an object {ra: <ra-value>, dec: <dec-value>}
     *  if not successful, errorCallback is called
     */
    Sesame.getTargetRADec = function(target, callback, errorCallback) {
        if (!callback) {
            return;
        }
        var isObjectName = /[a-zA-Z]/.test(target);

        // try to parse as a position
        if ( ! isObjectName) {
            var coo = new Coo();

            coo.parse(target);
            if (callback) {
                callback({ra: coo.lon, dec: coo.lat});
            }
        }
        // ask resolution by Sesame
        else {
            Sesame.resolve(target,

                           function(data) { // success callback
                               callback({ra:  data.Target.Resolver.jradeg,
                                         dec: data.Target.Resolver.jdedeg});
                           },

                           function(data) { // error callback
                               if (errorCallback) {
                                   errorCallback();
                               }
                           }
                           );
        }
    };
    
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

