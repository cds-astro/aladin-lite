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
import { ContextMenu } from "../Widgets/ContextMenu.js";

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
            ...options,
            cssStyle: {
                backgroundPosition: 'center center',
                cursor: 'pointer',
                ...options.cssStyle
            },
            action(e) {
                enableTooltips()

                let isHidden = self.ctxMenu.isHidden

                ContextMenu.hideAll();

                // If it was hidden then reopen it
                if (isHidden) {
                    if (options.action) {
                        options.action(e)
                    }

                    if (self.layout) {
                        self.ctxMenu.attach(self.layout)
                    }

                    self.ctxMenu.show({
                        position: {
                            nextTo: self,
                            direction: options.openDirection,
                        },
                    });
                }

                // the panel is now open and we know the button has a tooltip
                // => we close it!
                if (self.tooltip && !self.ctxMenu.isHidden) {
                    self.tooltip.element().style.visibility = 'hidden'
                    self.tooltip.element().style.transitionDelay = '0ms';

                    aladin.aladinDiv.addEventListener("click", enableTooltips)
                }
            }
        })

        self = this;

        let ctxMenu;
        if (options.ctxMenu instanceof ContextMenu) {
            ctxMenu = options.ctxMenu;
        } else {
            this.layout = options.ctxMenu;
            ctxMenu = new ContextMenu(aladin, {hideOnClick: true, hideOnResize: true})
        }

        self.ctxMenu = ctxMenu;
    }

    hideMenu() {
        this.ctxMenu._hide();
    }

    _hide() {
        this.hideMenu();
        super._hide();
    }

    update(options) {
        if(options.ctxMenu) {
            if (options.ctxMenu instanceof ContextMenu) {
                this.ctxMenu = options.ctxMenu
            } else {
                this.layout = options.ctxMenu;
            }
        }

        if (!this.ctxMenu) {
            this.ctxMenu = new ContextMenu(aladin, {hideOnClick: true, hideOnResize: true})
        }

        super.update(options)

        if (!this.ctxMenu.isHidden) {
            if (this.layout) {
                this.ctxMenu.attach(this.layout)
            }

            this.ctxMenu.show({
                position: {
                    nextTo: this,
                    // it case it is not given then it will be computed by default
                    direction: options.openDirection,
                },
            });
        }
    }
}