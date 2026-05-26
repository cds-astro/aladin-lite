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

/******************************************************************************
 * Aladin Lite project
 * 
 * File FoV.js
 * 
 * Author: Matthieu Baumann[CDS]
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
            let zoomIn = new ActionButton({
                classList: 'aladin-zoom-in',
                size: 'small',
                tooltip: {content: 'zoom in', position: {direction: 'left'}},
                icon: {
                    monochrome: true,
                    size: 'small',
                    url: plusIconUrl,
                },
                action(o) {
                    aladin.increaseZoom();
                }
            })
            let zoomOut = new ActionButton({
                size: 'small',
                classList: 'aladin-zoom-out',
                tooltip: {content: 'zoom out', position: {direction: 'left'}},
                icon: {
                    monochrome: true,
                    size: 'small',
                    url: minusIconUrl,
                },
                action(o) {
                    aladin.decreaseZoom();
                }
            });
            zoomIn.el.classList.add('aladin-zoom-in');
            zoomOut.el.classList.add('aladin-zoom-out');

            let aladinZoomDiv = document.createElement("div")
            aladinZoomDiv.classList.add('aladin-zoom')
            aladinZoomDiv.appendChild(zoomIn.element());
            aladinZoomDiv.appendChild(zoomOut.element());

            aladin.aladinDiv.appendChild(aladinZoomDiv);

            //layout.push(zoomOut)
            //layout.push(zoomIn)
        }

        if (options.showFov) {
            layout.push(
                '<div class="aladin-monospace-text"></div>' +
                '<div class="aladin-monospace-text">&times;</div>' +
                '<div class="aladin-monospace-text"></div>'
            )
        }

        let el = Layout.horizontal(layout);
        if (el) {
            el.addClass('aladin-fov');
        }

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
        let [fovX, _, fovY] = this.el.querySelectorAll('.aladin-monospace-text')
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

