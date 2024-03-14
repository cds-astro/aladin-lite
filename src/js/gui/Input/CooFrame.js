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


 import { CooFrameEnum }   from "../../CooFrameEnum";
 
 import { ALEvent } from "../../events/ALEvent.js";
 import { Input } from "./../Widgets/Input.js";
 
 export class CooFrame extends Input {
     // constructor
     constructor(aladin, options) {
         let self;
         let cooFrame = CooFrameEnum.fromString(aladin.options.cooFrame, CooFrameEnum.J2000);

         super({
            name: 'cooFrame',
            type: 'select',
            value: cooFrame.label,
            options: [CooFrameEnum.J2000.label, CooFrameEnum.J2000d.label, CooFrameEnum.GAL.label],
            change(e) {
                aladin.setFrame(e.target.value)
            },
            classList: ['aladin-cooFrame'],
            tooltip: {
                content: "Change the frame",
                position: {
                    direction: 'bottom'
                }
            },
            ...options
        })

        self = this;

        this._addEventListeners(aladin);
    }

    _addEventListeners(aladin) {
        let self = this;
        ALEvent.FRAME_CHANGED.listenedBy(aladin.aladinDiv, function (e) {
            let frame = e.detail.cooFrame;

            self.update({
                value: frame.label
            }, aladin);
        });
    }
};
 
 