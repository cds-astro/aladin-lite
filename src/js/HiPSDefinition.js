// Copyright 2013-2017 - UDS/CNRS
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
 * File HiPSDefinition
 * 
 * Author: Thomas Boch [CDS]
 * 
 *****************************************************************************/

import { Utils } from "./Utils.js";
import $ from 'jquery';
import { MocServer } from "./MocServer.js";

export let HiPSDefinition = (function() {

    // constructor
    function HiPSDefinition(properties) {
        this.properties = properties; // key-value object corresponding to the properties file

        this.id = this.getID();
        this.obsTitle = properties['obs_title'];
        this.frame = properties['hips_frame'];
        this.order = parseInt(properties['hips_order']);
        this.clientSortKey = properties['client_sort_key'];
        this.tileFormats = properties.hasOwnProperty('hips_tile_format') && properties['hips_tile_format'].split(' ');
        this.urls = [];
        this.urls.push(properties['hips_service_url']);
        var k = 1;
        while (properties.hasOwnProperty('hips_service_url_' + k)) {
            this.urls.push(properties['hips_service_url_' + k]);
            k++;
        }

        this.clientApplications = properties['client_application'];
    };

    HiPSDefinition.prototype = {
        getServiceURLs: function(httpsOnly) {
            httpsOnly = httpsOnly === true;

            // TODO: TO BE COMPLETED
        },

        // return the ID according to the properties
        getID: function() {
            // ID is explicitely given
            if (this.properties.hasOwnProperty('ID')) {
                return this.properties['ID'];
            }

            var id = null;
            // ID might be built from different fields
            if (this.properties.hasOwnProperty('creator_did')) {
                id = this.properties['creator_did'];
            }
            if (id==null && this.properties.hasOwnProperty('publisher_did')) {
                id = this.properties['publisher_did'];
            }

            if (id != null) {
                // remove ivo:// prefix
                if (id.slice(0, 6) === 'ivo://') {
                    id = id.slice(6);
                }

                // '?' are replaced by '/'
                id = id.replace(/\?/g, '/')
            }

            return id;
        }
    };

    // parse a HiPS properties and return a dict-like object with corresponding key-values
    // return null if parsing failed
    HiPSDefinition.parseHiPSProperties = function(propertiesStr) {
        if (propertiesStr==null) {
            return null;
        }

        var propertiesDict = {};
        // remove CR characters
        propertiesStr = propertiesStr.replace(/[\r]/g, '');
        // split on LF
        var lines = propertiesStr.split('\n');
        for (var k=0; k<lines.length; k++)  {
            var l = $.trim(lines[k]);
            // ignore comments lines
            if (l.slice(0, 1)==='#') {
                continue;
            }
            var idx = l.indexOf('=');
            if (idx<0) {
                continue;
            }
            var key = $.trim(l.slice(0, idx));
            var value = $.trim(l.slice(idx+1));

            propertiesDict[key] = value;
        }

        return propertiesDict;
    };


    // find a HiPSDefinition by id.
    // look first locally, and remotely only if local search was unsuccessful
    //
    // call callback function with a list of HiPSDefinition candidates, empty array if nothing found

    // Create a HiPSDefinition object from a URL
    //
    // If the URL ends with 'properties', it is assumed to be the URL of the properties file
    // else, it is assumed to be the base URL of the HiPS
    //
    // return a HiPSDefinition if successful, null if it failed
    HiPSDefinition.fromURL = function(url, callback) {
        var hipsUrl, propertiesUrl;
        if (url.slice(-10) === 'properties') {
            propertiesUrl = url;
            hipsUrl = propertiesUrl.slice(0, -11);
        }
        else {
            if (url.slice(-1) === '/') {
                url = url.slice(0, -1);
            }
            hipsUrl = url;
            propertiesUrl = hipsUrl + '/properties';
        }

        var callbackWhenPropertiesLoaded = function(properties) {
            // Sometimes, hips_service_url is missing. That can happen for instance Hipsgen does not set the hips_service_url keyword
            // --> in that case, we add as an attribyte the URL that was given as input parameter
            var hipsPropertiesDict = HiPSDefinition.parseHiPSProperties(properties);
            if (! hipsPropertiesDict.hasOwnProperty('hips_service_url')) {
                hipsPropertiesDict['hips_service_url'] = hipsUrl;
            }
            (typeof callback === 'function') && callback(new HiPSDefinition(hipsPropertiesDict));
        };

        // try first without proxy
        var ajax = Utils.getAjaxObject(propertiesUrl, 'GET', 'text', false);
        ajax
            .done(function(data) {
                callbackWhenPropertiesLoaded(data);
            })
            .fail(function() {
                // if not working, try with the proxy
                var ajax = Utils.getAjaxObject(propertiesUrl, 'GET', 'text', true);
                ajax
                    .done(function(data) {
                        callbackWhenPropertiesLoaded(data);
                    })
                    .fail(function() {
                        (typeof callback === 'function') && callback(null);
                    })
            });
    };

    // HiPSDefinition generation from a properties dict-like object
    HiPSDefinition.fromProperties = function(properties) {
        return new HiPSDefinition(properties);
    };

    return HiPSDefinition;

})();

