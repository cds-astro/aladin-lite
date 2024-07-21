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

import { CtxMenuActionButtonOpener } from "./CtxMenuOpener";
import stackOverlayIconUrl from './../../../../assets/icons/stack.svg';
import { OverlayStackBox } from "../Box/StackBox";

import { ActionButton } from "./../Widgets/ActionButton";
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
 * @extends CtxMenuActionButtonOpener
 */
 export class OverlayStackButton extends ActionButton {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(aladin, options) {
        let self;
        let stack;
        super({
            icon: {
                size: 'medium',
                monochrome: true,
                url: stackOverlayIconUrl
            },
            classList: ['aladin-stack-control'],
            tooltip: {
                content: 'Open the overlays menu',
                position: {
                    direction: 'top right'
                }
            },
            toggled: false,
            action(e) {
                if (stack.isHidden) {
                    stack._show({
                        position: {
                            nextTo: self,
                            direction: 'right'
                        }
                    })
                } else {
                    stack._hide()
                }
            },
            ...options
        });
        self = this;
        stack = new OverlayStackBox(aladin, self);
    }
}
