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
                self.toggle(o)
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
        
        if (this.toggled && this.options.on) {
            this.options.on(o)
        }

        if (!this.toggled && this.options.off) { 
            this.options.off(o)
        }

        // once the actions has been executed, modify the styling
        this.update({toggled: this.toggled})
    }

    // It may happen that the widget closes and so the toggler
    // has to be notified. For example when the user clicks on a Box that
    // is attached to a toggler.
    notify(state) {
        if (this.toggled === state)
            return;

        this.toggled = state;
        this.update({toggled: this.toggled})
    }
}

/**
 * Class representing a Tabs layout
 * @extends TogglerActionButton
 */
 export class WidgetTogglerButton extends TogglerActionButton {
    /**
     * UI responsible for displaying the viewport infos
     * @param {Aladin} aladin - The aladin instance.
     */
    constructor(options) {
        let self;

        let {position, obj} = options && options.widget;

        let widget = obj;
        let enable = options && options.enable;

        super({
            toggled: false,
            on(o) {
                if (enable)
                    enable(o)

                widget._show({position})
            },
            off(_) {
                self.close();
            },
            ...options
        });
        self = this;

        widget.setToggler(this);
        this.widget = widget;

        if (position && position.direction) {
            position['nextTo'] = this;
        }
    }

    close() {
        this.widget._hide();

        super.close()
    }
}
