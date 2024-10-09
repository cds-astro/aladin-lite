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

import { Utils } from "./../Utils";

/*
/// VOTable-rust version ///
export class VOTable {
    constructor(url, successCallback, errorCallback, useProxy) {
        Utils.fetch({url, useProxy})
            .then((resp) => resp.text())
            .then((text) => {
                ALEvent.AL_USE_WASM.dispatchedTo(document.body, {
                    callback: (wasm) => {
                        let votable;
                        try {
                            votable = wasm.parseVOTable(xml);
                        } catch (e) {
                            errorCallback('Catalogue failed to be parsed: ' + e);
                        }
                        
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

    static parseTableRsc(rsc) {
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

    static parseSODAServiceRsc(rsc) {
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
};
*/

/// Pure Vanilla JS version (lighter)
export class VOTable {
    static parser = new DOMParser();
    static textDecoder = new TextDecoder();

    constructor(url, successCallback, errorCallback, useProxy) {
        Utils.fetch({
            url,
            desc: 'Downloading VOTable: ' + url,
            useProxy,
            success: data => {
                try {
                    let xml = VOTable.parser.parseFromString(data, "text/xml")
                    xml.querySelectorAll("RESOURCE").forEach((rsc) => { successCallback(rsc) })
                } catch(e) {
                    if (errorCallback) {
                        errorCallback('Catalogue failed to be parsed: ' + e);
                    } else {
                        throw e;
                    }
                }
            },
            error: e => {
                if (errorCallback) {
                    errorCallback(e);
                } else {
                    throw e;
                }
            }
        })
    };

    static parseRsc(rsc) {
        // Case of a table
        if (rsc.querySelectorAll("TABLE").length > 0) {
            return VOTable._parseTableRsc(rsc)
        }
        
        // Case of SODA service
        let utype = rsc.getAttribute('utype');
        if(utype && utype.includes('service')) {
            return VOTable._parseServiceRsc(rsc)
        }

        // nothing has been parsed
        return null;
    }

    static _parseTableRsc(rsc) {
        var fields = [];
        var k = 0;
        const attributes = ["name", "ID", "ucd", "utype", "unit", "datatype", "arraysize", "width", "precision"];
        rsc.querySelectorAll("FIELD").forEach((field) => {
            var f = {};
            for (var i=0; i<attributes.length; i++) {
                var attribute = attributes[i];
                if (field.hasAttribute(attribute)) {
                    f[attribute] = field.getAttribute(attribute);
                }
            }
            if (!f.ID) {
                f.ID = "col_" + k;
            }

            fields.push(f);
            k++;
        });

        let rows = [];
        if (rsc.querySelectorAll("TABLEDATA").length > 0) {
            rsc.querySelectorAll("TR")
                .forEach((row) => {
                    let r = [];
                    row.querySelectorAll("TD")
                        .forEach((td) => r.push(td.textContent))
                    
                    rows.push(r)
                })
        // This has been adapted from the votable.js given here:
        // https://github.com/aschaaff/votable.js/blob/master/votable.js#L451
        } else if (rsc.querySelectorAll("BINARY").length > 0) {
            let base64ToBytes = function(base64) {
                const binString = atob(base64);
                return Uint8Array.from(binString, (m) => m.codePointAt(0));
            }
            let stream = rsc.querySelector("STREAM").textContent;
            let bytes = base64ToBytes(stream);
            let numDataBytes = bytes.length;

            const tabDataSize = {short: 2, int: 4, float: 4, long: 8, double: 8, unsignedByte: 1};
            const numFields = fields.length;

            const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);

            let fieldIdx = 0;
            let numBytesRead = 0;
            let r = [];
            while (numBytesRead < numDataBytes) {
                let cellSizeInBytes;
                let dataType = fields[fieldIdx].datatype;
                if (dataType === 'char') {
                    // check the arraysize
                    const arraySize = fields[fieldIdx].arraysize;

                    if (arraySize.includes('*')) {
                        // variable size, read 32 bits
                        cellSizeInBytes = view.getUint32(numBytesRead)
                        numBytesRead += 4;

                        if (cellSizeInBytes === 0) {
                            // empty data
                            dataType = 'NULL';
                        }
                    } else {
                        cellSizeInBytes = arraySize;
                    }
                } else {
                    cellSizeInBytes = tabDataSize[dataType];
                }
                let value;
                let prec;
                switch (dataType) {
                    case 'short': 
                        value = view.getUint16(numBytesRead);
                    break;
                    case 'int': 
                        value = view.getInt32(numBytesRead);
                    break;
                    case 'long': 
                        value = view.getBigInt64(numBytesRead);
                    break;
                    case 'float': 
                        value = view.getFloat32(numBytesRead);
                        prec = fields[fieldIdx].precision;
                        if (prec) {
                            value = +value.toFixed(prec); // round (arrondi)
                        }
                    break;
                    case 'double': 
                        value = view.getFloat64(numBytesRead);
                        prec = fields[fieldIdx].precision;
                        if (prec) {
                            value = +value.toFixed(prec); // round (arrondi)
                        }
                    break;
                    case 'unsignedByte':
                        value = view.getUint8(numBytesRead);
                    break;
                    case 'char':
                        value = bytes.slice(numBytesRead, numBytesRead + cellSizeInBytes);
                        value = VOTable.textDecoder.decode(value)
                    break;
                    case 'NULL': // Empty Data
                        value = null;
                    break;
                    default:
                        throw dataType + 'not supported!'
                }

                r.push(value)

                numBytesRead += cellSizeInBytes;
                fieldIdx++;
                
                if (fieldIdx === numFields) {
                    fieldIdx = 0;
                    rows.push(r);
                    r = [];
                }
            }
        }
        
