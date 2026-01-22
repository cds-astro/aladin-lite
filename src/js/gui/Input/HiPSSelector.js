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
 * File gui/Input/HiPSSelector.js
 *
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

import { ALEvent } from "../../events/ALEvent.js";
import { Input } from "../Widgets/Input.js";
import { HiPSBrowserBox } from "../Box/HiPSBrowserBox.js";
import A from "../../A.js";

export class HiPSSelector extends Input {
    static cachedHiPS = {}
    static objects = [];

    // constructor
    constructor(options) {
        let surveys = Object.keys(HiPSSelector.cachedHiPS);
        surveys.sort()
        
        surveys = Array.from([...surveys])
        let current;
        if (options.layer) {
            current = options.layer.name || options.layer.id
        } else {
            current = surveys[0];
        }

        if (surveys.indexOf(current) < 0) {
            surveys.push(current)
        }

        surveys.push("More...")

        super({
            type: "select",
            value: current,
            options: surveys,
            title: current,
            ...options
        })

        self = this;

        HiPSSelector.objects.push(self);
    }
};

(function () {
    ALEvent.FAVORITE_HIPS_LIST_UPDATED.listenedBy(document.body, (event) => {
        let favoritesHips = event.detail;

        HiPSSelector.cachedHiPS = {};

        for (var hips of favoritesHips) {
            let key = hips.name || hips.id || hips.url;
            HiPSSelector.cachedHiPS[key] = hips;
        }

        // Update the options of the selector
        const favorites = Object.keys(HiPSSelector.cachedHiPS);
        for (var selector of HiPSSelector.objects) {
            // refers to an HiPS image survey
            let currentHiPS = selector.options.value

            let favoritesCopy = [...favorites];

            // add the current hips to the selector as well, even if it has been manually
            // removed from the HiPSList
            if (favoritesCopy.indexOf(currentHiPS) < 0) {
                favoritesCopy.push(currentHiPS)
            }

            // one must add the current HiPS too!
            favoritesCopy.sort();

            favoritesCopy.push("More...")

            selector.update({
                value: currentHiPS,
                options: favoritesCopy
            });
        }
    });
})();