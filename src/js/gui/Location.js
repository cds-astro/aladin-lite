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
 * File Location.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/


import { Coo }            from "../libs/astro/coo.js";
import { CooFrameEnum }   from "../CooFrameEnum.js";

import { DOMElement } from "./Widgets/Widget.js";

import { ALEvent } from "../events/ALEvent.js";
import { Layout } from "./Layout.js";
import { ActionButton } from "./Widgets/ActionButton.js";

export class Location extends DOMElement {
    // constructor
    constructor(aladin) {
        let self;

        let el = Layout.horizontal({
            layout: [
                ActionButton.createIconBtn({
                    content: "📋",
                    cssStyle: {
                        backgroundColor: '#bababa',
                        borderColor: '#484848',
                        textAlign: 'center',
                        fontSize: '10px',
                        width: '17px',
                        height: '15px',
                        fontWeight: 'bold',
                        lineHeight: '0px',
                        padding: '8px 2px' 
                    },
                    tooltip: {content: 'Copy to clipboard!', position: {direction: 'bottom'}},
                    action(e) {
                        self.copyCoordinatesToClipboard()
                    }
                }),
                '<div class="aladin-monospace-text"></div>'
            ]
        })
        el.addClass('aladin-location');

        super(el)

        self = this;
        ALEvent.CANVAS_EVENT.listenedBy(aladin.aladinDiv, function (e) {
            let param = e.detail;

            if (param.type === 'mouseout') {
                let [lon, lat] = aladin.wasm.getCenter();
                self.update({
                    lon: lon,
                    lat: lat,
                    frame: aladin.view.cooFrame,
                    isViewCenter: true,
                }, aladin);
            }

            if (param.type === 'mousemove' && param.state.dragging === false) {
                self.update({
                    mouseX: param.xy.x,
                    mouseY: param.xy.y,
                    frame: aladin.view.cooFrame,
                    isViewCenter: false,
                }, aladin);
            }
        });

        ALEvent.POSITION_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            self.update({
                lon: e.detail.lon, 
                lat: e.detail.lat,
                isViewCenter: true,
                frame: aladin.view.cooFrame
            }, aladin);
        });

        ALEvent.FRAME_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            let [lon, lat] = aladin.wasm.getCenter();

            self.update({
                lon: lon,
                lat: lat,
                isViewCenter: true,
                frame: e.detail.cooFrame
            }, aladin);
        });
    };

    update(options, aladin) {
        let self = this;
        const updateFromLonLatFunc = (lon, lat, cooFrame) => {
            var coo = new Coo(lon, lat, 7);
            let textEl = self.el.querySelector('.aladin-monospace-text')
            if (cooFrame == CooFrameEnum.J2000) {
                textEl.innerHTML = coo.format('s/');
            }
            else if (cooFrame == CooFrameEnum.J2000d) {
                textEl.innerHTML = coo.format('d/');
            }
            else {
                textEl.innerHTML = coo.format('d/');
            }

            textEl.style.color = options.isViewCenter ? aladin.getReticle().getColor() : 'white';
        };

        if (options.lon && options.lat) {
            updateFromLonLatFunc(options.lon, options.lat, options.frame, true);
        } else if (options.mouseX && options.mouseY) {
            let radec = aladin.pix2world(options.mouseX, options.mouseY); // This is given in the frame of the view
            if (radec) {
                if (radec[0] < 0) {
                    radec = [radec[0] + 360.0, radec[1]];
                }

                updateFromLonLatFunc(radec[0], radec[1], options.frame, false);
            }
        }
    }

    copyCoordinatesToClipboard() {
        let copyTextEl = this.el.querySelector('.aladin-monospace-text');
        var r = document.createRange();
        r.selectNode(copyTextEl);
        window.getSelection().removeAllRanges();
        window.getSelection().addRange(r);
        try {
            let successful = document.execCommand('copy');
            let msg = successful ? 'successful' : 'unsuccessful';
            console.log('Copying text command was ' + msg);
        } catch (err) {
            console.log('Oops, unable to copy');
        }
        window.getSelection().removeAllRanges();
    }
};

