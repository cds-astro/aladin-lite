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
import { Catalog } from "../Catalog.js";
import { ObsCore } from "./ObsCore.js";
import { Utils } from "./../Utils.js";

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

    VOTable.parse = function (url, callback, raField, decField) {
        url = Utils.handleCORSNotSameOrigin(url);

        fetch(url)
            .then((response) => response.text())
            .then((xml) => {
                ALEvent.AL_USE_WASM.dispatchedTo(document.body, {callback: (wasm) => {
                    let votable = wasm.parseVOTable(xml);
                    votable.votable.get("resources")
                        .forEach((resource) => {
                            let tables = resource.get("tables")
                            if (tables) {
                                tables.forEach((table) => {
                                    let fields = table.get("elems")
                                        .filter((elem) => {
                                            const elemType = elem["elem_type"] || elem.get("elem_type")
                                            return elemType === "Field";
                                        })
                                        .map((field) => {
                                            // convert a map into a javascript object
                                            return Object.fromEntries(field);
                                        })

                                    try {
                                        fields = ObsCore.parseFields(fields);

                                        fields.subtype = "ObsCore";
                                    } catch(e) {
                                        // It is not an ObsCore table
                                        fields = Catalog.parseFields(fields, raField, decField);
                                    }
    
                                    let data = table.get("data");
                                    if (data) {
                                        let rows = data.get("rows");
    
                                        if (rows) {
                                            callback(fields, rows)
                                        }
                                    }
                                })
                            }
                        })
                    }
                })
            })
    };

    // return an array of Source(s) from a VOTable url
    // callback function is called each time a TABLE element has been parsed
    return VOTable;
})();
