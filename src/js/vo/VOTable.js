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
import { Utils } from "./../Utils";

export let VOTable = (function() {

    function VOTable(url, successCallback, errorCallback) {
        fetch(url)
            .then((response) => response.text())
            .then((xml) => {
                ALEvent.AL_USE_WASM.dispatchedTo(document.body, {callback: (wasm) => {
                    let votable = wasm.parseVOTable(xml);

                    successCallback(votable);
                }});
            })
            .catch((e) => errorCallback(e))
    };

    VOTable.parse = function (url, successCallback, errorCallback, useProxy) {
        if(useProxy) {
            url = Utils.handleCORSNotSameOrigin(url);
        }

        fetch(url)
            .then((response) => response.text())
            .then((xml) => {
                ALEvent.AL_USE_WASM.dispatchedTo(document.body, {
                    callback: (wasm) => {
                        let votable = wasm.parseVOTable(xml);
                        votable.votable.get("resources")
                            .forEach((rsc) => successCallback(rsc))
                    }
                })
            })
            .catch((e) => {
                if (errorCallback) {
                    errorCallback(e);
                } else {
                    throw e;
                }
            })
    };

    VOTable.parseTableRsc = function (rsc) {
        let tables = rsc.get("tables")
        if (tables) {
            // take only the first table
            let table = tables[0];
        
            let fields = table.get("elems")
                .filter((elem) => {
                    const elemType = elem["elem_type"] || elem.get("elem_type")
                    return elemType === "Field";
                })
                .map((field) => {
                    // convert a map into a javascript object
                    return Object.fromEntries(field);
                });

            let data = table.get("data");
            if (data) {
                let rows;
                if (data.get("data_type") === "TableData") {
                    rows = data.get("rows");
                } else if(data.get("data_type") === "Binary") {
                    rows = data.get("stream")
                        .get("rows");
                } else {
                    throw 'VOTable has data type not handled:' + data.get("data_type");
                }

                return {fields: fields, rows: rows};
            }
        }
    };

    VOTable.parseSODAServiceRsc = function (rsc) {
        let isSODAService = rsc.get("utype") === "adhoc:service";
        if (isSODAService) {
            let elems = rsc.get("elems");
            let id = rsc.get("ID");

            if (id && id.includes("async")) {
                // First way to check if the resource refers to a async SODA service
                return;
            }

            if (elems) {
                let accessUrl;
                let inputParams = {};
    
                elems.forEach((elem) => {
                    if (elem instanceof Map) {
                        elem = Object.fromEntries(elem.entries());
                    }

                    // SODA access url
                    if (elem["elem_type"] === "Param" && (elem["ucd"] === "meta.ref.url" || elem["name"] === "accessURL")) {
                        accessUrl = elem["value"];
                    } else if (elem["elem_type"] === "Param" && elem["name"] === "standardID") {
                        // Check if it is the sync service
                        // discard the async
                        if (elem["value"].includes("async")) {
                            return;
                        }
                    // Input params group
                    } else if (elem["name"] === "inputParams") {
                        elem["elems"].forEach((inputParam) => {
                            let name = inputParam.get("name");
                            let utype = inputParam.get("utype");
                            let values;
                            switch (name) {
                                case 'ID':
                                    inputParams['ID'] = {
                                        name: 'ID',
                                        type: 'group',
                                        description: inputParam.get("description"),
                                        subInputs: [{
                                            label: "ID",
                                            name: 'ID',
                                            type: "text",
                                            value: inputParam.get("value")
                                        }],
                                    }
                                    break;
                                case 'CIRCLE':
                                    if (inputParam.get("values")) {
                                        values = inputParam.get("values")["max"]["value"].split(" ").map((v) => {return +v;});
                                    }

                                    inputParams['Circle'] = {
                                        name: 'CIRCLE',
                                        type: 'group',
                                        description: inputParam.get("description"),
                                        subInputs: [{
                                            name: 'ra',
                                            label: 'ra[' + utype + ']',
                                            type: 'number',
                                            value: values && values[0],
                                        },
                                        {
                                            name: 'dec',
                                            label: 'dec[' + utype + ']',
                                            type: 'number',
                                            value: values && values[1],
                                        },
                                        {
                                            name: 'rad',
                                            label: 'rad[' + utype + ']',
                                            type: 'number',
                                            value: values && values[2],
                                        }]
                                    };
                                    break;
                                case 'BAND':
                                    if (inputParam.get("values")) {
                                        values = inputParam.get("values")["max"]["value"].split(" ").map((v) => {return +v;});
                                    }

                                    inputParams['Band'] = {
                                        name: 'BAND',
                                        type: 'group',
                                        description: inputParam.get("description"),
                                        subInputs: [{
                                            name: 'fmin',
                                            label: 'fmin[' + utype + ']',
                                            type: 'number',
                                            value: values && values[0],
                                        },
                                        {
                                            name: 'fmax',
                                            label: 'fmax[' + utype + ']',
                                            type: 'number',
                                            value: values && values[1],
                                        }]
                                    };
                                    break;
                                case 'RANGE':
                                    if (inputParam.get("values")) {
                                        values = inputParam.get("values")["max"]["value"].split(" ").map((v) => {return +v;});
                                    }    
                                    inputParams['Range'] = {
                                        name: 'RANGE',
                                        type: 'group',
                                        description: inputParam.get("description"),
                                        subInputs: [{
                                            name: 'ramin',
                                            label: 'ramin[' + utype + ']',
                                            type: 'number',
                                            value: values && values[0],
                                        },
                                        {
                                            name: 'ramax',
                                            label: 'ramax[' + utype + ']',
                                            type: 'number',
                                            value: values && values[1],
                                        },
                                        {
                                            name: 'decmin',
                                            label: 'decmin[' + utype + ']',
                                            type: 'number',
                                            value: values && values[2],
                                        },
                                        {
                                            name: 'decmax',
                                            label: 'decmax[' + utype + ']',
                                            type: 'number',
                                            value: values && values[3],
                                        }]
                                    };
                                default:
                                    break;
                            }
                        })
                    }
                })
    
                return {
                    baseUrl: accessUrl,
                    inputParams: inputParams,
                }
            }
        }
    };

    // return an array of Source(s) from a VOTable url
    // callback function is called each time a TABLE element has been parsed
    return VOTable;
})();
