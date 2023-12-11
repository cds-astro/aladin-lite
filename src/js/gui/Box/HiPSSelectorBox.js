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
import { ActionButton } from "../Widgets/ActionButton.js";
import { Utils } from "../../Utils.ts";

/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/HiPSSelector.js
 *
 * 
 * Author: Thomas Boch, Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

 export class HiPSSelectorBox extends Box {

    constructor(options, aladin) {
        let layer = options.layer;
        let fnIdSelected = (IDOrURL) => {
            let name;

            if (layer) {
                name = layer;
            } else {
                name = Utils.uuidv4();
            }

            aladin.setOverlayImageLayer(IDOrURL, name);
        };

        let inputText = Input.text({
            //tooltip: {content: 'Search for a VizieR catalogue', position: {direction :'bottom'}},
            label: "Survey",
            name: 'autocomplete',
            type: 'text',
            placeholder: "Type ID, title, keyword or URL",
            autocomplete: 'off',
            change(e) {
                loadBtn.update({disable: true})
                // Unfocus the keyboard on android devices (maybe it concerns all smartphones) when the user click on enter
                //inputText.element().blur();
            }
        });

        let self;
        let loadBtn = new ActionButton({
            content: 'Load',
            disable: true,
            action(e) {
                fnIdSelected && fnIdSelected(inputText.get());
                // reset the field
                inputText.set('');

                self._hide();
            }
        })
        super({
            ...options,
            content: Layout.horizontal({
                layout: [
                    inputText,
                    loadBtn
                ]
            })
        }, aladin.aladinDiv)
        self = this;
        // Query the mocserver
        MocServer.getAllHiPSes();

        autocomplete({
            input: inputText.element(),
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
                inputText.set(item.ID);
                loadBtn.update({disable: false});

                //self.fnIdSelected && self.fnIdSelected(item.ID);
                inputText.element().blur();
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

    static box = undefined;

    static getInstance(options, aladin) {
        if (!HiPSSelectorBox.box) {
            HiPSSelectorBox.box = new HiPSSelectorBox(options, aladin);
        }

        return HiPSSelectorBox.box;
    }
}
