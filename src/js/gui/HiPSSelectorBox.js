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

import { MocServer } from "../MocServer.js";
import  autocomplete from 'autocompleter';

import { SelectBox } from "./Widgets/Box.js";
import { Layout } from "./Layout.js";
import { Form } from "./Widgets/Form.js";
import { ActionButton } from "./Widgets/ActionButton.js";
import uploadImg from '../../../assets/icons/upload.svg';
import { Utils } from "../Utils.ts";
import { Input } from "./Widgets/Input.js";

/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/HiPSSelector.js
 *
 * 
 * Author: Thomas Boch, Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

 export class HiPSSelectorBox extends SelectBox {

    constructor(aladin, layer) {
        let fnIdSelected = (IDOrURL) => {
            let name;

            if (layer) {
                name = layer.layer;
            } else {
                name = Utils.uuidv4();
            }

            aladin.setOverlayImageLayer(IDOrURL, name);
        };

        let FITSFileLoaded = {};
        let inputTextFITS = new Input({
            layout: {
                name: 'FITS-name',
                type: 'text',
                placeholder: "Open the file dialog...",
                change(e) {
                    FITSFileLoaded.name = e.target;
                }
            }
        });

        let inputTextSurvey = new Form({
            label: "Survey",
            name: 'autocomplete',
            type: 'text',
            placeholder: "Type ID, title, keyword or URL",
            change(e) {
                let input = e.target;
                // Unfocus the keyboard on android devices (maybe it concerns all smartphones) when the user click on enter
                input.blur();
            }
        });

        let input = inputTextSurvey.getInput('autocomplete');


        super({
            header: {
                title: 'Survey selector',
                draggable: true,
            },
            selected: 'Survey',
            possibilities: [
                {
                    label: 'FITS',
                    content: Layout.horizontal({
                        cssStyle: {
                            'margin-bottom': '5px',
                            border: '2px solid #d2d2d2',
                            margin: '0px',
                            padding: '4px'
                        },
                        layout: [
                            Layout.horizontal({
                                layout: [
                                    ActionButton.createIconBtn({
                                        iconURL: uploadImg,
                                        tooltip: {content: 'Upload a FITS file', position: {direction: 'bottom'}},
                                        cssStyle: {
                                            backgroundPosition: 'center center',
                                            backgroundColor: '#bababa',
                                            border: '1px solid rgb(72, 72, 72)',
                                            cursor: 'help',
                                        },
                                        action(e) {
                                            let fileLoader = document.createElement('input');
                                            fileLoader.type = 'file';
                                            // Case: The user is loading a FITS file
                        
                                            fileLoader.addEventListener("change", (e) => {    
                                                let file = e.target.files[0];                                
                                                FITSFileLoaded = {
                                                    url: URL.createObjectURL(file),
                                                    name: file.name
                                                };
                        
                                                inputTextFITS.set(FITSFileLoaded.name); 
                                            });
                        
                                            fileLoader.click();
                                        }
                                    }),
                                    new Input({
                                        layout: {
                                            name: 'FITS-name',
                                            type: 'text',
                                            placeholder: "Open the file dialog...",
                                            change(e) {
                                                FITSFileLoaded.name = e.target;
                                            }
                                        }
                                    })
                                ]
                            }),
                            new ActionButton({
                                content: 'Load',
                                action(e) {
                                    const { url, name } = FITSFileLoaded;
                                    const image = aladin.createImageFITS(
                                        url,
                                        name,
                                        undefined,
                                        (ra, dec, fov, _) => {
                                            // Center the view around the new fits object
                                            aladin.gotoRaDec(ra, dec);
                                            aladin.setFoV(fov * 1.1);
                                        },
                                        undefined
                                    );

                                    let layerName;
                                    if (layer) {
                                        layerName = layer.layer;
                                    } else {
                                        layerName = Utils.uuidv4();
                                    }

                                    aladin.setOverlayImageLayer(image, layerName)
                                    inputTextFITS.set('');
                                }
                            })
                        ]
                    })
                },
                {
                    label: 'Survey',
                    content: Layout.horizontal({
                        cssStyle: {
                            'margin-bottom': '5px',
                            border: '2px solid #d2d2d2',
                            margin: '0px',
                            padding: '4px'
                        },
                        layout: [
                            inputTextSurvey,
                            new ActionButton({
                                content: 'Load',
                                action(e) {
                                    if (input) {
                                        fnIdSelected && fnIdSelected(input.value);
                                    }
                                    // reset the field
                                    inputTextSurvey.set('autocomplete', '');
                                }
                            })
                        ]
                    })
                }
            ]
        }, aladin.aladinDiv)

        /*let box = new Box({
            draggable: true,
            title: 'Survey selector',
            content: Layout.vertical({layout: [
                searchLayout,
                Layout.horizontal({layout: [
                    ,
                    new ActionButton({
                        content: 'Load coverage',
                        info: new Form({
                            header: "Coverage creation panel",
                            subInputs: [
                                {
                                    label: "Label",
                                    name: 'moc-name',
                                    type: 'text',
                                    placeholder: "Type the overlay name",
                                    change(e) {}
                                },
                                {
                                    label: "Color",
                                    type: 'color',
                                    change(e) {}
                                },
                                {
                                    label: "Perimeter",
                                    type: 'checkbox',
                                    change(e) {}
                                },
                                {
                                    label: "Filled",
                                    type: 'checkbox',
                                    change(e) {}
                                }
                            ]
                        }),
                        action(e) {
                            let url;
                            if (input.value.startsWith('http')) {
                                url = input.value + '/Moc.fits';
                            }
                            else {
                                url = self.selectedItem.hips_service_url + '/Moc.fits';
                            }
                            url = Utils.fixURLForHTTPS(url);
                
                            const moc = A.MOCFromURL(url, {lineWidth: 5, opacity: 0.3});
                            self.aladin.addMOC(moc);
                        }
                    }),
                ]})
            ]})
        }, this.parentDiv);*/
        this.addClass('aladin-anchor-center');

        // Query the mocserver
        MocServer.getAllHiPSes();

        autocomplete({
            input: input,
            fetch: function(text, update) {
                text = text.toLowerCase();
                // filter suggestions
                const suggestions = MocServer.getAllHiPSes().filter(n => n.ID.toLowerCase().includes(text) || n.obs_title.toLowerCase().includes(text))

                // sort suggestions
                suggestions.sort( function(a , b) {
                    let scoreForA = 0;
                    let scoreForB = 0;

                    if (a.ID.toLowerCase().includes(text)) {
                        scoreForA += 100;
                    }
                    if (b.ID.toLowerCase().includes(text)) {
                        scoreForB += 100;
                    }

                    if (a.obs_title.toLowerCase().includes(text)) {
                        scoreForA += 50;
                    }
                    if (b.obs_title.toLowerCase().includes(text)) {
                        scoreForB += 50;
                    }

                    if (a.obs_description && a.obs_description.toLowerCase().includes(text)) {
                        scoreForA += 10;
                    }
                    if (b.obs_description && b.obs_description.toLowerCase().includes(text)) {
                        scoreForB += 10;
                    }

                    if (scoreForA > scoreForB) {
                        return -1;
                    }
                    if (scoreForB > scoreForA) {
                        return  1;
                    }

                    return 0;
                });

                // limit to 50 first suggestions
                const returnedSuggestions = suggestions.slice(0, 50);

                update(returnedSuggestions);
            },
            onSelect: function(item) {
                input.value = item.ID;

                //self.fnIdSelected && self.fnIdSelected(item.ID);
                input.blur();
            },
            // attach container to AL div if needed (to prevent it from being hidden in full screen mode)
            customize: function(input, inputRect, container, maxHeight) {
                // this tests if we are in full screen mode
                if (aladin.isInFullscreen) {
                    aladin.aladinDiv.appendChild(container);
                }
            },
            render: function(item, currentValue) {
                const itemElement = document.createElement("div");
                itemElement.innerHTML = item.obs_title + ' - '  + '<span style="color: #ae8de1">' + item.ID + '</span>';


                return itemElement;
            }
        });
    }

    static layerSelector = undefined;

    static getInstance(aladin, layer) {
        if (!HiPSSelectorBox.layerSelector) {
            HiPSSelectorBox.layerSelector = new HiPSSelectorBox(aladin, layer);
        }

        return HiPSSelectorBox.layerSelector;
    }
}
