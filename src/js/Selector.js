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

import { Aladin } from "./Aladin";
import { Color } from "./Color";
import { CircleSelect } from "./FiniteStateMachine/CircleSelect";
import { PolySelect } from "./FiniteStateMachine/PolySelect";
import { RectSelect } from "./FiniteStateMachine/RectSelect";

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
        let color = (options && options.color) || Aladin.DEFAULT_OPTIONS.reticleColor;
        this.color = new Color(color).toHex();
        this.lineWidth = (options && options.lineWidth) || 1;

        this.select = null;
        this.view = view;
    };

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

    dispatch(to, params) {
        this.select.dispatch(to, params);
    }

    static getObjects(selection, view) {
        if (!selection) {
            return;
        }

        if (!selection.contains) {
            return;
        }

        var objList = [];
        var cat, sources, s;
        var footprints, f;
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
                    if (selection.contains(s)) {
                        objListPerCatalog.push(s);
                    }
                }
                // footprints
                footprints = cat.getFootprints();
                if (footprints) {
                    const {x, y, w, h} = selection.bbox();
                    for (var l = 0; l < footprints.length; l++) {
                        f = footprints[l];
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
        return objList;
    }
}