        return {fields, rows};
    };

    static _parseServiceRsc(rsc) {
        // find the baseUrl
        const baseUrl = rsc.querySelectorAll('[name="accessURL"]')[0]
            .getAttribute('value');

        const name = rsc.getAttribute('ID');

        // find the input params
        let inputParams = {};
        rsc.querySelectorAll('[name="inputParams"]')[0]
            .querySelectorAll("PARAM")
            .forEach((param) => {
                let values;
                let min, max;

                const name = param.getAttribute("name");
                const unit = param.getAttribute("unit");
                const description = param.getAttribute("description");

                switch (name) {
                    case 'ID':
                        inputParams['ID'] = {
                            name: 'ID',
                            type: 'group',
                            description,
                            subInputs: [{
                                label: "ID",
                                name: 'ID',
                                type: "text",
                                value: param.getAttribute("value")
                            }],
                        }
                        break;
                    case 'CIRCLE':
                        values = null;
                        max = param.querySelector("VALUES MAX");
                        if (max && max.hasAttribute("value")) {
                            values = max.getAttribute("value")
                                .split(" ").map((v) => {return +v;});
                        }

                        inputParams['Circle'] = {
                            name: 'CIRCLE',
                            type: 'group',
                            description,
                            subInputs: [{
                                name: 'ra',
                                label: 'ra[' + unit + ']',
                                type: 'text',
                                value: values && values[0],
                            },
                            {
                                name: 'dec',
                                label: 'dec[' + unit + ']',
                                type: 'text',
                                value: values && values[1],
                            },
                            {
                                name: 'rad',
                                label: 'rad[' + unit + ']',
                                type: 'text',
                                value: values && values[2],
                            }]
                        };
                        break;
                    case 'BAND':
                        values = [NaN, NaN];
                        min = param.querySelector("VALUES MIN");
                        max = param.querySelector("VALUES MAX");

                        if (min && min.hasAttribute('value')) {
                            values[0] = +min.getAttribute("value");
                        }

                        if (max && max.hasAttribute('value')) {
                            values[1] = +max.getAttribute("value");
                        }

                        inputParams['Band'] = {
                            name: 'BAND',
                            type: 'group',
                            description,
                            subInputs: [{
                                name: 'fmin',
                                label: 'fmin[' + unit + ']',
                                type: 'number',
                                value: values && values[0],
                            },
                            {
                                name: 'fmax',
                                label: 'fmax[' + unit + ']',
                                type: 'number',
                                value: values && values[1],
                            }]
                        };
                        break;
                    case 'RANGE':
                        values = null;

                        max = param.querySelector("VALUES MAX");
                        if (max && max.hasAttribute("value")) {
                            values = max.getAttribute("value")
                                .split(" ").map((v) => {return +v;});
                        }

                        inputParams['Range'] = {
                            name: 'RANGE',
                            type: 'group',
                            description,
                            subInputs: [{
                                name: 'ramin',
                                label: 'ramin[' + unit + ']',
                                type: 'number',
                                value: values && values[0],
                            },
                            {
                                name: 'ramax',
                                label: 'ramax[' + unit + ']',
                                type: 'number',
                                value: values && values[1],
                            },
                            {
                                name: 'decmin',
                                label: 'decmin[' + unit + ']',
                                type: 'number',
                                value: values && values[2],
                            },
                            {
                                name: 'decmax',
                                label: 'decmax[' + unit + ']',
                                type: 'number',
                                value: values && values[3],
                            }]
                        };
                    default:
                        break;
                }
            })

            return { name, baseUrl, inputParams}
    };
};
