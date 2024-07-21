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

export class TogglerActionButton extends ActionButton {
     // Constructor
    constructor(options) {
        let self;
        let toggled = false;
        if (options.toggled !== undefined) {
            toggled = options.toggled;
        }

        super({
            ...options,
            toggled,
            action(o) {
                options.action && options.action(o);
                self.toggle(o);
            }
        })
        this.toggled = toggled;

        self = this;
    }

    close() {
        if (this.toggled) {
            this.toggle()
        }
    }

    open() {
        if (!this.toggled) {
            this.toggle();
        }
    }

    toggle(o) {
        this.toggled = !this.toggled;
        
        if (this.toggled && this.options.actionOn) {
            this.options.actionOn(o)
        }

        if (!this.toggled && this.options.actionOff) { 
            this.options.actionOff(o)
        }

        // once the actions has been executed, modify the styling
        this.update({toggled: this.toggled, tooltip: this.toggled ? this.options.tooltipOn : this.options.tooltipOff})
    }
}