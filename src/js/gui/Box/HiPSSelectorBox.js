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

import { Box } from "../Widgets/Box.js";
import { Layout } from "../Layout.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import { Input } from "../Widgets/Input.js";
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
    static HiPSList = {};

    constructor(aladin) {
        MocServer.getAllHiPSes()
            .then((HiPSes) => {
                HiPSes.forEach((h) => {
                    HiPSSelectorBox.HiPSList[h.obs_title] = h
                });

                inputText.update({autocomplete: {options: Object.keys(HiPSSelectorBox.HiPSList)}})
            });

        let self;
        let loadBtn = new ActionButton({
            content: 'Add',
            disable: true,
            action(e) {
                self.callback && self.callback(inputText.get());
                // reset the field
                inputText.set('');

                self._hide();
            }
        })

        let inputText = Input.text({
            classList: ['search'],
            name: 'survey',
            placeholder: "Type survey keywords",
            actions: {
                change() {
                    const HiPS = HiPSSelectorBox.HiPSList[this.value];
                    inputText.set(HiPS.ID);
                    loadBtn.update({disable: false});
                },
                keydown() {
                    loadBtn.update({disable: true});
                }
            }
        });

        super(
            aladin,
            {
                content: Layout.horizontal({
                    layout: [
                        inputText,
                        loadBtn
                    ]
                })
            }
        )

        self = this;
        // Query the mocserver
        /*MocServer.getAllHiPSes();

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
        });*/
    }

    attach(callback) {
        this.callback = callback;
    }
}
