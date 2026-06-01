// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
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

    HiPSCache.prototype.update = function (key, obj) {
        this.cache[key] = obj;

        ALEvent.HIPS_CACHE_UPDATED.dispatchedTo(document.body);
    };

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.prototype.append = function (key, obj) {
        this.cache[key] = obj;

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
        let obj = this.cache[key];

        return obj;
    };

    /*
    * key can be a CDS ID or an url. TODO could be an options.name too.
    */
    HiPSCache.prototype.contains = function (key) {
        return this.cache[key] !== undefined && this.cache[key] !== null;
    };

    return HiPSCache;
})();
