// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
//


import { Color } from "./Color";
import { CircleSelect } from "./FiniteStateMachine/CircleSelect";
import { PolySelect } from "./FiniteStateMachine/PolySelect";
import { LineSelect } from "./FiniteStateMachine/LineSelect";
import { RectSelect } from "./FiniteStateMachine/RectSelect";
import { ALEvent } from "./events/ALEvent";
import { Utils } from './Utils';
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

    start(mode, callback) {
        this.view.aladin.addStatusBarMessage({
            id: 'selector',
            message: 'You entered the selection mode',
            type: 'info'
        })

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
        } else if (mode === 'line') {
            this.select = new LineSelect(options, this.view)
        }

        this.dispatch('start', {callback})
    }

    cancel() {
        this.select && this.dispatch('off')
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

        const bbox = selection.bbox();
        var objList = [];
        var cat, sources, s;
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

                    if (!s.isShowing || !s.x || !s.y) {
                        continue;
                    }

                    // footprints
                    if (s.isFootprint() && s.tooSmallFootprint === false) {
                        if (s.footprint.intersectsBBox(bbox.x, bbox.y, bbox.w, bbox.h, view)) {
                            objListPerCatalog.push(s);
                        }

                        continue;
                    }

                    if (selection.contains(s)) {
                        objListPerCatalog.push(s);
                    }
                }

                if (objListPerCatalog.length > 0) {
                    objList.push(objListPerCatalog);
                }
                objListPerCatalog = [];
            }
        }

        if (view.overlays) {
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

                    if (o.intersectsBBox(bbox.x, bbox.y, bbox.w, bbox.h, view)) {
                        objList.push([o]);
                    }
                }
            }
        }

        return objList;
    }

    /**
     * Retrieves objects skewered by the cursor position or specified coordinates.  An object is
     * skewered if it is a shape that contains the specified coordinate, or is a catalog object within 3 pixels
     * of the specified coordinate.
     *
     * If e is a mouse event (as opposed to an object with x and y values), the mouse coordinates
     * of the event are used.
     *
     * This is implemented by simulating the interactive selection of a circle region with a 3 pixel radius)
     * around the given coordinates and returns all catalog sources and overlay items intersecting with it.
     *
     * @param {Event|Object} e - Mouse coordinate via mouse event or object with x and y properties
     * @param {Object} view - The Aladin View instance containing catalogs and overlays
     * @returns {Array<Array>} Array of object lists, where each subarray contains objects
     *          from a single catalog or overlay that intersect with the selection region.
     *          Returns empty array if no objects are found.
     */
    static getSkewerObjects(e, view) {
        // Get the xy from the event
        let xymouse;
        if (e instanceof Event) {
            xymouse = Utils.relMouseCoords(e);
        } else {
            xymouse = e;
        }
        const x = xymouse.x;
        const y = xymouse.y;

        // Perform a selection using a circle around x, y as if drawn by dragging 3 pixels.
        const r2 = 9;
        const r = Math.sqrt(r2);

        let selectorObject = {
            x, y, r,
            label: 'circle',
            contains(s) {
                let dx = (s.x - x)
                let dy = (s.y - y);

                return dx*dx + dy*dy <= r2;
            },
            bbox() {
                return {
                    x: x - r,
                    y: y - r,
                    w: 2*r,
                    h: 2*r
                }
            }
        };

        let objList = Selector.getObjects(selectorObject, view);

        return objList;
    }
}