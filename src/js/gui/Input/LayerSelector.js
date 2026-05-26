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
 * File gui/Input/LayerSelector.js
 *
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

import { ALEvent } from "../../events/ALEvent.js";
import { Input } from "../Widgets/Input.js";

export class LayerSelector extends Input {
    static cachedLayers = {}
    static objects = [];

    // constructor
    constructor(options) {
        let surveys = [];
        for (var survey of Object.values(LayerSelector.cachedLayers)) {
            surveys.push({value: survey.id, label: survey.name});
        }
        surveys.sort((s1, s2) => {
            const s1l = s1.label.toLowerCase()
            const s2l = s2.label.toLowerCase()

            if (s1l < s2l) {
                return -1;
            }
            if (s1l > s2l) {
                return 1;
            }

            return 0;
        });
        
        let current = {};
        if (options.layer) {
            let id = options.layer.id;
            let name = options.layer.name;

            current["value"] = id;
            current["label"] = name;
            
            if (!surveys.some(item => item.value === id)) {
                surveys.push(current)
            }
        } else {
            current = surveys[0];
        }

        current["title"] = current["label"];

        surveys.push("More...")

        super({
            type: "select",
            options: surveys,
            ...current,
            ...options
        })

        self = this;

        LayerSelector.objects.push(self);
    }
};

(function () {
    ALEvent.FAVORITE_LAYERS_LIST_UPDATED.listenedBy(document, (event) => {
        let favoritesLayer = event.detail;

        LayerSelector.cachedLayers = {};

        for (var layer of favoritesLayer) {
            let key = layer.id || layer.url || layer.name;
            LayerSelector.cachedLayers[key] = layer;
        }

        // Update the options of the selector
        let favorites = []
        for(var layer of Object.values(LayerSelector.cachedLayers)) {
            favorites.push({
                value: layer.id,
                label: layer.name
            })
        }

        favorites.sort((s1, s2) => {
            const s1l = s1.label.toLowerCase()
            const s2l = s2.label.toLowerCase()

            if (s1l < s2l) {
                return -1;
            }
            if (s1l > s2l) {
                return 1;
            }

            return 0;
        });

        for (var selector of LayerSelector.objects) {
            // refers to an HiPS image survey
            let currentFavorite = {
                value: selector.options.value,
                label: selector.options.label,
            };

            let favoritesCopy = [...favorites];

            // Add the current hips to the selector as well, even if it has been manually
            // removed from the HiPSList
            if (!favoritesCopy.some(item => item.value === currentFavorite.value)) {
                favoritesCopy.push(currentFavorite)
            }

            favoritesCopy.push("More...")

            currentFavorite["title"] = currentFavorite["label"];

            selector.update({
                ...currentFavorite,
                options: favoritesCopy
            });
        }
    });
})();