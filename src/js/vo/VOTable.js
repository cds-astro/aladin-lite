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
 * File Catalog
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
import { ALEvent } from "../events/ALEvent.js";

export let VOTable = (function() {

   function VOTable(url, callback) {
        fetch(url)
            .then((response) => response.text())
            .then((xml) => {
                ALEvent.AL_USE_WASM.dispatchedTo(document.body, {callback: (wasm) => {
                    let votable = wasm.parseVOTable(xml);
                    callback(votable);
                }});
            })
    };
    // return an array of Source(s) from a VOTable url
    // callback function is called each time a TABLE element has been parsed

    return VOTable;
})();
