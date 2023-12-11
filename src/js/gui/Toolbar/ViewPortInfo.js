// Copyright 2023 - UDS/CNRS
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

import { Layout } from "../Layout";
import { Input } from "../Widgets/Input";
import { DOMElement } from "../Widgets/Widget";
import { CooFrameEnum } from "../../CooFrameEnum";
import { Location } from "../Location";
import { FoV } from "../FoV";
import { ProjectionActionButton } from "../Button/Projection";

/******************************************************************************
 * Aladin Lite project
 *
 * File gui/ActionButton.js
 *
 * A context menu that shows when the user right clicks, or long touch on touch device
 *
 *
 * Author: Matthieu Baumann[CDS]
 *
 *****************************************************************************/

/**
 * Class representing a Tabs layout
 * @extends DOMElement
 */
 export class ViewPortInfo extends DOMElement {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin) {
        let cooFrame = CooFrameEnum.fromString(aladin.options && aladin.options.cooFrame, CooFrameEnum.J2000);


        let layout = [];
        // Add the projection control
        if (aladin.options && aladin.options.showProjectionControl) {
            layout.push(new ProjectionActionButton(aladin))
        }
        // Add the frame control
        if (aladin.options && aladin.options.showFrame) {
            layout.push(new Input({
                layout: {
                    name: 'frame',
                    type: 'select',
                    value: cooFrame.label,
                    options: [CooFrameEnum.J2000.label, CooFrameEnum.J2000d.label, CooFrameEnum.GAL.label],
                    change(e) {
                        aladin.setFrame(e.target.value)
                    }
                }
            }))
        }
        // Add the location info
        layout.push(new Location(aladin));
        // Add the FoV info
        layout.push(new FoV(aladin))
        let el = Layout.horizontal({
            layout: layout
        });

        super(el)
    }
}
