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

import { LayerEditBox } from "./EditBox";
import { FSM } from "../../../../FiniteStateMachine";
import { Stack } from "./Stack.js";
import { DOMElement } from "../../../Widgets/Widget.js";

export class StackLayerMenu extends DOMElement {
    // Constructor
    constructor(aladin, menu) {
        super()
        this.aladin = aladin;
        this.menu = menu;

        let self = this;

        let hideOnDispatch = () => {
            let stack = Stack.getInstance(this.aladin, this.menu, this.fsm);
            stack.hide();
            let editBox = LayerEditBox.getInstance(this.aladin, this.menu);
            editBox._hide();

            super._hide();
        }

        let showOnDispatch = () => {
            let stack = Stack.getInstance(aladin, menu, self.fsm);
            stack.show();

            super._show();
        }
        self.fsm = new FSM({
            state: 'hide',
            transitions: {
                hide: {
                    displayStack: showOnDispatch
                },
                displayStack: {
                    displayEditBox(params) {
                        let stack = Stack.getInstance(aladin, menu, self.fsm);
                        stack.hide();

                        let editBox = LayerEditBox.getInstance(aladin, menu);
                        editBox.update({layer: params.layer})
                    },
                    hide: hideOnDispatch
                },
                displayEditBox: {
                    hide: hideOnDispatch
                }
            }
        })
    }

    _show() {
        this.fsm.dispatch('displayStack');
    }

    _hide() {
        this.fsm.dispatch("hide")
    }
    
    static singleton;

    static getInstance(aladin, menu) {
        if (!StackLayerMenu.singleton) {
            StackLayerMenu.singleton = new StackLayerMenu(aladin, menu);
        }

        return StackLayerMenu.singleton;
    }
}
  