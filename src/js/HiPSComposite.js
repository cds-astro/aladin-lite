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

export let HiPSComposite = (function () {
    /**
     * The object describing an image survey
     *
     * @class
     * @constructs HiPSComposite
     *
     * @param {string} id - Mandatory unique identifier for the layer. Can be an arbitrary name
     * @param {string|FileList|HiPSLocalFiles} url - Can be:
     * <ul>
     * <li>An http url towards a HiPS.</li>
     * <li>A relative path to your HiPS</li>
     * <li>A special ID pointing towards a HiPS. One can found the list of IDs {@link https://aladin.cds.unistra.fr/hips/list| here}</li>
     * <li>A dict storing a local HiPS files. This object contains a tile file: hips[order][ipix] = File and refers to the properties file like so: hips["properties"] = File. </li>
     *     A javascript {@link FileList} pointing to the opened webkit directory is also accepted.
     * </ul>
     * @param {HiPSOptions} [options] - The option for the survey
     *
     * @description Giving a CDS ID will do a query to the MOCServer first to retrieve metadata. Then it will also check for the presence of faster HiPS nodes to choose a faster url to query to tiles from.
     */
    function HiPSComposite(hipses, options) {
        let name = (options && options.name) || hipses[0].name + "_composite";

    };

    return HiPSComposite;
})();
