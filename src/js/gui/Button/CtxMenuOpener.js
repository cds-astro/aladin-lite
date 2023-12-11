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
    constructor(options) {
        let self;

        let ctxMenu = options.ctxMenu;

        super({
            ...options,
            cssStyle: {
                backgroundPosition: 'center center',
                cursor: 'pointer',
                ...options.cssStyle
            },
            action(e) {
                if (ctxMenu.isHidden) {
                    ctxMenu.show({
                        position: {
                            anchor: self,
                            direction: 'bottom',
                        },
                        cssStyle: options.ctxMenu && options.ctxMenu.cssStyle
                    });
                } else {
                    ctxMenu._hide();
                }

                if (options.action) {
                    options.action(e)
                }
            }
        })

        self = this;
        this.ctxMenu = ctxMenu;
    }
}