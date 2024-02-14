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
import copyIconBtn from '../../../assets/icons/copy.svg';

import { ALEvent } from "../events/ALEvent.js";
import { Layout } from "./Layout.js";
import { ActionButton } from "./Widgets/ActionButton.js";
import { Utils } from "../Utils";

export class Location extends DOMElement {
    // constructor
    constructor(aladin) {
        let self;

        let el = Layout.horizontal({
            layout: [
                ActionButton.createSmallSizedIconBtn({
                    iconURL: copyIconBtn,
                    tooltip: {content: 'Copy to clipboard!', position: {direction: 'bottom'}},
                    action(e) {
                        self.copyCoordinatesToClipboard()
                    }
                }),
                '<div class="aladin-monospace-text" style="white-space: pre;"></div>'
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

        this.aladin = aladin;

        let [lon, lat] = aladin.wasm.getCenter();
        this.update({
            lon: lon,
            lat: lat,
            isViewCenter: true,
            frame: aladin.view.cooFrame
        }, aladin)
    };

    static prec = 7;

    update(options, aladin) {
        let self = this;
        const updateFromLonLatFunc = (lon, lat, cooFrame) => {
            var coo = new Coo(lon, lat, Location.prec);
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
            try {
                let radec = aladin.pix2world(options.mouseX, options.mouseY); // This is given in the frame of the view
                if (radec) {
                    if (radec[0] < 0) {
                        radec = [radec[0] + 360.0, radec[1]];
                    }
    
                    updateFromLonLatFunc(radec[0], radec[1], options.frame, false);
                }
            } catch(e) {}
        }
    }

    copyCoordinatesToClipboard() {
        let copyTextEl = this.el.querySelector('.aladin-monospace-text');

        let msg;
        navigator.clipboard.writeText(copyTextEl.innerText)
            .then(() => {
                msg = 'successful'
                if (this.aladin.statusBar) {
                    this.aladin.statusBar.appendMessage({
                        message: 'Reticle location saved!',
                        duration: 2000,
                        type: 'info'
                    })
                }
            })
            .catch((e) => {
                msg = 'unsuccessful'
                console.info('Oops, unable to copy', e);
            })
            .finally(() => {
                console.info('Copying text command was ' + msg);
            })        
    }
};

