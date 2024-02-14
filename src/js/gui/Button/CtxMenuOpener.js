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
    //static BUTTONS = [];

    // Constructor
    constructor(options, aladin) {
        let self;

        let ctxMenu = new ContextMenu(aladin, {hideOnClick: true, hideOnResize: true})
        super({
            ...options,
            cssStyle: {
                backgroundPosition: 'center center',
                cursor: 'pointer',
                ...options.cssStyle
            },
            action(e) {

                //self.ctxMenu._hide();

                if (self.ctxMenu.isHidden === true) {
                    if (options.action) {
                        options.action(e)
                    }

                    self.ctxMenu.attach(self.layout)
                    self.ctxMenu.show({
                        position: {
                            nextTo: self,
                            direction: options.openDirection || 'bottom',
                        },
                        cssStyle: options.ctxMenu && options.ctxMenu.cssStyle
                    });

                    //CtxMenuActionButtonOpener.BUTTONS.forEach(b => {b.hidden = true})
                } else {
                    self.hideMenu();
                }

                //self.hidden = !self.hidden;
            }
        })

        //this.hidden = true;

        this.layout = options.ctxMenu;

        self = this;
        self.ctxMenu = ctxMenu;

        //CtxMenuActionButtonOpener.BUTTONS.push(this);
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
            this.layout = options.ctxMenu;
        }

        super.update(options)

        if (!this.ctxMenu.isHidden) {
            this.ctxMenu.attach(this.layout)
            this.ctxMenu.show({
                position: {
                    nextTo: this,
                    direction: options.openDirection || 'bottom',
                },
                cssStyle: options.ctxMenu && options.ctxMenu.cssStyle
            });
        }
    }
}