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

import { MocServer } from "../../MocServer.js";
import  autocomplete from 'autocompleter';

import { Box } from "../Widgets/Box.js";
import { Layout } from "../Layout.js";
import { Input } from "../Widgets/Input.js";
import A from "../../A.js";
import { ConeSearchBox } from "./ConeSearchBox.js";
import { CtxMenuActionButtonOpener } from "../Button/CtxMenuOpener.js";
import { ContextMenu } from "../Widgets/ContextMenu.js";
/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/HiPSSelector.js
 *
 * 
 * Author: Thomas Boch, Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

 export class CatalogQueryBox extends Box {
    constructor(aladin, position) {
        const fnIdSelected = function(type, params) {
            if (type=='coneSearch') {
                let errorCallback = (e) => {
                    alert(e + '.\nThe table might contain no data for the cone search specified.');
                }
                if (params.baseURL.includes('/vizier.')) {
                    A.catalogFromVizieR(
                        params.id.replace('CDS/', ''),
                        params.ra + ' ' + params.dec,
                        params.radiusDeg,
                        {limit: params.limit, onClick: 'showTable'},
                        (catalog) => {
                            aladin.addCatalog(catalog)
                        },
                        errorCallback
                    );
                }
                else {
                    let url = params.baseURL;
                    if (! url.endsWith('?')) {
                        url += '?';
                    }
                    url += 'RA=' + params.ra + '&DEC=' + params.dec + '&SR=' + params.radiusDeg;
                    A.catalogFromURL(
                        url,
                        {limit: params.limit, onClick: 'showTable'},
                        (catalog) => {
                            aladin.addCatalog(catalog)
                        },
                        errorCallback
                    );
                }
            }
            else if (type=='hips') {
                const hips = A.catalogHiPS(params.hipsURL, {onClick: 'showTable', name: params.id});
                aladin.addCatalog(hips);
            }
        };

        let catNameTextInput = Input.text({
            //tooltip: {content: 'Search for a VizieR catalogue', position: {direction :'bottom'}},
            label: "catalogue",
            name: 'autocomplete',
            type: 'text',
            placeholder: "Type ID, title, keyword or URL",
            autocomplete: 'off',
            change(e) {
                self._selectItem(undefined, aladin)
                //resetCatalogueSelection();
                // Unfocus the keyboard on android devices (maybe it concerns all smartphones) when the user click on enter
                //input.element().blur();
            }
        });
        let self;

        let loadBtn = new CtxMenuActionButtonOpener({
            openDirection: "left",
            content: 'Load',
            disable: true,
        }, aladin)

        super(aladin, {
            position,
            content: Layout.horizontal({
                layout: [catNameTextInput, loadBtn]
            })
        })

        this.addClass('aladin-box-night')

        self = this;
        this.loadBtn = loadBtn;
        this.catNameTextInput = catNameTextInput;
        this.fnIdSelected = fnIdSelected;
        // Query the mocserver
        MocServer.getAllCatalogHiPSes();
        autocomplete({
            input: catNameTextInput.element(),
            minLength: 3,
            fetch: function(text, update) {
                text = text.toLowerCase();

                const filterCats = function(item) {
                    const ID = item.ID;
                    const obsTitle = item.obs_title || '';
                    const obsDescription = item.obs_description || '';

                    return ID.toLowerCase().includes(text) || obsTitle.toLowerCase().includes(text) || obsDescription.toLowerCase().includes(text);
                }

                // filter suggestions
                const suggestions = MocServer.getAllCatalogHiPSes().filter(filterCats);
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

                    if (a.obs_description.toLowerCase().includes(text)) {
                        scoreForA += 10;
                    }
                    if (b.obs_description.toLowerCase().includes(text)) {
                        scoreForB += 10;
                    }

                    // HiPS catalogue available
                    if (a.hips_service_url) {
                        scoreForA += 20;
                    }
                    if (b.hips_service_url) {
                        scoreForB += 20;
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
                catNameTextInput.set(item.ID);
                self._selectItem(item, aladin);

                // enable the load button
                //loadBtn.update({disable: false});

                catNameTextInput.element().blur();
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
                itemElement.innerHTML = (item.obs_title || '') + ' - '  + '<span style="color: #ae8de1">' + item.ID + '</span>';

                return itemElement;
            },
        });
    }

    _selectItem(item, aladin) {
        this.selectedItem = item;

        if (!item) {
            this.loadBtn.update({disable: true}, aladin)
            return;
        }

        let self = this;
        let layout = [];

        if (item && item.cs_service_url) {
            layout.push({
                label: 'Cone search',
                disable: !item.cs_service_url,
                action(o) {
                    let box = ConeSearchBox.getInstance(aladin);
                    box.attach({
                        callback: (cs) => {
                            self.fnIdSelected('coneSearch', {
                                baseURL: self.selectedItem.cs_service_url,
                                id: self.selectedItem.ID,
                                ra: cs.ra,
                                dec: cs.dec,
                                radiusDeg: cs.rad,
                                limit: cs.limit
                            })
                        },
                        position: {
                            anchor: 'center center',
                        }
                    })
                    box._show();

                    self._hide();
                }
            })
        }
        
        if (item && item.hips_service_url) {
            layout.push({
                label: 'HiPS catalogue',
                disable: !item.hips_service_url,
                action(o) {
                    self.fnIdSelected('hips', {
                        hipsURL: item.hips_service_url,
                        id: item.ID,
                    })

                    self._hide();
                }
            })
        }
        this.loadBtn.update({ctxMenu: layout, disable: false}, aladin)
    }

    _hide() {
        if (this.loadBtn) {
            this.loadBtn._hide();
        }

        super._hide()
    }

    static layerSelector = undefined;

    static getInstance(aladin, position) {
        if (!CatalogQueryBox.layerSelector) {
            CatalogQueryBox.layerSelector = new CatalogQueryBox(aladin, position);
        }

        return CatalogQueryBox.layerSelector;
    }
}
