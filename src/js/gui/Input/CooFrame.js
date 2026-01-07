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
 * File Location.js
 * 
 * Author: Thomas Boch[CDS]
 * 
 *****************************************************************************/


 import { CooFrameEnum }   from "../../CooFrameEnum";
 
 import { ALEvent } from "../../events/ALEvent.js";
 import { Input } from "./../Widgets/Input.js";
 
 export class CooFrame extends Input {
     // constructor
     constructor(aladin, options) {
         let self;
         let cooFrame = CooFrameEnum.fromString(aladin.options.cooFrame, CooFrameEnum.ICRS);

         super({
            name: 'cooFrame',
            type: 'select',
            value: cooFrame.label,
            options: [CooFrameEnum.ICRS.label, CooFrameEnum.ICRSd.label, CooFrameEnum.GAL.label],
            change(e) {
                aladin.setFrame(e.target.value)
            },
            classList: ['aladin-cooFrame'],
            tooltip: {
                content: cooFrame.explain,
                position: {
                    direction: 'bottom'
                }
            },
            ...options
        })

        this.addClass('aladin-medium-sized');

        self = this;

        this._addEventListeners(aladin);
    }

    _addEventListeners(aladin) {
        let self = this;
        ALEvent.FRAME_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            let frame = e.detail.cooFrame;

            self.update({
                value: frame.label,
                tooltip: {
                    content: frame.explain,
                    position: {
                        direction: 'bottom'
                    }
                },
            }, aladin);
        });
    }
};
 
 