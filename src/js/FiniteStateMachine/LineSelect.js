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
 * A line selector, used for example by the sky distance measuring tool or to retrieve pixels along a line
 * 
 * Author: Matthieu Baumann[CDS]
 * 
 *****************************************************************************/

export class LineSelect extends FSM {
    // constructor
    constructor(options, view) {
        // Off initial state
        let off = () => {   
            view.aladin.showReticle(true)
            view.setMode(View.PAN)
            view.setCursor('default');

            // in case of a mouseout we would like to erase the selection draw
            // in the canvas
            view.requestRedraw();

            view.aladin.removeStatusBarMessage('selector')
        }
        let mouseout = (params) => {
            let {e, coo} = params;
            self.dispatch('mousemove', {coo});
        };

        let start = (params) => {
            const {callback} = params;
            this.callback = callback;
            // reset the coo
            this.coos = [];
        }

        let mousedown = (params) => {
            const {coo} = params;

            this.coos.push(coo);
        };

        let mouseup = (params) => {
            const {coo} = params;

            this.coos.push(coo);
            self.dispatch('finish');
        };

        let mousemove = (params) => {
            const {coo} = params;
            this.moveCoo = coo;

            view.requestRedraw();
        };

        let draw = () => {
            let ctx = view.catalogCtx;
            
            // draw the selection
            ctx.save();
            let colorValue = (typeof options.color === 'function') ? options.color() : options.color;
            ctx.strokeStyle = colorValue;
            ctx.lineWidth = options.lineWidth;

            ctx.beginPath();

            const startCoo = this.coos[0];
            const endCoo = this.moveCoo;

            // Unproject the coordinates
            let [lon1, lat1] = view.aladin.pix2world(endCoo.x, endCoo.y);
            let [lon2, lat2] = view.aladin.pix2world(startCoo.x, startCoo.y);

            let vertices = view.wasm.projectGreatCircleArc(lon1, lat1, lon2, lat2)

            for (var i = 0; i < vertices.length; i+=4) {
                ctx.moveTo(vertices[i], vertices[i+1]);
                ctx.lineTo(vertices[i+2], vertices[i+3]);
            }
            
            ctx.stroke();
            ctx.restore();
        }

        let finish = () => {
            // finish the selection
            let s = {
                a: this.coos[0],
                b: this.coos[1],
                label: 'line',
            };
            (typeof this.callback === 'function') && this.callback(s);

            this.coos = [];

            // TODO execute general callback
            view.requestRedraw();

            this.dispatch('off');
        };

        let fsm = {
            state: 'off',
            transitions: {
                off: {
                    start,
                },
                start: {
                    mousedown
                },
                mousedown: {
                    mousemove,
                    draw,
                },
                mouseout: {
                    start,
                    mousemove,
                    draw,
                },
                mousemove: {
                    draw,
                    mouseup,
                },
                draw: {
                    mouseout,
                    mousemove,
                    mouseup,
                },
                mouseup: {
                    finish
                },
                finish: {
                    off
                }
            }
        };

        super(fsm)
        let self = this;

        this.coos = [];
    };
}