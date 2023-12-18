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

import { FSM } from "../FiniteStateMachine";
import { View } from "../View";
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

export class RectSelect extends FSM {
    // constructor
    constructor(options, view) {
        let start = (params) => {
            const {callback} = params;
            view.setCursor('crosshair');
            view.aladin.showReticle(false)

            this.callback = callback;
            view.setMode(View.SELECT)
        }

        let mousedown = (params) => {
            const {coo} = params;
            // start a new selection
            this.startCoo = coo;
        }

        let mousemove = (params) => {
            const {coo} = params;
            this.coo = coo;

            view.requestRedraw();
        };

        let draw = () => {
            let ctx = view.catalogCtx;

            if (!view.catalogCanvasCleared) {
                ctx.clearRect(0, 0, view.width, view.height);
                view.catalogCanvasCleared = true;
            }
            // draw the selection
            ctx.fillStyle = options.color + '7f';
            ctx.strokeStyle = options.color;
            ctx.lineWidth = options.lineWidth;

            var w = this.coo.x - this.startCoo.x;
            var h = this.coo.y - this.startCoo.y;

            ctx.fillRect(this.startCoo.x, this.startCoo.y, w, h);
            ctx.strokeRect(this.startCoo.x, this.startCoo.y, w, h); 
        }

        let mouseup = (params) => {
            var x, y;
            const {coo} = params;
            this.coo = coo;
            // finish the selection
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

            // TODO: remove these modes in the future
            view.aladin.showReticle(true)
            view.setCursor('default');

            // execute general callback
            if (view.callbacksByEventName) {
                var callback = view.callbacksByEventName['select'];
                if (typeof callback === "function") {
                    // !todo
                    let selectedObjects = view.selectObjects(this);
                    console.log(selectedObjects)

                    callback(selectedObjects);
                }
            }
            view.setMode(View.PAN)
            view.requestRedraw();
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
    };
}