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
 * File gui/Stack/Menu.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/

import { WidgetTogglerButton } from "./Toggler.js";
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
            widget: aladin.contextMenu,
            enable(e) {
                // If it was hidden then reopen it
                if (self.layout && self.ctxMenu) {
                    self.ctxMenu.attach(self.layout, self)
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
            this.layout = options.ctxMenu;
            //this.ctxMenu.attach(this.layout, this)
        }

        super.update(options)
    }
}