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

import { Box } from "../Widgets/Box.js";
import { Layout } from "../Layout.js";
import { ActionButton } from "../Widgets/ActionButton.js";
import { ALEvent } from "../../events/ALEvent.js";
/******************************************************************************
 * Aladin Lite project
 * 
 * File gui/HiPSSelector.js
 *
 * 
 * Author: Thomas Boch, Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

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
 * File Location.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/
 
 import { Input } from "./../Widgets/Input.js";
 
 export class HiPSSearch extends Input {
    static HiPSList = {};

     // constructor
     constructor(aladin, options) {
        let self;
        let layer = options && options.layer;

        aladin.view.catalogCanvas.addEventListener('click', (e) => {
            self.el.blur();
        });

        let prevKey = layer.name;
        super({
            name: 'HiPS search',
            type: 'text',
            classList: ['search'],
            name: 'survey',
            placeholder: "Survey keywords or url",
            autocomplete: {options: Object.keys(HiPSSearch.HiPSList)},
            title: layer.name,
            actions: {
                change(e) {
                    const key = e.target.value;
                    if (!key) {
                        self.update({value: prevKey, title: prevKey});
                        return;
                    }

                    let image;
                    // A user can put an url
                    try {
                        image = new URL(key).href;
                    } catch(e) {
                        // Or he can select a HiPS from the list given
                        let hips = HiPSSearch.HiPSList[key]
                        //console.log("HIPS", key, hips)
                        if (hips) {
                            image = hips.id || hips.url || undefined;
                        } else {
                            // Finally if not found, interpret the input text value as the HiPS (e.g. ID)
                            image = key;
                        }
                    }

                    self.el.blur();

                    if (image) {
                        prevKey = key;
                        aladin.setOverlayImageLayer(image, layer.layer);
                    }
                },
                /*input(e) {
                    let value = e.target.value;

                    self.update({value, title: value})
                }*/
            },
            value: layer.name,
            ...options
        })
        this.addClass('aladin-HiPS-search')

        self = this;
        this.layer = layer;

        this._addEventListeners(aladin);
    }

    setAutocompletionList(options) {
        this.update({autocomplete: {options}})
    }

    _addEventListeners(aladin) {
        let self = this;
        ALEvent.HIPS_LAYER_ADDED.listenedBy(aladin.aladinDiv, (e) => {
            const layer = e.detail.layer;
            if (layer.layer === self.layer.layer) {
                let value = layer.name
                self.update({value, title: value})
            }
        });
    }
};