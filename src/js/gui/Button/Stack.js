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

import { OverlayStackBox } from "../Box/StackBox";
import { WidgetTogglerButton } from "./Toggler";
import stackOverlayIconUrl from "./../../../../assets/icons/stack.svg";
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
 * Class representing the stack
 * @extends WidgetTogglerButton
 */
 export class Stack extends WidgetTogglerButton {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin, options) {
        super({
            openDirection: (options && options.openDirection) || 'right',
            widget: new OverlayStackBox(aladin),
            icon: {
                size: 'medium',
                monochrome: true,
                url: stackOverlayIconUrl
            },
            classList: ['aladin-stack-control'],
            tooltip: {
                content: 'Open the overlays menu',
                position: {
                    direction: (options && options.openDirection) || 'left'
                }
            },
        });
    }
}
