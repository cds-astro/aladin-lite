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

import { Utils } from "./Utils";

export let Sesame = (function() {
    let Sesame = {};
    
    Sesame.cache = {};

    Sesame.lastObjectNameResolved = undefined;

    //Sesame.SESAME_URL = "http://cds.unistra.fr/cgi-bin/nph-sesame.jsonp";
    Sesame.SESAME_URL = "https://cds.unistra.fr/cgi-bin/nph-sesame/-o/SVNI?";

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
                    callback({
                        ra:  data.coo.jradeg,
                        dec: data.coo.jdedeg
                    });
                },

                function(data) { // error callback
                    if (errorCallback) {
                        errorCallback(data);
                    }
                }
           );
        }
    };
    
    Sesame.resolve = function(objectName, callbackFunctionSuccess, callbackFunctionError, useCache = true) {
        let self = this;
        // check the cache first
        if (useCache && this.cache[objectName]) {
            let data = this.cache[objectName];
            callbackFunctionSuccess(data)
            return;
        }

        var sesameUrl = Sesame.SESAME_URL;

        Sesame.lastObjectNameResolved = objectName;
        Utils.fetch({
            desc: "Resolving name of: " + objectName,
            url: sesameUrl + objectName,
            dataType: 'text',
            useProxy: false,
            success: function(text) {
                if (Sesame.lastObjectNameResolved !== objectName) {
                    return;
                }
                try {
                    let coo;
                    // Find the coo
                    if (text.includes('%J ')) {
                        let pos = text.slice(text.indexOf('%J ') + 3);
                        pos = pos.split(' ')
                        coo = {
                            jradeg: +pos[0],
                            jdedeg: +pos[1]
                        }
                    } else {
                        throw 'coo not found';
                    }

                    const data = {coo}

                    // Cache the result
                    // check if there is no IMCCE in there
                    // i.e. planets are moving faster
                    // better not cache that
                    if (!text.includes('IMCCE')) {
                        self.cache[objectName] = data; 
                    }
                    
                    callbackFunctionSuccess(data);
                } catch(e) {
                    callbackFunctionError('Error resolving object: '  + objectName + '\nReason: ' + e);
                }
            },
            error: callbackFunctionError
        });
    };
    
    return Sesame;
})();

