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
 * File Datalink
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

import { VOTable } from "./VOTable.js";
import { Utils } from './../Utils';
import { ActionButton } from "../gui/widgets/ActionButton.js";
import { Catalog } from "../Catalog.js";

export let Datalink = (function() {

    let Datalink = function () {
        this.SODAServerParams = undefined;
        this.sodaQueryWindow = undefined;
    };

    Datalink.prototype.handleActions = function(obscoreRow, aladinInstance) {
        const url = obscoreRow["access_url"];
        VOTable.parse(
            url,
            (rsc) => {
                let table = VOTable.parseTableRsc(rsc);

                if (table && table.fields && table.rows) {
                    table.fields = Catalog.parseFields(table.fields);

                    // Get the fields and the rows
                    let measures = [];
                    const { fields, rows } = table;
                    rows.forEach(row => {        
                        let data = {};

                        for (const [_, field] of Object.entries(fields)) {
                            var key = field.name;
                            data[key] = row[field.idx];
                        }

                        measures.push({data: data})
                    })
                    let self = this;
                    let datalinkTable = {
                        'name': 'Datalink:' + url,
                        'color': 'purple',
                        'rows': measures,
                        'fields': fields,
                        'showCallback': {
                            'service_def': (data) => {
                                const service = data['service_def'];

                                if (data['semantics'] === "#cutout") {
                                    return ActionButton.createIconBtn({
                                        content: '📡',
                                        cssStyle: {
                                            backgroundColor: '#bababa',
                                            borderColor: '#484848',
                                        },
                                        info: 'Open the cutout service form',
                                        action(e) {
                                            aladinInstance.sodaQueryWindow.hide();
                                            aladinInstance.sodaQueryWindow.setParams(self.SODAServerParams);
                                            aladinInstance.sodaQueryWindow.show(aladinInstance);
                                        }
                                    }).element();
                                } else {
                                    return service || '--';
                                }
                            },
                            'access_url': (data) => {
                                let url = data['access_url'];

                                let accessUrlEl = document.createElement('div');

                                if (url) {
                                    let contentType = data['content_type'];
                                    let contentQualifier = data['content_qualifier'];
    
                                    try {
                                        // Just create a URL object to verify it is a good url
                                        // If not, it will throw an exception
                                        let _ = new URL(url);
                                        accessUrlEl.classList.add('aladin-href-td-container');
                                        accessUrlEl.innerHTML = '<a href=' + url + ' target="_blank">' + url + '</a>';

                                        accessUrlEl.addEventListener('click', (e) => {

                                            let processImageFitsClick = () => {
                                                var successCallback = ((ra, dec, fov, _) => {
                                                    aladinInstance.gotoRaDec(ra, dec);
                                                    aladinInstance.setFoV(fov);
                                                });

                                                let image = aladinInstance.createImageFITS(url, url, {}, successCallback);
                                                aladinInstance.setOverlayImageLayer(image, Utils.uuidv4())
                                            };

                                            switch (contentType) {
                                                case 'application/hips':
                                                    // Clic on a HiPS
                                                    let survey = aladinInstance.newImageSurvey(url);
                                                    aladinInstance.setOverlayImageLayer(survey, Utils.uuidv4())
                                                    break;
                                                // Any generic FITS file
                                                case 'application/fits':
                                                    if (contentQualifier === "cube") {
                                                        // fits cube
                                                        console.warn("Cube not handled, only first slice downloaded")
                                                    }
            
                                                    processImageFitsClick();
                                                    break;
                                                case 'image/fits':
                                                    if (contentQualifier === "cube") {
                                                        // fits cube
                                                        console.warn("Cube not handled, only first slice downloaded")
                                                    }
            
                                                    processImageFitsClick();
                                                    break;
                                                default:
                                                    // When all has been done, download what's under the link
                                                    //Utils.download(url);
                                                    break;
                                            }
                                        });
                                    } catch(e) {
                                        accessUrlEl.innerText = '--';
                                    }
                                } else {
                                    accessUrlEl.innerText = '--';
                                }

                                return accessUrlEl;
                            }
                        }
                    }

                    aladinInstance.measurementTable.showMeasurement([datalinkTable]);
                } else {
                    // Try to parse a SODA service descriptor resource
                    let SODAServerParams = VOTable.parseSODAServiceRsc(rsc);

                    if (SODAServerParams) {
                        this.SODAServerParams = SODAServerParams;

                        // Try to populate the SODA form fields with obscore values
                        let populateSODAFields = (SODAParams) => {
                            for (const name in SODAParams.inputParams) {
                                let inputParam = SODAParams.inputParams[name];

                                if (inputParam.type === "group") {
                                    for (const param of inputParam.subInputs) {
                                        if (param.value) {
                                            continue;
                                        }

                                        if (param.name === "ra") {
                                            param.value = obscoreRow['s_ra'];
                                        } else if (param.name === "dec") {
                                            param.value = obscoreRow['s_dec'];
                                        } else if (param.name === "fmin") {
                                            param.value = obscoreRow['em_min'];
                                        } else if (param.name === "fmax") {
                                            param.value = obscoreRow['em_max'];
                                        } else if (param.name === "rad") {
                                            param.value = obscoreRow['s_fov'] / 2;
                                        }
                                    }
                                }
                            }
                        };

                        // Request the base url of the SODA server to check if there
                        // is a self description VOTable
                        VOTable.parse(this.SODAServerParams.baseUrl, (rsc) => {
                            const SODAServerDesc = VOTable.parseSODAServiceRsc(rsc);

                            if (SODAServerDesc) {
                                for (const name in SODAServerDesc.inputParams) {
                                    let inputParam = SODAServerDesc.inputParams[name];
                                    if (!this.SODAServerParams.inputParams[name]) {
                                        this.SODAServerParams.inputParams[name] = inputParam;
                                    }
                                }
                            }

                            populateSODAFields(this.SODAServerParams);
                        }, undefined, true);

                        populateSODAFields(this.SODAServerParams);
                    }
                }
            },
            undefined,
            true
        )
    };

    return Datalink;
})();
