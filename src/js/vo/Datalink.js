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
import { ActionButton } from "../gui/Widgets/ActionButton.js";
import { Catalog } from "../Catalog.js";
import { ServiceQueryBox } from "../gui/Box/ServiceQueryBox.js";
import { Input } from "../gui/Widgets/Input.js";
import { Layout } from "../gui/Layout.js";
import { HiPSProperties } from "../HiPSProperties.js";
import A from "../A.js";

export let Datalink = (function() {

    let Datalink = function () {
        this.services = {};
    };

    Datalink.prototype.handleActions = function(url, obscoreRow, aladinInstance) {
        //const url = obscoreRow["access_url"];
        new VOTable(
            url,
            (rsc) => {
                rsc = VOTable.parseRsc(rsc);

                // It is a table
                if (rsc && rsc.fields && rsc.rows) {
                    let table = rsc;
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
                        name: 'Datalink:' + url,
                        color: 'purple',
                        rows: measures,
                        fields,
                        showCallback: {
                            content_type: (data) => {
                                let contentType = data['content_type'];

                                if (contentType && contentType.includes('datalink')) {
                                    return new ActionButton({
                                        size: 'small',
                                        content: '🔗',
                                        tooltip: {content: 'Datalink VOTable', aladin: aladinInstance, global: true},
                                        action(e) {}
                                    }).element();
                                } else {
                                    return contentType;
                                }
                            },
                            'service_def': (data) => {
                                const serviceName = data['service_def'];

                                if (data['semantics'] === "#cutout") {
                                    return new ActionButton({
                                        size: 'small',
                                        content: '📡',
                                        tooltip: {global: true, aladin: aladinInstance, content: 'Open the cutout service form'},
                                        action(e) {
                                            if (self.serviceQueryBox) {
                                                self.serviceQueryBox.remove()
                                            }

                                            self.serviceQueryBox = new ServiceQueryBox(aladinInstance);
                                            self.serviceQueryBox._hide();
                                            self.serviceQueryBox.attach(self.services[serviceName]);
                                            self.serviceQueryBox._show({
                                                position: {
                                                    anchor: 'center center'
                                                }
                                            });
                                        }
                                    }).element();
                                } else {
                                    return serviceName || '--';
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
                                            e.preventDefault();
                                            e.stopPropagation();

                                            let processImageFitsClick = () => {
                                                var successCallback = ((ra, dec, fov, _) => {
                                                    aladinInstance.gotoRaDec(ra, dec);
                                                    aladinInstance.setFoV(fov);
                                                });

                                                let image = aladinInstance.createImageFITS(url, {name: url}, successCallback);
                                                aladinInstance.setOverlayImageLayer(image, Utils.uuidv4())
                                            };

                                            switch (contentType) {
                                                // A datalink response containing links to datasets or services attached to the current dataset
                                                case 'application/x-votable+xml;content=datalink':
                                                    new Datalink().handleActions(url, obscoreRow, aladinInstance);
                                                    break;
                                                case 'application/hips':
                                                    // Clic on a HiPS
                                                    let layer = Utils.uuidv4();
                                                    if (contentQualifier === "cube") {
                                                        let cubeOnTheFlyUrl = url + '/';

                                                        HiPSProperties.fetchFromUrl(cubeOnTheFlyUrl)
                                                            .then(properties => {
                                                                let numSlices = +properties.hips_cube_depth;
                                                                let idxSlice = +properties.hips_cube_firstframe;
                                                
                                                                let updateSlice = () => {
                                                                    let colorCfg = aladinInstance.getOverlayImageLayer(layer).getColorCfg();
                                                                    let hips = aladinInstance.setOverlayImageLayer(cubeOnTheFlyUrl + idxSlice, layer)
                                                                    hips.setColorCfg(colorCfg)
                                                
                                                                    slicer.update({
                                                                        value: idxSlice,
                                                                        tooltip: {content: (idxSlice + 1) + '/' + numSlices, position: {direction: 'bottom'}},
                                                                    })

                                                                    cubeDisplayer.update({content: Layout.horizontal([prevBtn, nextBtn, slicer, (idxSlice + 1) + '/' + numSlices])})
                                                                };
                                                
                                                                let slicer = Input.slider({
                                                                    label: "Slice",
                                                                    name: "cube slicer",
                                                                    ticks: [idxSlice],
                                                                    tooltip: {content: (idxSlice + 1) + '/' + numSlices , position: {direction: 'bottom'}},
                                                                    min: 0,
                                                                    max: numSlices - 1,
                                                                    value: idxSlice,
                                                                    actions: {
                                                                        change: (e) => {
                                                                            idxSlice = Math.round(e.target.value);
                                                
                                                                            updateSlice();
                                                                        },
                                                                        input: (e) => {
                                                                            idxSlice = Math.round(e.target.value);

                                                                            slicer.update({
                                                                                value: idxSlice,
                                                                                tooltip: {content: (idxSlice + 1) + '/' + numSlices, position: {direction: 'bottom'}},
                                                                            })
                                                                        }
                                                                    },
                                                                    cssStyle: {
                                                                        width: '300px'
                                                                    }
                                                                });
                                                
                                                                let prevBtn = A.button({
                                                                    size: 'small',
                                                                    content: '<',
                                                                    action(o) {
                                                                        idxSlice = Math.max(idxSlice - 1, 0);
                                                                        updateSlice()
                                                                    }
                                                                })
                                                
                                                                let nextBtn = A.button({
                                                                    size: 'small',
                                                                    content: '>',
                                                                    action(o) {
                                                                        idxSlice = Math.min(idxSlice + 1, numSlices - 1);
                                                                        updateSlice()
                                                                    }
                                                                })
                                                
                                                                let cubeDisplayer = A.box({
                                                                    close: true,
                                                                    header: {
                                                                        title: 'HiPS cube player',
                                                                        draggable: true,
                                                                    },
                                                                    content: Layout.horizontal([prevBtn, nextBtn, slicer, (idxSlice + 1) + '/' + numSlices]),
                                                                    position: {anchor: 'center top'},
                                                                });
                                                                aladinInstance.addUI(cubeDisplayer)
                                                
                                                                aladinInstance.setOverlayImageLayer(cubeOnTheFlyUrl + idxSlice, layer)
                                                            })
                                                    } else {
                                                        let survey = aladinInstance.newImageSurvey(url);
                                                        aladinInstance.setOverlayImageLayer(survey, layer)
                                                    }
                                                   
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
                                                    Utils.openNewTab(url);
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
                // SODA service descriptor
                } else if (rsc && rsc.baseUrl && rsc.inputParams) {
                    // Try to parse a SODA service descriptor resource
                    let serviceDesc = rsc;

                    if (serviceDesc) {
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
                        new VOTable(serviceDesc.baseUrl, (rsc) => {
                            const res = VOTable.parseRsc(rsc);

                            if (res && res.baseUrl && res.inputParams) {
                                for (const name in res.inputParams) {
                                    let inputParam = res.inputParams[name];
                                    if (!serviceDesc.inputParams[name]) {
                                        serviceDesc.inputParams[name] = inputParam;
                                    }
                                }
                            }

                            populateSODAFields(serviceDesc);
                        }, undefined, true);

                        populateSODAFields(serviceDesc);

                        this.services[serviceDesc.name] = serviceDesc;
                    }
                }
            },
            undefined,
            true
        )
    };

    return Datalink;
})();
