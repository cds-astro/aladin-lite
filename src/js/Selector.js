// Copyright 2015 - UDS/CNRS
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

import { Color } from "./Color";
import { CircleSelect } from "./FiniteStateMachine/CircleSelect";
import { PolySelect } from "./FiniteStateMachine/PolySelect";
import { RectSelect } from "./FiniteStateMachine/RectSelect";
import { ALEvent } from "./events/ALEvent";
/******************************************************************************
 * Aladin Lite project
 * 
 * Class Selector
 * 
 * A selector
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

export class Selector {
    // constructor
    constructor(view, options) {
        this.customColor = false;
        this.color = options && options.color;
        if (this.color) {
            this.customColor = true;
        }

        this.lineWidth = (options && options.lineWidth) || 2;

        this.select = null;
        this.view = view;

        this._addListeners(view.aladin)
    };

    _addListeners(aladin) {
        let self = this;
        ALEvent.RETICLE_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            if (!self.customColor) {
                let reticleColor = e.detail.color;
                // take the color of the reticle
                self.color = new Color(reticleColor).toHex();
            }
        })
    }

    setMode(mode) {
        if (mode) {
            let options = {
                color: this.color,
                lineWidth: this.lineWidth
            };

            if (mode === 'circle') {
                this.select = new CircleSelect(options, this.view)
            } else if (mode === 'rect') {
                this.select = new RectSelect(options, this.view)
            } else if (mode === 'poly') {
                this.select = new PolySelect(options, this.view)
            }
        }
    }

    start(mode, callback) {
        this.setMode(mode);
        this.dispatch('start', {callback})

        this.view.aladin.addStatusBarMessage({
            id: 'selector',
            message: 'You entered the selection mode',
            type: 'info'
        })
    }

    cancel() {
        this.dispatch('off')
    }

    dispatch(to, params) {
        this.select.dispatch(to, params);
    }

    static getObjects(selection, view) {
        if (!selection) {
            return;
        }

        if (!selection.contains) {
            // contains must be implemented for the region
            return;
        }

        var objList = [];
        var cat, sources, s;
        var overlayItems, f;
        var objListPerCatalog = [];
        if (view.catalogs) {
            for (var k = 0; k < view.catalogs.length; k++) {
                cat = view.catalogs[k];

                if (!cat.isShowing) {
                    continue;
                }
                sources = cat.getSources();
                for (var l = 0; l < sources.length; l++) {
                    s = sources[l];
                    if (!s.isShowing || !s.x || !s.y || s.tooSmallFootprint === false) {
                        continue;
                    }
                    if (selection.contains(s)) {
                        objListPerCatalog.push(s);
                    }
                }
                // footprints
                overlayItems = cat.getFootprints();
                if (overlayItems) {
                    const {x, y, w, h} = selection.bbox();
                    for (var l = 0; l < overlayItems.length; l++) {
                        f = overlayItems[l];
                        if (f.intersectsBBox(x, y, w, h, view)) {
                            objListPerCatalog.push(f);
                        }
                    }
                }

                if (objListPerCatalog.length > 0) {
                    objList.push(objListPerCatalog);
                }
                objListPerCatalog = [];
            }
        }

        if (view.overlays) {
            const {x, y, w, h} = selection.bbox();
            for (var k = 0; k < view.overlays.length; k++) {
                let overlay = view.overlays[k];
                if (!overlay.isShowing) {
                    continue;
                }
                var overlayItems = overlay.overlayItems;
                for (var l = 0; l < overlayItems.length; l++) {
                    let o = overlayItems[l];
                    if (!o.isShowing) {
                        continue;
                    }

                    if (o.intersectsBBox(x, y, w, h, view)) {
                        objList.push([o]);
                    }
                }
            }
        }

        return objList;
    }
}