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
 * File HiPSComposite.js
 *
 * Authors: Matthieu Baumann [CDS]
 *
 *****************************************************************************/
import { HiPS } from "./HiPS.js";
import A from "./A.js";

export let HiPSComposite = (function () {
    /**
     * The object describing the color composition of image surveys
     *
     * @class
     * @constructs HiPSComposite
     *
     * @param {HiPSOptions[]} [options] - The option for the survey
     */
    function HiPSComposite(options) {
        this.name = options && options.name || 'Composite HiPS';
        this.hipses = []
    };

    HiPSComposite.prototype.setOptions = function(options) {
        this.hipses = [];
        for (var hipsOptions of options) {
            let hips = A.HiPS(hipsOptions.id, hipsOptions)
            this.hipses.push(hips)
        }
    };

    HiPSComposite.prototype._setView = HiPS.prototype._setView;

    HiPSComposite.prototype._addToView = function (layer) {
        this._removeFromView();

        this.layer = layer;

        let i = 0;
        for (var hips of this.hipses) {
            hips._addToView(layer + '_' + i)
            i++;
        }

        return this
    };

    HiPSComposite.prototype._removeFromView = function() {
        for (var hips of this.hipses) {
            hips._removeFromView()
        }
    }

    HiPSComposite.prototype.getOpacity = function() {
        return this.hipses[0].getOpacity()
    }

    HiPSComposite.prototype.setOpacity = function(opacity) {
        for (var hips of this.hipses) {
            hips.setOpacity(opacity)
        }
    }

    return HiPSComposite;
})();
