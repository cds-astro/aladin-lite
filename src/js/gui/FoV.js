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
import { Numbers } from "../libs/astro/coo.js";
import { Layout } from "./Layout.js";

import { DOMElement } from "./Widgets/Widget.js";

import { ALEvent } from "../events/ALEvent.js";
import { ActionButton } from "./Widgets/ActionButton.js";

import plusIconUrl from "../../../assets/icons/plus.svg"
import minusIconUrl from "../../../assets/icons/minus.svg"

export class FoV extends DOMElement {
    // constructor
    constructor(aladin, options) {
        let layout = [];

        if (options.showZoomControl) {
            layout.push(new ActionButton({
                size: 'small',
                icon: {
                    monochrome: true,
                    size: 'small',
                    url: plusIconUrl,
                },
                cssStyle: {
                    marginRight: 0,
                    borderRight: 'none',
                    borderRadius: '5px 0px 0px 5px'
                },
                action(o) {
                    aladin.increaseZoom();
                }
            }))
            layout.push(new ActionButton({
                size: 'small',
                cssStyle: {
                    borderRadius: '0px 5px 5px 0px'
                },
                icon: {
                    monochrome: true,
                    size: 'small',
                    url: minusIconUrl,
                },
                action(o) {
                    aladin.decreaseZoom();
                }
            }))
        }

        if (options.showFov) {
            layout.push(...['<div class="aladin-monospace-text"></div>',
            '<div class="aladin-label-text">&times;</div>',
            '<div class="aladin-monospace-text"></div>'])
        }

        let el = Layout.horizontal(layout);
        el.addClass('aladin-fov');
        el.addClass('aladin-dark-theme')

        super(el)

        if (options.showFov) {
            let self = this;
            ALEvent.ZOOM_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
                let [fovXDeg, fovYDeg] = aladin.getFov();
    
                self._update(fovXDeg, fovYDeg)
            });
    
            let [fovXDeg, fovYDeg] = aladin.getFov();
            self._update(fovXDeg, fovYDeg)
        }
    };

    _update(fovXDeg, fovYDeg) {
        let [fovX, fovY] = this.el.querySelectorAll('.aladin-monospace-text')
        fovX.innerText = this._format(fovXDeg) 
        fovY.innerText = this._format(fovYDeg) 
    }

    _format(fovDeg) {
        let suffix;
        let fov;
        if (Math.floor(fovDeg) == 0) {
            let fovMin = fovDeg*60.0;
    
            if (Math.floor(fovMin) == 0) {
                // sec
                suffix = '"';
                fov = fovMin*60.0;
            } else {
                // min
                suffix = '\'';
                fov = fovMin;
            }
        } else {
            // d
            suffix = '°';
            fov = fovDeg;
        }
    
        return Numbers.toDecimal(fov, 1) + suffix;
    }
};

