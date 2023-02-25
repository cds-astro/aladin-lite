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
 * File ImageSurvey
 * 
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 * 
 *****************************************************************************/
import { Utils } from "./Utils.js";
import { HiPSDefinition } from "./HiPSDefinition.js";
import { MocServer } from "./MocServer.js";

export let HiPSProperties = {};

HiPSProperties.fetch = async function(urlOrId) {
    try {
        urlOrId = new URL(urlOrId);
    } catch (e) {}

    let result = {};
    if (!(urlOrId instanceof URL)) {
        // Use the MOCServer to retrieve the
        // properties
        const ID = urlOrId;
        const params = {
            get: "record",
            fmt: "json",
            ID: "*" + ID + "*",
        };

        let metadata = await Utils.loadFromMirrors(MocServer.MIRRORS_HTTPS, {
            data: params,
        }).then(response => response.json());

        // We get the property here
        // 1. Ensure there is exactly one survey matching
        if (!metadata || metadata.length == 0) {
            throw 'No surveys matching have been found for the id: ' + ID;
        } else {
            if (metadata.length > 1) {
                let matching = metadata.find((m) => m.ID === ID);

                if (!matching) {
                    result = metadata[0];
                    console.warn("Multiple surveys are matching, please choose one. The chosen one is: " + result);
                }
            } else {
                // Exactly one matching
                result = metadata[0];
            }
        }
    } else {
        // Fetch the properties of the survey
        const HiPSServiceUrl = urlOrId.toString();
        
        let url = HiPSServiceUrl;
        // Use the url for retrieving the HiPS properties
        // remove final slash
        if (url.slice(-1) === '/') {
            url = url.substr(0, url.length - 1);
        }
        url = url + '/properties';

        // make URL absolute
        url = Utils.getAbsoluteURL(url);
        // fix for HTTPS support --> will work for all HiPS served by CDS
        url = Utils.fixURLForHTTPS(url)

        let init = {};
        if (Utils.requestCORSIfNotSameOrigin(url)) {
            init = { mode: 'cors' };
        }

        result = await fetch(url, init)
            .then((response) => response.text())
            .then((response) => {
                // We get the property here
                let metadata = HiPSDefinition.parseHiPSProperties(response);

                // 1. Ensure there is exactly one survey matching
                if (metadata) {
                    // Set the service url if not found
                    metadata.hips_service_url = HiPSServiceUrl;
                } else {
                    throw 'No surveys matching at this url: ' + rootURL;
                }

                return metadata;
            });
    }

    return result;
}

HiPSProperties.getFasterMirrorUrl = function (metadata) {
    const pingHiPSServiceUrl = (hipsServiceUrl) => {
        hipsServiceUrl = Utils.fixURLForHTTPS(hipsServiceUrl);

        const controller = new AbortController()

        let startRequestTime = Date.now();
        const maxTime = 2000;
        // 5 second timeout:
        const timeoutId = setTimeout(() => controller.abort(), maxTime)
        const promise = fetch(hipsServiceUrl + '/properties', { cache: 'no-store', signal: controller.signal, mode: "cors" }).then(response => {
            const duration = Date.now() - startRequestTime;//the time needed to do the request
            // completed request before timeout fired
            clearTimeout(timeoutId)
            // Resolve with the time duration of the request
            return { duration: duration, baseUrl: hipsServiceUrl, validRequest: true };
        }).catch((e) => {
            return { duration: maxTime, baseUrl: hipsServiceUrl, validRequest: false };
        });

        return promise;
    };

    // Get all the possible hips_service_url urls
    let promises = [];
    promises.push(pingHiPSServiceUrl(metadata.hips_service_url));

    let numHiPSServiceURL = 1;
    while (metadata.hasOwnProperty("hips_service_url_" + numHiPSServiceURL.toString())) {
        const key = "hips_service_url_" + numHiPSServiceURL.toString();

        let curUrl = metadata[key];
        promises.push(pingHiPSServiceUrl(curUrl))
        numHiPSServiceURL += 1;
    }

    return Promise.all(promises)
        .then((responses) => {
            // filter the ones that failed to not choose them
            // it may be a cors issue at this point
            let validResponses = responses.filter((resp) => { return resp.validRequest === true; });

            const getRandomIntInclusive = function (min, max) {
                min = Math.ceil(min);
                max = Math.floor(max);
                return Math.floor(Math.random() * (max - min + 1)) + min;
            };

            validResponses.sort((r1, r2) => {
                return r1.duration - r2.duration;
            });

            if (validResponses.length >= 2) {
                const isSecondUrlOk = ((validResponses[1].duration - validResponses[0].duration) / validResponses[0].duration) < 0.20;

                if (isSecondUrlOk) {
                    return validResponses[getRandomIntInclusive(0, 1)].baseUrl;
                } else {
                    return validResponses[0].baseUrl;
                }
            } else {
                return validResponses[0].baseUrl;
            }
        })
        .then((url) => Utils.fixURLForHTTPS(url));
}