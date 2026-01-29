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

import { WidgetTogglerButton } from "./Toggler.js";
/*
export class CtxMenuActionButtonOpener extends ActionButton {

    // Constructor
    constructor(options, aladin) {
        let self;

        const enableTooltips = () => {
            aladin.aladinDiv.removeEventListener('click', enableTooltips);

            aladin.aladinDiv.querySelectorAll('.aladin-tooltip')
                // for each tooltips reset its visibility and transition delay
                .forEach((t) => {
                    t.style.visibility = ''
                    t.style.transitionDelay = ''
                })
        };

        super({
            action(e) {
                enableTooltips()

                let wasClosed = self.ctxMenu.isHidden;
                self.close()

                if (self.ctxMenu.toggler === self && !wasClosed) {
                    return;
                }

                // If it was hidden then reopen it
                self.open(e);

                // the panel is now open and we know the button has a tooltip
                // => we close it!
                if (self.tooltip && !self.ctxMenu.isHidden) {
                    self.tooltip.element().style.visibility = 'hidden'
                    self.tooltip.element().style.transitionDelay = '0ms';

                    aladin.aladinDiv.addEventListener("click", enableTooltips)
                }
            },
            ...options,
        })

        self = this;

        this.ctxMenu = aladin.contextMenu;
        this.layout = options.ctxMenu;
    }

    close() {
        this.closed = true;
        this.ctxMenu._hide();
    }

    open(e) {
        if (this.layout) {
            this.ctxMenu.attach(this.layout, this)
        }

        this.ctxMenu.show({
            position: {
                nextTo: this,
                direction: this.options.openDirection,
            },
        });

        this.closed = false;
    }

    _hide() {
        this.close();
        super._hide();
    }
}*/

export class CtxMenuActionButtonOpener extends WidgetTogglerButton {

    // Constructor
    constructor(options, aladin) {
        let self;

        const enableTooltips = () => {
            aladin.aladinDiv.removeEventListener('click', enableTooltips);

            aladin.aladinDiv.querySelectorAll('.aladin-tooltip')
                // for each tooltips reset its visibility and transition delay
                .forEach((t) => {
                    t.style.visibility = ''
                    t.style.transitionDelay = ''
                })
        };
        super({
            widget: {
                obj: aladin.contextMenu,
                position: {direction: (options && options.openDirection) || 'right'}
            },
            enable(e) {
                enableTooltips()
                // If it was hidden then reopen it
                if (self.layout) {
                    self.ctxMenu.attach(self.layout, self)
                }

                // the panel is now open and we know the button has a tooltip
                // => we close it!
                if (self.tooltip && !self.ctxMenu.isHidden) {
                    self.tooltip.element().style.visibility = 'hidden'
                    self.tooltip.element().style.transitionDelay = '0ms';

                    aladin.aladinDiv.addEventListener("click", enableTooltips)
                }
            },
            ...options,
        })

        self = this;

        this.ctxMenu = aladin.contextMenu;
        this.layout = options.ctxMenu;
    }

    update(options) {
        if (options && options.ctxMenu) {
            console.log(this.ctxMenu, "attach", options.ctxMenu)
            this.layout = options.ctxMenu;
            this.ctxMenu.attach(this.layout, this)
        }

        super.update(options)
    }
}