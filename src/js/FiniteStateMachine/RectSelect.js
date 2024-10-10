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
import { Selector } from "../Selector";
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

            // draw the selection
            let colorValue = (typeof options.color === 'function') ? options.color(this.startCoo, this.coo) : options.color;

            ctx.fillStyle = colorValue + '7f';
            ctx.strokeStyle = colorValue;
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

            let s = {
                x, y, w, h,
                label: 'rect',
                contains(s) {
                    return s.x >= x && s.x <= x + w && s.y >= y && s.y <= y + h;
                },
                bbox() {
                    return {x, y, w, h}
                }
            };

            (typeof this.callback === 'function') && this.callback(s, Selector.getObjects(s, view));

            // TODO: remove these modes in the future
            view.aladin.showReticle(true)
            view.setCursor('default');

            // execute general callback
            if (view.aladin.callbacksByEventName) {
                var callback = view.aladin.callbacksByEventName['objectsSelected'] || view.aladin.callbacksByEventName['select'];
                if (typeof callback === "function") {
                    let objList = Selector.getObjects(s, view);
                    view.selectObjects(objList);
                    callback(objList);
                }
            }

            this.dispatch('off');
        };

        let off = () => {
            view.aladin.showReticle(true)
            view.setMode(View.PAN)
            view.setCursor('default');

            // in case of a mouseout we would like to erase the selection draw
            // in the canvas
            view.requestRedraw();

            view.aladin.removeStatusBarMessage('selector')
        }

        let mouseout = mouseup;

        super({
            state: 'off',
            transitions: {
                off: {
                    start,
                },
                start: {
                    mousedown,
                    mouseup,
                    mouseout,
                    off
                },
                mousedown: {
                    mousemove,
                    off
                },
                mousemove: {
                    draw,
                    mouseup,
                    mouseout,
                    off
                },
                draw: {
                    mousemove,
                    mouseup,
                    mouseout,
                    off
                },
                mouseout: {
                    off
                },
                mouseup: {
                    off,
                }
            }
        })
    };
}