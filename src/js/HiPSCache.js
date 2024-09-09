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
 * File HiPS
 *
 * Authors: Thomas Boch & Matthieu Baumann [CDS]
 *
 *****************************************************************************/
import { ALEvent } from "./events/ALEvent.js";

export let HiPSCache = (function () {

    let HiPSCache = function() {
        this.cache = {}
    };

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.prototype.append = function (key, image) {
        this.cache[key] = image;

        ALEvent.HIPS_CACHE_UPDATED.dispatchedTo(document.body);
    };

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.prototype.delete = function (key) {
        delete this.cache[key];

        ALEvent.HIPS_CACHE_UPDATED.dispatchedTo(document.body);
    };

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.prototype.get = function (key) {
        return this.cache[key];
    };

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.prototype.contains = function (key) {
        return this.cache[key] !== undefined && this.cache[key] !== null;
    };

    return HiPSCache;
})();
