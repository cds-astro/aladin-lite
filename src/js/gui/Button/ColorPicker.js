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
 * File gui/Stack/Menu.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/

import { ActionButton } from "../Widgets/ActionButton.js";
import targetIcon from '../../../../assets/icons/color-picker.svg';
import { ALEvent } from "../../events/ALEvent.js";
import { View } from "../../View.js";
export class ColorPicker extends ActionButton {
    // Constructor
    constructor(aladin) {
        let self;
        super({
            icon: {
                url: targetIcon,
                monochrome: true,
            },
            classList: ['aladin-colorPicker-control'],
            size: 'medium',
            tooltip: {
                content: 'A color picker tool',
                position: { direction: 'top right' },
            },
            action(o) {
                if (self.mode !== View.TOOL_COLOR_PICKER) {
                    aladin.fire('colorpicker');
                } else {
                    aladin.fire('default');
                }
            }
        })
        self = this;

        this.aladin = aladin;
        this.mode = aladin.view.mode;

        this.addListeners()
    }

    updateStatus() {
        if (this.mode === View.TOOL_COLOR_PICKER) {
            if (this.aladin.statusBar) {
                this.aladin.statusBar.appendMessage({
                    id: 'colorpicker',
                    message: 'Color picker mode, click on a pixel to copy it',
                    type: 'info'
                })
            }
        } else {
            if (this.aladin.statusBar) {
                this.aladin.statusBar.removeMessage('colorpicker')
            }
        }

        this.update({toggled: this.mode === View.TOOL_COLOR_PICKER})
    }

    addListeners() {
        ALEvent.MODE.listenedBy(this.aladin.aladinDiv, e => {
            let mode = e.detail.mode;
            this.mode = mode;

            this.updateStatus();
        });
    }
}
 