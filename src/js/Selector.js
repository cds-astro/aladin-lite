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
import { FSM } from "./FiniteStateMachine";
import { View } from "./View";
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

export class Selector extends FSM {
    // constructor
    constructor(view, options) {
        let start = (params) => {
            const {mode, callback} = params;
            this.view.setCursor('crosshair');
            this.view.aladin.showReticle(false)

            this.mode = mode;
            this.callback = callback;
            this.view.setMode(View.SELECT)
        }

        let mousedown = (params) => {
            const {coo} = params;
            // start a new selection
            this.startCoo = coo;
        }

        let mousemove = (params) => {
            const {coo} = params;
            this.coo = coo;

            this.view.requestRedraw();
        };

        let draw = () => {
            let ctx = this.view.catalogCtx;

            if (!this.view.catalogCanvasCleared) {
                ctx.clearRect(0, 0, this.view.width, this.view.height);
                this.view.catalogCanvasCleared = true;
            }
            // draw the selection
            ctx.fillStyle = this.color + '7f';
            ctx.strokeStyle = this.color;
            ctx.lineWidth = this.lineWidth;
            switch (this.mode) {
                case 'rect':
                    var w = this.coo.x - this.startCoo.x;
                    var h = this.coo.y - this.startCoo.y;
    
                    ctx.fillRect(this.startCoo.x, this.startCoo.y, w, h);
                    ctx.strokeRect(this.startCoo.x, this.startCoo.y, w, h);
                    break;
                case 'circle':
                    var r2 = (this.coo.x - this.startCoo.x) * (this.coo.x - this.startCoo.x) + (this.coo.y - this.startCoo.y) * (this.coo.y - this.startCoo.y);
                    var r = Math.sqrt(r2);
    
                    ctx.beginPath();
                    ctx.arc(this.startCoo.x, this.startCoo.y, r, 0, 2 * Math.PI);
                    ctx.fill();
                    ctx.stroke();
                    break;
                default:
                    break;
            }
        }

        let mouseup = (params) => {
            var x, y;
            const {coo} = params;
            this.coo = coo;
            // finish the selection
            switch (this.mode) {
                case 'rect':
                    var w = this.coo.x - this.startCoo.x;
                    var h = this.coo.y - this.startCoo.y;
                    x = this.startCoo.x;
                    y = this.startCoo.y;
                    
                    if (w < 0) {
                        x = x + w;
                        w = -w;
                    }
                    if (h < 0) {
                        y = y + h;
                        h = -h;
                    }

                    (typeof this.callback === 'function') && this.callback({
                        x, y, w, h,
                        label: 'rect',
                        contains(s) {
                            return s.x >= x && s.x <= x + w && s.y >= y && s.y <= y + h;
                        },
                        bbox() {
                            return {x, y, w, h}
                        }
                    });
                    break;
                case 'circle':
                    var r2 = (this.coo.x - this.startCoo.x) * (this.coo.x - this.startCoo.x) + (this.coo.y - this.startCoo.y) * (this.coo.y - this.startCoo.y);
                    var r = Math.sqrt(r2);

                    x = this.startCoo.x;
                    y = this.startCoo.y;
                    (typeof this.callback === 'function') && this.callback({
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
                    });
                    break;
                default:
                    break;
            }

            // TODO: remove these modes in the future
            this.view.aladin.showReticle(true)
            this.view.setCursor('default');

            // execute general callback
            if (this.view.callbacksByEventName) {
                var callback = this.view.callbacksByEventName['select'];
                if (typeof callback === "function") {
                    // !todo
                    let selectedObjects = this.view.selectObjects(this);
                    callback(selectedObjects);
                }
            }
            this.view.setMode(View.PAN)
            this.view.requestRedraw();
        };

        let mouseout = mouseup;

        super({
            state: 'mouseup',
            transitions: {
                start: {
                    mousedown
                },
                mousedown: {
                    mousemove
                },
                mousemove: {
                    draw,
                    mouseup,
                    mouseout
                },
                draw: {
                    mousemove,
                    mouseup,
                    mouseout
                },
                mouseout: {
                    start
                },
                mouseup: {
                    start,
                }
            }
        })

        this.view = view;
        this.options = options;

        let color = (options && options.color) || Aladin.DEFAULT_OPTIONS.reticleColor;
        this.color = new Color(color).toHex();
        this.lineWidth = (options && options.lineWidth) || 2;
    };

    static getObjects(selection, view) {
        if (!selection) {
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

    _getBBox() {
        let x, y, w, h;
        switch (this.mode) {
            case 'rect':
                w = this.coo.x - this.startCoo.x;
                h = this.coo.y - this.startCoo.y;
                x = this.startCoo.x;
                y = this.startCoo.y;
                break;
            case 'circle':
                var r2 = (this.coo.x - this.startCoo.x) * (this.coo.x - this.startCoo.x) + (this.coo.y - this.startCoo.y) * (this.coo.y - this.startCoo.y);
                var r = Math.sqrt(r2);

                x = this.startCoo.x - r;
                y = this.startCoo.y - r;
                w = 2 * r;
                h = 2 * r;
                break;
            default:
                break;
        }

        return {x: x, y: y, w: w, h: h};
    };
}