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
import { ActionButton } from "../gui/Widgets/ActionButton";
import { View } from "../View";
import finishIconUrl from '../../../assets/icons/finish.svg';
import { Utils } from "../Utils";

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

export class PolySelect extends FSM {
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
        }
        let btn;
        let mouseout = (params) => {
            let {e, coo} = params;

            if (btn.el.contains(e.relatedTarget) || e.relatedTarget.contains(btn.el)) {
                // mouseout on the btn
                self.dispatch('mousemove', {coo});
            } else {
                btn.remove();

                off();
            }
        };

        let start = (params) => {
            const {callback} = params;
            view.setMode(View.SELECT)

            this.callback = callback;
            // reset the coo
            this.coos = [];

        }

        let click = (params) => {
            const {coo} = params;

            const firstClick = this.coos.length === 0;
            if (firstClick) {
                // create a btn at the first click
                btn = ActionButton.createIconBtn({
                    position: {
                        left: coo.x,
                        top: coo.y,
                    },
                    cssStyle: {
                        zIndex: 100,
                    },
                    tooltip: {content: 'Finish the selection', position: {direction: 'bottom'}},
                    iconURL: finishIconUrl, 
                    action(e) {
                        e.stopPropagation();
                        e.preventDefault()

                        btn.remove();
                        self.dispatch('finish');
                    }
                });

                btn.attachTo(view.aladin.aladinDiv);
            }

            this.coos.push(coo);

            view.requestRedraw();
        };

        let mousemove = (params) => {
            const {coo} = params;
            this.moveCoo = coo;

            view.requestRedraw();
        };

        let draw = () => {
            let ctx = view.catalogCtx;

            if (!view.catalogCanvasCleared) {
                ctx.clearRect(0, 0, view.width, view.height);
                view.catalogCanvasCleared = true;
            }
            // draw the selection
            ctx.save();
            ctx.fillStyle = options.color + '7f';
            ctx.strokeStyle = options.color;
            ctx.lineWidth = options.lineWidth;

            ctx.beginPath();

            const startCoo = this.coos[0];
            ctx.moveTo(startCoo.x, startCoo.y);

            for (var i = 1; i < this.coos.length; i++) {
                ctx.lineTo(this.coos[i].x, this.coos[i].y);
            }

            if (this.moveCoo)
                ctx.lineTo(this.moveCoo.x, this.moveCoo.y);

            ctx.stroke();
            ctx.fill();
            ctx.restore();
        }

        let finish = () => {
            // finish the selection
            let xMin = this.coos[0].x
            let yMin = this.coos[0].y
            let xMax = this.coos[0].x
            let yMax = this.coos[0].y
            for (var i = 1; i < this.coos.length; i++) {
                xMin = Math.min(xMin, this.coos[i].x)
                yMin = Math.min(yMin, this.coos[i].y)
                xMax = Math.max(xMax, this.coos[i].x)
                yMax = Math.max(yMax, this.coos[i].y)
            }

            let x = xMin;
            let y = yMin;
            let w = xMax - xMin;
            let h = yMax - yMin;
            (typeof this.callback === 'function') && this.callback({
                vertices: this.coos,
                label: 'polygon',
                bbox() {
                    return {x, y, w, h}
                }
            });

            this.coos = [];

            // TODO execute general callback
            view.mustClearCatalog = true;
            view.requestRedraw();

            this.dispatch('off');
        };

        let fsm;
        if (Utils.hasTouchScreen()) {
            let mousedown = click;
            let mouseup = click;

            // smartphone, tablet
            fsm = {
                state: 'off',
                transitions: {
                    off: {
                        start,
                    },
                    start: {
                        mousedown
                    },
                    mousedown: {
                        //mouseout,
                        mousemove,
                        draw,
                    },
                    mouseout: {
                        start,
                        mousemove,
                    },
                    mousemove: {
                        draw,
                        mouseup,
                        finish
                    },
                    mouseup: {
                        mousedown,
                        finish,
                        draw,
                    },
                    draw: {
                        mouseup,
                        mouseout,
                        mousemove,
                        finish
                    },
                    finish: {
                        off
                    }
                }
            }
        } else {
            // desktop, laptops...
            fsm = {
                state: 'off',
                transitions: {
                    off: {
                        start,
                    },
                    start: {
                        click
                    },
                    click: {
                        //mouseout,
                        mousemove,
                        draw,
                    },
                    mouseout: {
                        start,
                        mousemove,
                    },
                    mousemove: {
                        draw,
                        click,
                        finish
                    },
                    draw: {
                        click,
                        mouseout,
                        mousemove,
                        finish
                    },
                    finish: {
                        off
                    }
                }
            }
        }

        super(fsm)
        let self = this;

        this.coos = [];
    };
}