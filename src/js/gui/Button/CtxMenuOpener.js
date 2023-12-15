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

 /*
 options = {
     action: (connector) => {
 
     }
     tooltip
 }
 */
export class CtxMenuActionButtonOpener extends ActionButton {
    // Constructor
    constructor(options, aladin) {
        let self;

        let layout = options.ctxMenu;
        let ctxMenu = ContextMenu.getInstance(aladin)
        super({
            ...options,
            cssStyle: {
                backgroundPosition: 'center center',
                cursor: 'pointer',
                ...options.cssStyle
            },
            action(e) {
                if (options.action) {
                    options.action(e)
                }

                if (self.ctxMenu.isHidden) {
                    self.ctxMenu.attach(layout)
                    self.ctxMenu.show({
                        position: {
                            nextTo: self,
                            direction: options.openDirection || 'bottom',
                        },
                        cssStyle: options.ctxMenu && options.ctxMenu.cssStyle
                    });
                } else {
                    self.ctxMenu._hide();
                }
            }
        })

        self = this;
        self.ctxMenu = ctxMenu;
    }

    hideCtxMenu() {
        this.ctxMenu._hide();
    }

    update(options) {
        if(options.ctxMenu) {
            this.ctxMenu = options.ctxMenu;
        }

        super.update(options)
    }
}